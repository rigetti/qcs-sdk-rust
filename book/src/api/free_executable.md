# `free_executable`

`free_executable` is a memory deallocation function which must be called for any [`Executable`] to avoid leakage.

## Safety

**Only call this function with the result of [`executable_from_quil`] as it expects memory that was originally allocated by Rust.**

## Definition

```c
{{#include ../../../c-lib/libqcs.h:free_executable}}
```

[`Executable`]: executable.md
[`executable_from_quil`]: executable_from_quil.md
[`execute_on_qvm`]: execute_on_qvm.md

