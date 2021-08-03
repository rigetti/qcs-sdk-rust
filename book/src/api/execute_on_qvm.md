# `execute_on_qvm`

Run an [`Executable`] against a locally-running simulator.

## Definition

```c
{{#include ../../../c-lib/libqcs.h:execute_on_qvm}}
```

## Safety

1. You must provide the return value from this function to [`free_execution_result`] once you're done with it.

## Usage

In order to execute, QVM must be running at <http://localhost:5000> (unless you've specified a different endpoint in config).

## Arguments

1. `executable`: An [`Executable`]

## Errors

This program will return the [`Error`] variant of [`ExecutionResult`] with a human-readable description of the error. Some common errors:

1. QVM was not running or not reachable.
3. A syntax error in the provided Quil `program`.
4. There was no data to read (improper or missing [`read_from`] option).
5. The type of the register is not supported by the implemented [`Variants`].

[`Executable`]: executable.md
[`free_execution_result`]: free_execution_result.md
[`ExecutionResult`]: execution_result.md
[`Error`]: execution_result.md#error
[`Variants`]: execution_result.md#variants
[`read_from`]: read_from.md
