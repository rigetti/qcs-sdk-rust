use std::ffi::CStr;
use std::ptr;

use libc::{c_char, c_uchar, c_uint};

use qvm::{run_program, QVMError};
use std::collections::HashMap;

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
    num_shots: c_uint,
    register_name: *mut c_char,
) -> QVMResponse {
    // SAFETY: If program is not a valid null-terminated string, this is UB
    let program = CStr::from_ptr(program);
    // SAFETY: If register is not a valid null-terminated string, this is UB
    let register = CStr::from_ptr(register_name);
    let program = match program.to_str() {
        Ok(program) => program,
        Err(std::str::Utf8Error { .. }) => {
            return QVMResponse::from_error(QVMStatus::ProgramIsNotUtf8)
        }
    };
    let register = match register.to_str() {
        Ok(register) => register,
        Err(std::str::Utf8Error { .. }) => {
            return QVMResponse::from_error(QVMStatus::RegisterIsNotUtf8)
        }
    };
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return QVMResponse::from_error(QVMStatus::CannotMakeRequest),
    };
    let fut = run_program(program, num_shots as usize, register);
    match rt.block_on(fut) {
        Ok(data) => QVMResponse::from_data(data),
        Err(error) => QVMResponse::from(error),
    }
}

/// Frees the memory of a [`QVMResponse`] as allocated by [`run_program_on_qvm`]
///
/// # Safety
/// This function should only be called with the result of [`run_program_on_qvm`]
#[no_mangle]
pub unsafe extern "C" fn free_qvm_response(response: QVMResponse) {
    let rust_managed: qvm::QVMResponse = response.into_api_response();
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
    pub number_of_shots: c_uint,
    /// How many bits were measured in the program in one shot. This is the inner dimension of
    /// `results_by_shot`.
    pub shot_length: c_uint,
    /// Tells you whether or not the request to the QVM was successful. If the status
    /// code is [`QVMStatus::Success`], then `results_by_shot` will be populated.
    /// If not, `results_by_shot` will be `NULL`.
    pub status_code: QVMStatus,
}

impl QVMResponse {
    fn from_error(status_code: QVMStatus) -> Self {
        Self {
            results_by_shot: ptr::null_mut(),
            number_of_shots: 0,
            shot_length: 0,
            status_code,
        }
    }

    fn from_data(data: Vec<Vec<u8>>) -> Self {
        let number_of_shots = data.len() as u32;
        let shot_length = data[0].len() as u32;
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
            status_code: QVMStatus::Success,
        }
    }

    unsafe fn into_api_response(self) -> qvm::QVMResponse {
        let Self {
            results_by_shot,
            number_of_shots,
            shot_length,
            status_code,
        } = self;

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

        drop(status_code);

        qvm::QVMResponse { registers }
    }
}

impl From<QVMStatus> for QVMResponse {
    fn from(status: QVMStatus) -> Self {
        Self {
            results_by_shot: ptr::null_mut(),
            number_of_shots: 0,
            shot_length: 0,
            status_code: status,
        }
    }
}

impl From<QVMError> for QVMResponse {
    fn from(error: QVMError) -> Self {
        let status = match error {
            QVMError::ShotsMismatch | QVMError::InconsistentShots => {
                QVMStatus::InconsistentShotLength
            }
            QVMError::RegisterMissing => QVMStatus::NoResults,
            QVMError::Connection(_) => QVMStatus::UnableToCommunicateWithQVM,
            QVMError::Configuration(_) => QVMStatus::ConfigError,
        };
        Self {
            results_by_shot: ptr::null_mut(),
            number_of_shots: 0,
            shot_length: 0,
            status_code: status,
        }
    }
}

/// Codes indicating the possible results of calling [`run_program_on_qvm`]. Every [`QVMResponse`]
/// will have one of these statuses in their `status_code` field. Note that in the generated C
/// headers, each variant will be prefixed with `QVMStatus` to prevent naming conflicts
/// (e.g. `QVMStatus_Success`).
#[repr(u8)]
pub enum QVMStatus {
    /// Program was run successfully, the [`QVMResponse`] containing this has valid data in other fields.
    Success = 0,
    /// The Program provided was not valid UTF-8 and could not be decoded for processing.
    ProgramIsNotUtf8 = 1,
    /// Something prevented this library from attempting to make the request, if this happens
    /// it's probably a bug.
    CannotMakeRequest = 2,
    /// QVM did not respond with a results in the specified register.
    NoResults = 3,
    /// One or more shots had differing numbers of result registers, this could be a bug with QVM.
    InconsistentShotLength = 5,
    /// A request to QVM was attempted but failed, is it running?
    UnableToCommunicateWithQVM = 6,
    /// The provided `register_name` was not valid UTF-8
    RegisterIsNotUtf8 = 7,
    /// Configuration could not be loaded, so QVM could not be contacted
    ConfigError = 8,
}
