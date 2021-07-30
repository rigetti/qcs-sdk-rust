#![deny(clippy::all)]
#![deny(clippy::pedantic)]
// C doesn't have namespaces, so exported functions should contain the module name
#![allow(clippy::module_name_repetitions)]

pub use crate::qpu::run_program_on_qpu;
pub use crate::qvm::run_program_on_qvm;

use eyre::{eyre, Report, Result};
use libc::{c_char, c_double, c_ushort};
use std::ffi::CString;

mod qpu;
mod qvm;

// ANCHOR: free_program_result
/// Frees the memory of a [`QVMResponse`] as allocated by [`run_program_on_qvm`]
///
/// # Safety
/// This function should only be called with the result of [`run_program_on_qvm`]
// ANCHOR_END: free_program_result
#[no_mangle]
pub unsafe extern "C" fn free_program_result(response: ProgramResult) {
    let rust_managed = response.into_rust();
    drop(rust_managed);
}

/// The return value of [`run_program_on_qvm`] or [`run_program_on_qpu`].
///
/// # Safety
/// In order to properly free the memory allocated in this struct, call [`free_program_result`]
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
/// ProgramResult result = run_program_on_qvm(program, 3, "ro");
/// ```
/// If `error` is `NULL` then `results_by_shot` will look something like:
///
/// ```
/// results_by_shot = [[0, 0], [0, 0], [0, 0]]
/// ```
///
/// where `results_by_shot[shot][bit]` can access the value of `ro[bit]` for a given `shot`.
#[repr(C)]
pub enum ProgramResult {
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

impl ProgramResult {
    fn from_rust(data: qcs::ProgramResult) -> Self {
        match data {
            qcs::ProgramResult::I8(data) => ProgramResult::from(data),
            qcs::ProgramResult::F64(data) => ProgramResult::from(data),
            _ => {
                return Self::from(eyre!(
                    "Only BIT, OCTET, and REAL DECLARE instructions are currently supported."
                ));
            }
        }
    }

    unsafe fn into_rust(self) -> Result<qcs::ProgramResult> {
        match self {
            ProgramResult::Error(error) => {
                if error.is_null() {
                    return Err(eyre!("Unknown error"));
                }
                // SAFETY: If this was manually constructed with a null-terminated string, bad things
                // will happen here. Proper usage should only see an error message here that was
                // constructed from `QVMResponse::from`
                let c_string = CString::from_raw(error);
                return Err(eyre!(c_string.into_string()?));
            }
            ProgramResult::Byte {
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

                Ok(qcs::ProgramResult::I8(results))
            }
            ProgramResult::Real {
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

                Ok(qcs::ProgramResult::F64(results))
            }
        }
    }
}

impl From<Report> for ProgramResult {
    fn from(err: Report) -> Self {
        let c_string = CString::new(format!("{:?}", err)).expect("Rust strings aren't null!");
        let ptr = c_string.into_raw();
        Self::Error(ptr)
    }
}

impl From<Vec<Vec<i8>>> for ProgramResult {
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

impl From<Vec<Vec<f64>>> for ProgramResult {
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
