use std::ffi::CStr;

use eyre::{Result, WrapErr};
use libc::{c_char, c_ushort};

use crate::ProgramResult;

/// Given a Quil program as a string, run that program on a QPU
///
/// # Safety
///
/// In order to run this function safely, you must provide the return value from this
/// function to [`crate::free_program_result`] once you're done with it. The inputs `program`,
/// `register_name`, and `qpu_id` must be valid, nul-terminated, non-null strings which remain
/// constant for the duration of this function.
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
/// 1. `program`: A string containing a valid Quil program. Any measurements that you'd like
/// to get back out must be in a register matching `register_name`. For example, if you have
/// `MEASURE 0 ro[0]` then `register_name` should be `"ro"`.
/// 2. `num_shots`: the number of times you'd like to run the program.
/// 3. `register_name`: the name of the register in the `program` that is being measured to.
/// 4. `qpu_id`: the ID of the QPU to run on (e.g. `"Aspen-9"`)
///
/// # Errors
///
/// This program will return a [`crate::ProgramResult`] with an `error` attribute which will be
/// `NULL` if successful or a human readable description of the error that occurred.
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
///     ProgramResult response = run_program_on_qpu(BELL_STATE_PROGRAM, shots, "ro", "Aspen-9");
///
///     if (response.error != NULL) {
///         printf("An error occurred when running the program:\n\t%s", response.error);
///         return 1;
///     }
///
///     for (int shot = 0; shot < response.number_of_shots; shot++) {
///         int bit_0 = response.results_by_shot[shot][0];
///         int bit_1 = response.results_by_shot[shot][1];
///         // With this program, bit_0 should always equal bit_1
///     }
///
///     free_qpu_response(response);
///
///     return 0;
/// }
/// ```
#[no_mangle]
pub unsafe extern "C" fn run_program_on_qpu(
    program: *mut c_char,
    num_shots: c_ushort,
    register_name: *mut c_char,
    qpu_id: *mut c_char,
) -> ProgramResult {
    match _run_program_on_qpu(program, num_shots, register_name, qpu_id) {
        Ok(data) => ProgramResult::from_rust(data),
        Err(error) => ProgramResult::from(error),
    }
}

/// Implements the actual logic of [`run_program_on_qpu`] but with `?` support.
unsafe fn _run_program_on_qpu(
    program: *mut c_char,
    num_shots: c_ushort,
    register_name: *mut c_char,
    qpu_id: *mut c_char,
) -> Result<qcs::ProgramResult> {
    // SAFETY: If program is not a valid null-terminated string, this is UB
    let program = CStr::from_ptr(program);
    let program = program
        .to_str()
        .wrap_err("Could not decode program as UTF-8")?;

    // SAFETY: If register is not a valid null-terminated string, this is UB
    let register = CStr::from_ptr(register_name);
    let register = register
        .to_str()
        .wrap_err("Could not decode register as UTF-8")?;

    // SAFETY: If qpu_id is not a valid null-terminated string, this is UB
    let qpu_id = CStr::from_ptr(qpu_id);
    let qpu_id = qpu_id
        .to_str()
        .wrap_err("Could not decode register as UTF-8")?;

    let rt = tokio::runtime::Runtime::new().wrap_err("Failed to create tokio runtime")?;
    let fut = qcs::qpu::run_program(program, num_shots, register, qpu_id);
    rt.block_on(fut)
}
