# `ExecutionResult`

An `ExecutionResult` is the struct you receive from a call to either [`execute_on_qvm`] or [`execute_on_qpu`]. This struct is implemented as a tagged union, describing the possible outcomes of program execution.

## SAFETY

**You must pass this struct to [`free_execution_result`] when you're done with it to avoid leaking the memory.**

**Any [`ExecutionData`] which was retrieved from this result will be freed when this is called.**


## Definition

```c
{{#include ../../../c-lib/libqcs.h:ExecutionResult}}
```

## Variants

There are multiple variants of `ExecutionResult`. The `tag` attribute determines which is in use via an enum:

```c
{{#include ../../../c-lib/libqcs.h:ExecutionResult_Tag}}
```

The [`Error`] variant indicates that execution failed. The [`Handle`] variant is populated in the case of a successful run, and provides access to a [`ResultHandle`].


### `Error`

If something goes wrong, `tag` will be `ExecutionResult_Error`, indicating it is the `Error` variant. This variant is a human-readable string of the error that occurred.

#### `Error` Example

Here, `result.error` is that string:

```c
{{#include ../../../c-lib/tests/integration_tests.c:run}}

{{#include ../../../c-lib/tests/integration_tests.c:errors}}
```

### `Handle`

If there is not an error, `tag` will instead be `ExecutionResult_Handle`. The `handle` attribute is a pointer to a [`ResultHandle`] which can be used with [`get_data`].

#### `Handle` Example

```c
{{#include ../../../c-lib/tests/integration_tests.c:get_data}}
```


[`execute_on_qvm`]: execute_on_qvm.md
[`execute_on_qpu`]: execute_on_qpu.md
[`free_execution_result`]: free_execution_result.md
[`Error`]: #error
[`Handle`]: #handle
[`wrap_in_shots`]: wrap_in_shots.md
[`read_from`]: read_from.md
[`ResultHandle`]: result_handle.md
[`get_data`]: get_data.md
[`ExecutionData`]: execution_data.md
