# read_from

Set the memory location to read out of for an [`Executable`]. If not set, the [`Executable`] assumes a default of "ro". You can call this function multiple times to read from multiple registers. The first time you call the function, the default of "ro" is not longer relevant.

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

We've declared a region called `first` and another called `second`—both of which we'd like to read out of. Since we are not using a single register called `"ro"` (the default), we need to specify where to read from.

```C
{{#include ../../../c-lib/tests/integration_tests.c:read_from}}
```

[`Executable`]: executable.md
[`executable_from_quil`]: executable_from_quil.md
