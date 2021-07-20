use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::ptr::null_mut;

use eyre::{eyre, Report, Result, WrapErr};
use libc::{c_char, c_uchar, c_ushort};

use qvm::run_program;

/// Given a Quil program as a string, run that program on a local QVM.
///
/// # Safety
///
/// In order to run this function safely, you must provide the return value from this
/// function to [`free_qvm_response`] once you're done with it. The inputs `program` and
/// `register_name` must be valid, nul-terminated, non-null strings which remain constant for
/// the duration of this function.
///
/// # Usage
///
/// In order to execute, QVM must be running at <http://localhost:5000>.
///
/// # Arguments
///
/// 1. `program`: A string containing a valid Quil program. Any measurements that you'd like
/// to get back out must be in a register matching `register_name`. For example, if you have
/// `MEASURE 0 ro[0]` then `register_name` should be `"ro"`.
/// 2. `num_shots` is the number of times you'd like to run the program.
/// 3. `register_name`:
///
/// # Errors
/// This program will return a [`QVMResponse`] with a `status_code` corresponding to any errors that
/// occur. See [`QVMStatus`] for more details on possible errors.
///
/// # Example
///
/// ```c
/// #include <stdio.h>
/// #include "../libqcs.h"
///
/// char* BELL_STATE_PROGRAM =
///         "DECLARE ro BIT[2]\n"
///         "H 0\n"
///         "CNOT 0 1\n"
///         "MEASURE 0 ro[0]\n"
///         "MEASURE 1 ro[1]\n";
///
/// int main() {
///     uint8_t shots = 10;
///     QVMResponse response = run_program_on_qvm(BELL_STATE_PROGRAM, shots, "ro");
///
///     if (response.status_code != QVMStatus_Success) {
///         // Something went wrong running the program
///         return 1;
///     }
///
///     for (int shot = 0; shot < response.number_of_shots; shot++) {
///         int bit_0 = response.results_by_shot[shot][0];
///         int bit_1 = response.results_by_shot[shot][1];
///         // With this program, bit_0 should always equal bit_1
///     }
///
///     free_qvm_response(response);
///
///     return 0;
/// }
/// ```
#[no_mangle]
pub unsafe extern "C" fn run_program_on_qvm(
    program: *mut c_char,
    num_shots: c_ushort,
    register_name: *mut c_char,
) -> QVMResponse {
    match _run_program_on_qvm(program, num_shots, register_name) {
        Ok(data) => QVMResponse::from_data(data),
        Err(error) => QVMResponse::from(error),
    }
}

/// Implements the actual logic of [`run_program_on_qvm`] but with `?` support.
unsafe fn _run_program_on_qvm(
    program: *mut c_char,
    num_shots: c_ushort,
    register_name: *mut c_char,
) -> Result<Vec<Vec<u8>>> {
    // SAFETY: If program is not a valid null-terminated string, this is UB
    let program = CStr::from_ptr(program);
    // SAFETY: If register is not a valid null-terminated string, this is UB
    let register = CStr::from_ptr(register_name);
    let program = program
        .to_str()
        .wrap_err("Could not decode program as UTF-8")?;
    let register = register
        .to_str()
        .wrap_err("Could not decode register as UTF-8")?;
    let rt = tokio::runtime::Runtime::new().wrap_err("Failed to create tokio runtime")?;
    let fut = run_program(program, num_shots, register);
    rt.block_on(fut)
}

/// Frees the memory of a [`QVMResponse`] as allocated by [`run_program_on_qvm`]
///
/// # Safety
/// This function should only be called with the result of [`run_program_on_qvm`]
#[no_mangle]
pub unsafe extern "C" fn free_qvm_response(response: QVMResponse) {
    let rust_managed = response.into_api_response();
    drop(rust_managed);
}

/// The return value of [`run_program_on_qvm`].
///
/// # Safety
/// In order to properly free the memory allocated in this struct, call [`free_qvm_response`]
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
/// QVMResponse response = run_program_on_qvm(program, 3, "ro");
/// ```
/// If `status_code` is `Success` then `results_by_shot` will look something like:
///
/// ```
/// results_by_shot = [[0, 0], [0, 0], [0, 0]]
/// ```
///
/// where `results_by_shot[shot][bit]` can access the value of `ro[bit]` for a given `shot`.
#[repr(C)]
pub struct QVMResponse {
    /// A 2-D array of integers containing the measurements into register provided as
    /// `register_name`. There will be one value per declared space in the register per "shot"
    /// (run of the program).
    pub results_by_shot: *mut *mut c_uchar,
    /// The number of times the program ran (should be the same as the `num_shots` param to
    /// [`run_program_on_qvm`]. This is the outer dimension of `results_by_shot`.
    pub number_of_shots: c_ushort,
    /// How many bits were measured in the program in one shot. This is the inner dimension of
    /// `results_by_shot`.
    pub shot_length: c_ushort,
    /// If this string is populated, there was an error. The string contains a description of that
    /// error and `results_by_shot` is `NULL`. If this string is `NULL`, the other fields contain
    /// data.
    pub error: *mut c_char,
}

impl QVMResponse {
    fn from_data(data: Vec<Vec<u8>>) -> Self {
        // Shots was passed into QVM originally as a u16 so this is safe.
        #[allow(clippy::cast_possible_truncation)]
        let number_of_shots = data.len() as u16;

        // This one is a guess. If more than 2^16 slots in a register then this will truncate
        #[allow(clippy::cast_possible_truncation)]
        let shot_length = data[0].len() as u16;

        let mut results: Vec<*mut u8> = IntoIterator::into_iter(data)
            .map(|mut shot| {
                let ptr = shot.as_mut_ptr();
                std::mem::forget(shot);
                ptr
            })
            .collect();
        let ptr = results.as_mut_ptr();
        std::mem::forget(results);
        #[allow(clippy::cast_possible_truncation)]
        Self {
            results_by_shot: ptr,
            number_of_shots,
            shot_length,
            error: null_mut(),
        }
    }

    unsafe fn into_api_response(self) -> Result<qvm::QVMResponse> {
        let Self {
            results_by_shot,
            number_of_shots,
            shot_length,
            error,
        } = self;

        if results_by_shot.is_null() {
            if error.is_null() {
                return Err(eyre!("Unknown error"));
            }
            // SAFETY: If this was manually constructed with a null-terminated string, bad things
            // will happen here. Proper usage should only see an error message here that was
            // constructed from `QVMResponse::from`
            let c_string = CString::from_raw(error);
            return Err(eyre!(c_string.into_string()?));
        }

        // SAFETY: If any of these pieces are wrong, this will read arbitrary memory
        let results: Vec<*mut u8> = Vec::from_raw_parts(
            results_by_shot,
            number_of_shots as usize,
            number_of_shots as usize,
        );

        let results: Vec<Vec<u8>> = results
            .into_iter()
            // SAFETY: If any of these pieces are wrong, this will read arbitrary memory
            .map(|ptr| Vec::from_raw_parts(ptr, shot_length as usize, shot_length as usize))
            .collect();

        let mut registers = HashMap::with_capacity(1);
        registers.insert("ro".to_string(), results);

        Ok(qvm::QVMResponse { registers })
    }
}

impl From<Report> for QVMResponse {
    fn from(err: Report) -> Self {
        let c_string = CString::new(err.to_string()).expect("Rust strings aren't null!");
        let ptr = c_string.into_raw();
        Self {
            results_by_shot: null_mut(),
            number_of_shots: 0,
            shot_length: 0,
            error: ptr,
        }
    }
}
