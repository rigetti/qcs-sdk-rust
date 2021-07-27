use std::ffi::CStr;

use crate::ProgramResult;
use eyre::{Result, WrapErr};
use libc::{c_char, c_ushort};

/// Given a Quil program as a string, run that program on a local QVM.
///
/// # Safety
///
/// In order to run this function safely, you must provide the return value from this
/// function to [`crate::free_program_result`] once you're done with it. The inputs `program` and
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
/// 2. `num_shots`: the number of times you'd like to run the program.
/// 3. `register_name`: the name of the register in the `program` that is being measured to.
///
/// # Errors
///
/// This program will return a [`ProgramResult`] with a `error` attribute. That `error` attribute will
/// either be `NULL` if successful, or a human readable description of the error that occurred.
#[no_mangle]
pub unsafe extern "C" fn run_program_on_qvm(
    program: *mut c_char,
    num_shots: c_ushort,
    register_name: *mut c_char,
) -> ProgramResult {
    match _run_program_on_qvm(program, num_shots, register_name) {
        Ok(data) => ProgramResult::from_rust(data),
        Err(error) => ProgramResult::from(error),
    }
}

/// Implements the actual logic of [`run_program_on_qvm`] but with `?` support.
unsafe fn _run_program_on_qvm(
    program: *mut c_char,
    num_shots: c_ushort,
    register_name: *mut c_char,
) -> Result<qcs::ProgramResult> {
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
    let fut = qcs::qvm::run_program(program, num_shots, register);
    rt.block_on(fut)
}
