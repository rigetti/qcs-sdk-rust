# read_from

Set the memory location to read out of for an [`Executable`]. If not set, the [`Executable`] assumes a default of "ro".

## Definition

```C
{{#include ../../../c-lib/libqcs.h:read_from}}
```

## Arguments

1. `executable`: The [`Executable`] to set the parameter on.
2. `name`: The name of the memory region to read out of. Must match a Quil `DECLARE` statement exactly.

## Safety

1. `executable` must be the result of [`executable_from_quil`]
2. `name` must be a valid, non-NULL, nul-terminated string. It must also live until `executable`is freed.

## Example

With a program like this one:

```C
    {{#include ../../../c-lib/tests/integration_tests.c:real_memory_program}}
```

We've declared a region called `mem` that we'd like to read out of. Since it is not called "ro" (the default), we need to specify that that's where results should be collected from.

```C
{{#include ../../../c-lib/tests/integration_tests.c:read_from}}
```

[`Executable`]: executable.md
[`executable_from_quil`]: executable_from_quil.md
