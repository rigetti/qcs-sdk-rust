#![deny(clippy::all)]
#![deny(clippy::pedantic)]
// C doesn't have namespaces, so exported functions should contain the module name
#![allow(clippy::module_name_repetitions)]

use std::ffi::{CStr, CString};

use eyre::{eyre, Report, Result};
use std::os::raw::{c_char, c_double, c_uint, c_ushort};

pub use crate::qpu::execute_on_qpu;
pub use crate::qvm::execute_on_qvm;

mod qpu;
mod qvm;

/// Constructs an [`Executable`] and returns a raw pointer to it.
///
/// # Safety
///
/// 1. The result of this function must later be passed to [`free_executable`] or the memory will be leaked.
/// 2. `quil` must be a valid, non-NULL, nul-terminated string which must remain constant and valid until [`free_executable`] is called.
///
/// # Arguments
///
/// 1. `quil`: A string containing a valid Quil program
///
/// # Errors
///
/// If an error occurs in this function, NULL will be returned.
///
/// 1. The contents of `quil` were not valid UTF-8. In this case, the returned value will be NULL.
#[no_mangle]
pub unsafe extern "C" fn executable_from_quil(quil: *mut c_char) -> *mut Executable {
    // SAFETY: If `quil` is NULL or not nul-terminated, this will cause issues.
    let quil = match CStr::from_ptr(quil).to_str() {
        Ok(rust_str) => rust_str,
        Err(_) => {
            return Box::into_raw(Box::new(Executable {
                inner: Err(eyre!("quil was not a valid UTF-8 string")),
            }));
        }
    };
    // SAFETY: `quil` is assumed to live longer than this Executable, if it's freed before
    // `free_executable` is called bad things will happen.
    Box::into_raw(Box::new(Executable {
        inner: Ok(qcs::Executable::from_quil(quil)),
    }))
}

/// Free an [`Executable`]
///
/// # Safety
///
/// 1. Only call this with the non-null output of [`executable_from_quil`].
/// 2. Only call this function once per executable if you don't want to double-free your memory.
#[no_mangle]
pub unsafe extern "C" fn free_executable(executable: *mut Executable) {
    drop(Box::from_raw(executable));
}

/// Holds the state required to execute a program.
/// Intentionally opaque to C.
pub struct Executable {
    inner: Result<qcs::Executable<'static, 'static>>,
}

/// Set the value of a parameter for parametric execution.
///
/// # Safety
///
/// 1. `executable` must be the non-NULL result of [`executable_from_quil`]
/// 2. `name` must be a valid, non-NULL, nul-terminated string. It must also live until `executable`
///     is freed.
///
/// # Arguments
///
/// 1. `executable`: The [`Executable`] to set the parameter on.
/// 2. `name`: The name of the memory region to set, must match a Quil `DECLARE` statement exactly.
/// 3. `index`: The index into the named memory region to set.
/// 3. `value`: The value to set the parameter to.
///
/// # Errors
///
/// If an error occurs, the return value of this function will be non-null
#[no_mangle]
pub unsafe extern "C" fn set_param(
    executable: *mut Executable,
    name: *mut c_char,
    index: c_uint,
    value: c_double,
) {
    // SAFETY: Bad things can happen here if `executable` is null, already freed, or invalid.
    let mut executable = Box::from_raw(executable);

    executable.inner = executable.inner.and_then(|mut inner| {
        // SAFETY: If `name` is NULL or not nul-terminated, this will cause issues.
        let name = CStr::from_ptr(name)
            .to_str()
            .map_err(|_| eyre!("name was not valid UTF-8"))?;
        inner.with_parameter(name, index as usize, value);
        Ok(inner)
    });

    Box::into_raw(executable);
}

/// Set the program to run multiple times on the QPU.
///
/// # Safety
///
/// 1. `executable` must be the result of [`executable_from_quil`]
///
/// # Arguments
///
/// 1. `executable`: The [`Executable`] to set the parameter on.
/// 2. `shots`: The number of times to run the program for each execution.
#[no_mangle]
pub unsafe extern "C" fn wrap_in_shots(executable: *mut Executable, shots: c_ushort) {
    // SAFETY: Bad things can happen here if `executable` is null, already freed, or invalid.
    let mut executable = Box::from_raw(executable);

    executable.inner = executable.inner.map(|inner| inner.with_shots(shots));

    Box::into_raw(executable);
}

/// Set the memory location to read out of.
///
/// # Safety
///
/// 1. `executable` must be the result of [`executable_from_quil`]
/// 2. `register` must be a valid, non-NULL, nul-terminated string. It must also live until `executable`
///     is freed.
///
/// # Arguments
///
/// 1. `executable`: The [`Executable`] to set the parameter on.
/// 2. `register`: The name of the memory region to read out of. Must match a Quil `DECLARE`
///     statement exactly.
#[no_mangle]
pub unsafe extern "C" fn read_from(executable: *mut Executable, name: *mut c_char) {
    // SAFETY: Bad things can happen here if `executable` is null, already freed, or invalid.
    let mut executable = Box::from_raw(executable);

    executable.inner = executable.inner.and_then(|inner| {
        // SAFETY: If `register` is NULL or not nul-terminated, this will cause issues.
        let name = CStr::from_ptr(name)
            .to_str()
            .map_err(|_| eyre!("name was not valid UTF-8"))?;
        Ok(inner.read_from(name))
    });

    Box::into_raw(executable);
}

fn err_to_c_string(err: &Report) -> *mut c_char {
    let c_string = CString::new(format!("{:?}", err)).expect("Rust strings aren't null!");
    c_string.into_raw()
}

/// Frees the memory of a [`ExecutionResult`] as allocated by [`execute_on_qvm`] or [`execute_on_qpu`]
///
/// # Safety
/// This function should only be called with the result of one of the above functions.
#[no_mangle]
pub unsafe extern "C" fn free_execution_result(response: ExecutionResult) {
    let rust_managed = response.into_rust();
    drop(rust_managed);
}

/// The return value of [`execute_on_qvm`] or [`execute_on_qpu`].
///
/// # Safety
/// In order to properly free the memory allocated in this struct, call [`free_execution_result`]
/// with any instances created.
///
/// # Example
/// If you have a Quil program with an "ro" register containing two items:
///
/// ```quil
/// DECLARE ro BIT[2]
/// ```
/// and you run that program 3 times (shots)
///
/// ```C
/// ExecutionResult result = run_program_on_qvm(program, 3, "ro");
/// ```
/// If `error` is `NULL` then `results_by_shot` will look something like:
///
/// ```
/// results_by_shot = [[0, 0], [0, 0], [0, 0]]
/// ```
///
/// where `results_by_shot[shot][bit]` can access the value of `ro[bit]` for a given `shot`.
#[repr(C)]
pub enum ExecutionResult {
    Error(*mut c_char),
    Byte {
        number_of_shots: c_ushort,
        shot_length: c_ushort,
        data_per_shot: *mut *mut c_char,
    },
    Real {
        number_of_shots: c_ushort,
        shot_length: c_ushort,
        data_per_shot: *mut *mut c_double,
    },
}

impl ExecutionResult {
    fn from_rust(data: qcs::ExecutionResult) -> Self {
        match data {
            qcs::ExecutionResult::I8(data) => ExecutionResult::from(data),
            qcs::ExecutionResult::F64(data) => ExecutionResult::from(data),
            _ => Self::from(eyre!(
                "Only BIT, OCTET, and REAL DECLARE instructions are currently supported."
            )),
        }
    }

    unsafe fn into_rust(self) -> Result<qcs::ExecutionResult> {
        match self {
            ExecutionResult::Error(error) => {
                if error.is_null() {
                    return Err(eyre!("Unknown error"));
                }
                // SAFETY: If this was manually constructed with a null-terminated string, bad things
                // will happen here. Proper usage should only see an error message here that was
                // constructed from `QVMResponse::from`
                let c_string = CString::from_raw(error);
                Err(eyre!(c_string.into_string()?))
            }
            ExecutionResult::Byte {
                data_per_shot,
                number_of_shots,
                shot_length,
            } => {
                // SAFETY: If any of these pieces are wrong, this will read arbitrary memory
                let results: Vec<*mut i8> = Vec::from_raw_parts(
                    data_per_shot,
                    number_of_shots as usize,
                    number_of_shots as usize,
                );

                let results: Vec<Vec<i8>> = results
                    .into_iter()
                    // SAFETY: If any of these pieces are wrong, this will read arbitrary memory
                    .map(|ptr| Vec::from_raw_parts(ptr, shot_length as usize, shot_length as usize))
                    .collect();

                Ok(qcs::ExecutionResult::I8(results))
            }
            ExecutionResult::Real {
                number_of_shots,
                shot_length,
                data_per_shot,
            } => {
                // SAFETY: If any of these pieces are wrong, this will read arbitrary memory
                let results: Vec<*mut f64> = Vec::from_raw_parts(
                    data_per_shot,
                    number_of_shots as usize,
                    number_of_shots as usize,
                );

                let results: Vec<Vec<f64>> = results
                    .into_iter()
                    // SAFETY: If any of these pieces are wrong, this will read arbitrary memory
                    .map(|ptr| Vec::from_raw_parts(ptr, shot_length as usize, shot_length as usize))
                    .collect();

                Ok(qcs::ExecutionResult::F64(results))
            }
        }
    }
}

impl From<Report> for ExecutionResult {
    fn from(err: Report) -> Self {
        let ptr = err_to_c_string(&err);
        Self::Error(ptr)
    }
}

impl From<String> for ExecutionResult {
    fn from(err_string: String) -> Self {
        let ptr = CString::new(err_string).unwrap().into_raw();
        Self::Error(ptr)
    }
}

impl From<Vec<Vec<i8>>> for ExecutionResult {
    fn from(data: Vec<Vec<i8>>) -> Self {
        // Shots was passed into QVM originally as a u16 so this is safe.
        #[allow(clippy::cast_possible_truncation)]
        let number_of_shots = data.len() as u16;

        // This one is a guess. If more than 2^16 slots in a register then this will truncate
        #[allow(clippy::cast_possible_truncation)]
        let shot_length = data[0].len() as u16;

        let mut results: Vec<*mut i8> = IntoIterator::into_iter(data)
            .map(|mut shot| {
                let ptr = shot.as_mut_ptr();
                std::mem::forget(shot);
                ptr
            })
            .collect();
        let ptr = results.as_mut_ptr();
        std::mem::forget(results);
        #[allow(clippy::cast_possible_truncation)]
        Self::Byte {
            data_per_shot: ptr,
            number_of_shots,
            shot_length,
        }
    }
}

impl From<Vec<Vec<f64>>> for ExecutionResult {
    fn from(mut data: Vec<Vec<f64>>) -> Self {
        // Shots was passed into QVM originally as a u16 so this is safe.
        #[allow(clippy::cast_possible_truncation)]
        let number_of_shots = data.len() as u16;

        // This one is a guess. If more than 2^16 slots in a register then this will truncate
        #[allow(clippy::cast_possible_truncation)]
        let shot_length = data[0].len() as u16;

        data.shrink_to_fit();
        let mut results: Vec<*mut f64> = IntoIterator::into_iter(data)
            .map(|mut shot| {
                shot.shrink_to_fit();
                let ptr = shot.as_mut_ptr();
                std::mem::forget(shot);
                ptr
            })
            .collect();
        let ptr = results.as_mut_ptr();
        std::mem::forget(results);
        #[allow(clippy::cast_possible_truncation)]
        Self::Real {
            data_per_shot: ptr,
            number_of_shots,
            shot_length,
        }
    }
}
