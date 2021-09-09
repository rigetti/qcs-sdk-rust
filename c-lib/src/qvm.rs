use eyre::{Result, WrapErr};

use crate::{Executable, ExecutionResult};
use std::collections::HashMap;

/// Given a Quil program as a string, run that program on a local QVM.
///
/// # Safety
///
/// 1. You must provide the return value from this function to [`crate::free_execution_result`] once
///    you're done with it.
/// 2. `executable` must be the valid, non-NULL result of [`crate::executable_from_quil`]
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
/// 2. `num_shots`: the number of times you'd like to run the program.
/// 3. `register_name`: the name of the register in the `program` that is being measured to.
///
/// # Errors
///
/// This program will return a [`crate::ExecutionResult::Error`] if an error occurs.
#[no_mangle]
pub unsafe extern "C" fn execute_on_qvm(executable: *mut Executable) -> ExecutionResult {
    match _execute_on_qvm(executable) {
        Ok(data) => ExecutionResult::from_data(data),
        Err(error) => ExecutionResult::from(error),
    }
}

/// Implements the actual logic of [`execute_on_qvm`] but with `?` support.
unsafe fn _execute_on_qvm(
    executable: *mut Executable,
) -> Result<HashMap<Box<str>, qcs::ExecutionResult>, String> {
    // SAFETY: If this wasn't constructed already, was already freed, or is NULL, bad things
    // happen here.
    let mut executable = Box::from_raw(executable);

    let result = match &mut executable.inner {
        Err(e) => Err(format!("{:?}", e)),
        Ok(inner) => {
            let rt = tokio::runtime::Runtime::new()
                .wrap_err("Failed to create tokio runtime")
                .map_err(|e| format!("{:?}", e))?;
            let fut = inner.execute_on_qvm();
            rt.block_on(fut).map_err(|e| format!("{:?}", e))
        }
    };

    Box::into_raw(executable);
    result
}
