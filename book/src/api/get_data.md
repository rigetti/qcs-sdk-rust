# get_data

`get_data` is how you retrieve the actual results (in an [`ExecutionData`]) from the [`ResultHandle`] within the [`Handle`] variant of an [`ExecutionResult`]. The register `name` provided to this function should match one that you provided to [`read_from`] (or `"ro"` if left at the default).

## Definition

```c
{{#include ../../../c-lib/libqcs.h:get_data}}
```

## Safety

All inputs must be non-null. `name` must be a nul-terminated string. `handle` must be the result of a non-error call to [`execute_on_qvm`] or [`execute_on_qpu`]. If there are no results matching the provided `name` then the return value will be `NULL`, make sure to check this return value.

## Arguments

1. `handle` is the `handle` attribute of [`ExecutionResult`] when it is not an error.
2. `name` is the name of a register you want the data from. It should correspond to a call to [`read_from`] and a Quil `DECLARE` instruction.

## Returns

An [`ExecutionData`] if there was data for the requested register, or `NULL` if not.

## Example

```c
{{#include ../../../c-lib/tests/integration_tests.c:read_from}}

{{#include ../../../c-lib/tests/integration_tests.c:get_multiple}}
```

[`ExecutionData`]: execution_data.md
[`ResultHandle`]: result_handle.md
[`Handle`]: execution_result.md#handle
[`ExecutionResult`]: execution_result.md
[`read_from`]: read_from.md
[`execute_on_qvm`]: execute_on_qvm.md
[`execute_on_qpu`]: execute_on_qpu.md
