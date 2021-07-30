# `free_program_result`

`free_program_result` is a memory deallocation function which must be called for any [`ProgramResult`] to avoid leakage.

## Safety

**Only call this function with the result of [`run_program_on_qpu`] or [`run_program_on_qvm`] as it expects memory that was originally allocated by Rust.**

## Definition

```c
{{#include ../../../libqcs.h:free_program_result}}
```

[`ProgramResult`]: ./program_result.md
[`run_program_on_qpu`]: ./run_program_on_qpu.md
[`run_program_on_qvm`]: ./run_program_on_qvm.md
