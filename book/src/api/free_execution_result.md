# `free_execution_result`

`free_execution_result` is a memory deallocation function which must be called for any [`ExecutionResult`] to avoid leakage.

## Safety

**Only call this function with the result of [`execute_on_qpu`] or [`execute_on_qvm`] as it expects memory that was originally allocated by Rust.**

## Definition

```c
{{#include ../../../c-lib/libqcs.h:free_execution_result}}
```

[`ExecutionResult`]: execution_result.md
[`execute_on_qpu`]: execute_on_qpu.md
[`execute_on_qvm`]: execute_on_qvm.md
