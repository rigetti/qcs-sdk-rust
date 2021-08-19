# set_param

Set the value of a parameter on an [`Executable`] for parametric execution.

## Definition

```C
{{#include ../../../c-lib/libqcs.h:set_param}}
```

## Arguments

1. `executable`: The [`Executable`] to set the parameter on.
2. `name`: The name of the memory region to set a value for.
3. `index`: The index into the memory region where the value should be set.
4. `value`: The value to set for `name[index]`.

## Safety

1. `executable` must be the result of [`executable_from_quil`]
2. `name` must be a valid, non-NULL, nul-terminated string. It must also live until `executable`is freed.

## Example

> Adapted from [pyQuil](https://pyquil-docs.rigetti.com/en/stable/basics.html#parametric-compilation)

With a program like this one:

```C
    {{#include ../../../c-lib/tests/integration_tests.c:parametrized_program}}
```

We've declared a region called `theta` that is used as a parameter. This parameter must be injected for any executions. You might have a loop which looks like this, where you execute on multiple different parameters:

```C
{{#include ../../../c-lib/tests/integration_tests.c:set_param}}
        
        // Do things with result...

{{#include ../../../c-lib/tests/integration_tests.c:free_execution_result}}
```

In that example, `exe` is an [`Executable`] which would have been previously allocated using [`executable_from_quil`].

Make sure to free each [`ExecutionResult`] in the loop to avoid leaking memory!

> If `theta` was a larger vector (e.g. `DECLARE theta REAL[2]`) then you would set the other indexes like `set_param(exe, "theta", index, another_value)`.

[`Executable`]: executable.md
[`executable_from_quil`]: executable_from_quil.md
[`ExecutionResult`]: execution_result.md