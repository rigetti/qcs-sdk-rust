# `execute_on_qpu`

Execute an [`Executable`] on a specified QPU.

## Definition
```c
{{#include ../../../c-lib/libqcs.h:execute_on_qpu}}
```

## Safety

1. In order to run this function safely, you must provide the return value from this function to [`free_execution_result`] once you're done with it. 
2. The input `qpu_id` must be a valid, nul-terminated, non-null string which remains constant for the duration of this function.

## Usage

In order to execute, you must have an active reservation for the QPU you're targeting as well as valid, configured QCS credentials.

### Configuration

Valid settings and secrets must be set either in ~/.qcs or by setting the OS environment variables `QCS_SECRETS_FILE_PATH` and `QCS_SETTINGS_FILE_PATH` for secrets and settings respectively. `QCS_PROFILE_NAME` can also be used to choose a different profile in those configuration files.

## Arguments

1. `executable`: The [`Executable`] to execute.
4. `qpu_id`: the ID of the QPU to run on (e.g. `"Aspen-9"`)

# Errors

This program will return the [`Error`] variant of [`ExecutionResult`] with a human-readable description of the error. Some common errors:

1. QCS could not be authenticated with due to missing / invalid [configuration].
2. Authenticated user has no active reservation for the requested `qpu_id`.
3. A syntax error in the provided [Quil] when calling [`executable_from_quil`].
4. There was no data to read (improper or missing [`read_from`] option).

[`Executable`]: executable.md
[`free_execution_result`]: free_execution_result.md
[`ExecutionResult`]: execution_result.md
[`Error`]: execution_result.md#error
[configuration]: #configuration
[`executable_from_quil`]: executable_from_quil.md
[Quil]: https://github.com/quil-lang/quil
[`read_from`]: read_from.md
