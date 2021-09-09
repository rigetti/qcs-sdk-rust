use std::ffi::CStr;

use eyre::{Result, WrapErr};
use std::os::raw::c_char;

use crate::{Executable, ExecutionResult};
use std::collections::HashMap;

/// Run an executable (created by [`crate::executable_from_quil`]) on a real QPU.
///
/// # Safety
///
/// 1. You must provide the return value from this function to [`crate::free_execution_result`]
///     once you're done with it.
/// 2. The input `qpu_id` must be valid, nul-terminated, non-null strings which remain constant for
///     the duration of this function.
/// 3. `executable` must be the non-NULL result of a call to [`crate::executable_from_quil`] and
///     must not be freed during the execution of this function.
///
/// # Usage
///
/// In order to execute, you must have an active reservation for the QPU you're targeting.
///
/// ## Configuration
///
/// Valid settings and secrets must be set either in ~/.qcs or by setting the OS environment
/// variables `QCS_SECRETS_FILE_PATH` and `QCS_SETTINGS_FILE_PATH` for secrets and settings
/// respectively. `QCS_PROFILE_NAME` can also be used to choose a different profile in those
/// configuration files.
///
/// # Arguments
///
/// 1. `executable`: the result of a call to [`crate::executable_from_quil`]
/// 2. `qpu_id`: the ID of the QPU to run on (e.g. `"Aspen-9"`)
///
/// # Errors
///
/// This program will return a [`crate::ExecutionResult::Error`] if an error occurs.
#[no_mangle]
pub unsafe extern "C" fn execute_on_qpu(
    executable: *mut Executable,
    qpu_id: *mut c_char,
) -> ExecutionResult {
    match _execute_on_qpu(executable, qpu_id) {
        Ok(data) => ExecutionResult::from_data(data),
        Err(error) => ExecutionResult::from(error),
    }
}

/// Implements the actual logic of [`execute_on_qpu`] but with `?` support.
unsafe fn _execute_on_qpu(
    executable: *mut Executable,
    qpu_id: *mut c_char,
) -> Result<HashMap<Box<str>, qcs::ExecutionResult>, String> {
    // SAFETY: If qpu_id is not a valid null-terminated string, this is UB
    let qpu_id = CStr::from_ptr(qpu_id);
    let qpu_id = qpu_id
        .to_str()
        .map_err(|_| String::from("Could not decode register as UTF-8"))?;

    // SAFETY: If this wasn't constructed already, was already freed, or is NULL, bad things
    // happen here.
    let mut executable = Box::from_raw(executable);

    let result = match &mut executable.inner {
        Err(e) => Err(format!("{:?}", e)),
        Ok(inner) => {
            let rt = tokio::runtime::Runtime::new()
                .wrap_err("Failed to create tokio runtime")
                .map_err(|e| format!("{:?}", e))?;
            let fut = inner.execute_on_qpu(qpu_id);
            rt.block_on(fut).map_err(|e| format!("{:?}", e))
        }
    };

    Box::into_raw(executable);
    result
}
