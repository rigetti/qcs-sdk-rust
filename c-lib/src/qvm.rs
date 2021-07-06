use std::ffi::CStr;
use std::ptr;

use libc::{c_char, c_uchar, c_uint};

use std::collections::HashMap;

/// Given a Quil program as a string, run that program on a local QVM.
///
/// # Safety
///
/// In order to run this function safely, you must provide the return value from this
/// function to [`free_qvm_response`] once you're done with it. The input `program` must be a
/// valid, null-terminated, non-null string which remains constant for the duration of this function.
///
/// # Usage
///
/// In order to execute, QVM must be running at <http://localhost:5000>. The provided program
/// is expected to measure any results into a register called "ro". If this register is missing,
/// there will be an error.
///
/// # Parameters
///
/// 1. `program` should be a string containing a valid Quil program. Any measurements that you'd like
/// to get back out __must be put in a register called "ro"__ (e.g. `DECLARE ro BIT[2]`).
/// 2. `num_shots` is the number of times you'd like to run the program.
///
/// # Errors
/// This program will return a [`QVMResponse`] with a `status_code` corresponding to any errors that
/// occur. See [`QVMStatus`] for more details on possible errors.
#[no_mangle]
pub unsafe extern "C" fn run_program_on_qvm(
    program: *mut c_char,
    num_shots: c_uint,
) -> QVMResponse {
    // SAFETY: If program is not a valid null-terminated string, this is UB
    let program = CStr::from_ptr(program);
    let program = match program.to_str() {
        Ok(program) => program,
        Err(std::str::Utf8Error { .. }) => {
            return QVMResponse::from_error(QVMStatus::ProgramIsNotUtf8)
        }
    };
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return QVMResponse::from_error(QVMStatus::CannotMakeRequest),
    };
    let fut = qvm_api::run_program_on_qvm(program, num_shots);
    match rt.block_on(fut) {
        Ok(response) => QVMResponse::from(response),
        Err(error) => QVMResponse::from(error),
    }
}

/// Frees the memory of a [`QVMResponse`] as allocated by [`run_program_on_qvm`]
///
/// # Safety
/// This function should only be called with the result of [`run_program_on_qvm`]
#[no_mangle]
pub unsafe extern "C" fn free_qvm_response(response: QVMResponse) {
    let rust_managed: qvm_api::QVMResponse = response.into_api_response();
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
/// QVMResponse response = run_program_on_qvm(program, 3);
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
    /// A 2-D array of integers containing the measurements into the "ro" memory.
    /// There will be one value per declared space in "ro" per "shot" (run of the program).
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

    unsafe fn into_api_response(self) -> qvm_api::QVMResponse {
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

        qvm_api::QVMResponse { registers }
    }
}

impl From<qvm_api::QVMResponse> for QVMResponse {
    fn from(mut response: qvm_api::QVMResponse) -> Self {
        let mut results = match response.registers.remove("ro") {
            Some(results) => results,
            None => {
                return QVMResponse::from_error(QVMStatus::NoRORegister);
            }
        };
        let outer_len = results.len();
        if outer_len == 0 {
            return QVMResponse::from_error(QVMStatus::NoResultsInRORegister);
        }
        let inner_len = results[0].len();
        for shot in &results {
            if shot.len() != inner_len {
                return QVMResponse::from_error(QVMStatus::InconsistentShotLength);
            }
        }

        results.shrink_to_fit();

        let mut results: Vec<*mut u8> = results
            .into_iter()
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
        Self {
            results_by_shot: ptr,
            number_of_shots: outer_len as u32,
            shot_length: inner_len as u32,
            status_code: QVMStatus::Success,
        }
    }
}

impl From<qvm_api::QVMError> for QVMResponse {
    fn from(mut _error: qvm_api::QVMError) -> Self {
        Self {
            results_by_shot: ptr::null_mut(),
            number_of_shots: 0,
            shot_length: 0,
            status_code: QVMStatus::UnableToCommunicateWithQVM,
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
    /// QVM did not respond with a result register called "ro", make sure one was declared in your
    /// program.
    NoRORegister = 3,
    /// QVM returned an "ro" register but it was empty.
    NoResultsInRORegister = 4,
    /// One or more shots had differing numbers of result registers, this could be a bug with QVM.
    InconsistentShotLength = 5,
    /// A request to QVM was attempted but failed, is it running?
    UnableToCommunicateWithQVM = 6,
}
