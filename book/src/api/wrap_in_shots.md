# wrap_in_shots

Set the [`Executable`] to run multiple times per execution on the QPU. If this option is not set, the [`Executable`] will be run one time per execution.

## Definition

```C
{{#include ../../../c-lib/libqcs.h:wrap_in_shots}}
```

## Arguments

1. `executable`: The [`Executable`] to set the parameter on.
2. `shots`: The number of times to run the executable for each execution.

## Safety

1. `executable` must be the result of [`executable_from_quil`]

## Example

Take a simple bell state program:

```C
    {{#include ../../../c-lib/tests/integration_tests.c:program}}
```

If we run the program like this:

```C
{{#include ../../../c-lib/tests/integration_tests.c:run}}
```

Then the program will be executed 3 times (per the number of shots set). The resulting [`Byte`] data of one execution will be a 2D array with an outer dimension of 3 (number of shots) and an inner dimension of 2 (amount of memory locations read per run). 

[`Executable`]: executable.md
[`executable_from_quil`]: executable_from_quil.md
[`execute_on_qpu`]: execute_on_qpu.md
[`Byte`]: execution_result.md#byte
