#![deny(clippy::all)]
#![deny(clippy::pedantic)]
// C doesn't have namespaces, so exported functions should contain the module name
#![allow(clippy::module_name_repetitions)]

use std::ffi::{CStr, CString};

use eyre::{eyre, Result};
use std::os::raw::{c_char, c_double, c_uint, c_ushort};

pub use crate::qpu::execute_on_qpu;
pub use crate::qvm::execute_on_qvm;
use std::collections::HashMap;
use std::mem::ManuallyDrop;
use std::ptr::null;

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

/// Frees the memory of an [`ExecutionResult`] as allocated by [`execute_on_qvm`] or [`execute_on_qpu`]
///
/// # Safety
/// This function should only be called with the result of one of the above functions.
#[no_mangle]
pub unsafe extern "C" fn free_execution_result(result: ExecutionResult) {
    let rust_managed = result.into_rust();
    drop(rust_managed);
}

#[repr(C)]
pub enum ExecutionResult {
    Handle(*mut ResultHandle),
    Error(*mut c_char),
}

impl ExecutionResult {
    unsafe fn into_rust(self) -> Result<HashMap<Box<str>, qcs::ExecutionResult>> {
        match self {
            Self::Error(error) => {
                if error.is_null() {
                    return Err(eyre!("Unknown error"));
                }
                // SAFETY: If this was manually constructed with a null-terminated string, bad things
                // will happen here. Proper usage should only see an error message here that was
                // constructed from `Self::from`
                let c_string = CString::from_raw(error);
                Err(eyre!(c_string.into_string()?))
            }
            Self::Handle(handle) => Ok(Box::from_raw(handle).into_rust()),
        }
    }

    fn from_data(data: HashMap<Box<str>, qcs::ExecutionResult>) -> Self {
        Self::Handle(ResultHandle::from_data(data))
    }
}

impl From<String> for ExecutionResult {
    fn from(err_string: String) -> Self {
        let ptr = CString::new(err_string).unwrap().into_raw();
        Self::Error(ptr)
    }
}

/// The return value of [`execute_on_qvm`] or [`execute_on_qpu`].
///
/// Holds result data internally, intentionally opaque to C since it uses a map internally
pub struct ResultHandle(HashMap<Box<str>, ExecutionData>);

/// Return a pointer to the [`ExecutionResult`] for a specific register or null if the register is not found.
///
/// # Safety
/// All inputs must be non-null. `name` must be a nul-terminated string. `handle` must be the result
/// of a non-error call to [`execute_on_qvm`] or [`execute_on_qpu`]
#[no_mangle]
pub unsafe extern "C" fn get_data(
    handle: *const ResultHandle,
    name: *const c_char,
) -> *const ExecutionData {
    // SAFETY: If register is null or not nul-terminated, then bad things happen here.
    let register = match CStr::from_ptr(name).to_str() {
        Ok(register) => register,
        Err(_) => return null(),
    };
    // SAFETY: if handle is null or an invalid pointer than this is undefined behavior.
    match (*handle).0.get(register) {
        Some(result) => result as *const ExecutionData,
        None => null(),
    }
}

impl ResultHandle {
    fn from_data(data: HashMap<Box<str>, qcs::ExecutionResult>) -> *mut Self {
        let inner = data
            .into_iter()
            .filter_map(|(key, data)| ExecutionData::from_rust(data).map(|c_data| (key, c_data)))
            .collect();
        Box::into_raw(Box::new(Self(inner)))
    }

    unsafe fn into_rust(self) -> HashMap<Box<str>, qcs::ExecutionResult> {
        self.0
            .into_iter()
            .map(|(key, data)| (key, data.into_rust()))
            .collect()
    }
}

/// The contents of a single register within a [`ResultHandle`], fetched with [`get_data`]
#[repr(C)]
pub struct ExecutionData {
    number_of_shots: c_ushort,
    shot_length: c_ushort,
    data: DataType,
}

#[repr(C)]
pub enum DataType {
    Byte(*mut *mut c_char),
    Real(*mut *mut c_double),
}

impl ExecutionData {
    fn from_rust(data: qcs::ExecutionResult) -> Option<Self> {
        match data {
            qcs::ExecutionResult::I8(data) => Some(ExecutionData::from(data)),
            qcs::ExecutionResult::F64(data) => Some(ExecutionData::from(data)),
            _ => None,
        }
    }

    unsafe fn into_rust(self) -> qcs::ExecutionResult {
        let Self {
            number_of_shots,
            shot_length,
            data,
        } = self;
        match data {
            DataType::Byte(data_per_shot) => {
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

                qcs::ExecutionResult::I8(results)
            }
            DataType::Real(data_per_shot) => {
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

                qcs::ExecutionResult::F64(results)
            }
        }
    }
}

impl From<Vec<Vec<i8>>> for ExecutionData {
    fn from(data: Vec<Vec<i8>>) -> Self {
        // Shots was passed into QVM originally as a u16 so this is safe.
        #[allow(clippy::cast_possible_truncation)]
        let number_of_shots = data.len() as u16;

        // This one is a guess. If more than 2^16 slots in a register then this will truncate
        #[allow(clippy::cast_possible_truncation)]
        let shot_length = data[0].len() as u16;

        let results: Vec<*mut i8> = IntoIterator::into_iter(data)
            .map(|mut shot| {
                shot.shrink_to_fit();
                ManuallyDrop::new(shot).as_mut_ptr()
            })
            .collect();
        #[allow(clippy::cast_possible_truncation)]
        Self {
            data: DataType::Byte(ManuallyDrop::new(results).as_mut_ptr()),
            number_of_shots,
            shot_length,
        }
    }
}

impl From<Vec<Vec<f64>>> for ExecutionData {
    fn from(mut data: Vec<Vec<f64>>) -> Self {
        // Shots was passed into QVM originally as a u16 so this is safe.
        #[allow(clippy::cast_possible_truncation)]
        let number_of_shots = data.len() as u16;

        // This one is a guess. If more than 2^16 slots in a register then this will truncate
        #[allow(clippy::cast_possible_truncation)]
        let shot_length = data[0].len() as u16;

        data.shrink_to_fit();
        let results: Vec<*mut f64> = IntoIterator::into_iter(data)
            .map(|mut shot| {
                shot.shrink_to_fit();
                ManuallyDrop::new(shot).as_mut_ptr()
            })
            .collect();
        #[allow(clippy::cast_possible_truncation)]
        Self {
            data: DataType::Real(ManuallyDrop::new(results).as_mut_ptr()),
            number_of_shots,
            shot_length,
        }
    }
}
