# `run_program_on_qpu`

Given a Quil program as a string, run that program on a QPU. Returns a [`ProgramResult`].

## Definition
```c
{{#include ../../../libqcs.h:run_program_on_qpu}}
```

## Safety

1. In order to run this function safely, you must provide the return value from this function to [`free_program_result`] once you're done with it. 
2. The inputs `program`,`register_name`, and `qpu_id` must be valid, nul-terminated, non-null strings which remain constant for the duration of this function.

## Usage

In order to execute, you must have an active reservation for the QPU you're targeting.

### Configuration

Valid settings and secrets must be set either in ~/.qcs or by setting the OS environment variables `QCS_SECRETS_FILE_PATH` and `QCS_SETTINGS_FILE_PATH` for secrets and settings respectively. `QCS_PROFILE_NAME` can also be used to choose a different profile in those configuration files.

## Arguments

1. `program`: A string containing a valid Quil program. Any measurements that you'd like to get back out must be in a register matching `register_name`. For example, if you have `MEASURE 0 ro[0]` then `register_name` should be `"ro"`.
2. `num_shots`: the number of times you'd like to run the program.
3. `register_name`: the name of the register in the `program` that is being measured to.
4. `qpu_id`: the ID of the QPU to run on (e.g. `"Aspen-9"`)

# Errors

This program will return the [`Error`] variant of [`ProgramResult`] with a human readable description of the error. Some common errors:

1. QCS could not be authenticated with due to missing / invalid [configuration].
2. Authenticated user has no active reservation for the requested `qpu_id`.
3. A syntax error in the provided Quil `program`.
4. The provided `register_name` is not a valid memory register in the provided `program`.
5. The type of the register is not supported by the implemented [`Variants`].

[`free_program_result`]: ./free_program_result.md
[`ProgramResult`]: ./program_result.md
[`Error`]: ./program_result.md#error
[`Variants`]: ./program_result.md#variants
[configuration]: #configuration
