# ExecutionData

`ExecutionData` contains the actual results of an execution, you get a pointer to one by calling [`get_data`].

## Definition

```c
{{#include ../../../c-lib/libqcs.h:ExecutionData}}

{{#include ../../../c-lib/libqcs.h:DataType}}

{{#include ../../../c-lib/libqcs.h:DataType_Tag}}
```

## Safety

The memory for any `ExecutionData` will be freed when calling [`free_execution_result`] for the corresponding [`ExecutionResult`]. **Make sure not to free the [`ExecutionResult`] until after you're done with the data**.

## Attributes

1. `number_of_shots` is the outer dimension of the 2D array of data. This should always be equal to the parameter provided to [`wrap_in_shots`] (or 1 if not called).
2. `shot_length` is the inner dimension of the data array, corresponding to the dimension of the declared memory. For example, declaring `BIT` in Quil will result in a `shot_length` of 1, but declaring `BIT[2]` in Quil will result in a `shot_length` of 2.
3. `data` is a `DataType` which contains the actual results as measured from the requested register. This is a 2D array with outer dimension of `number_of_shots` and inner dimension of `shot_length`. The type of this data depends on the type of the declared memory. The `tag` field tells you which type of data is contained within, then `byte` or `real` is the 2D array.

## Variants

The type of `data` will depend on how the memory was declared in Quil.

### `Byte`

The result of reading from a `BIT` or `OCTET` register is the `Byte` variant. `data.tag` will be `DataType_Byte`, and `data.byte` will be populated.

### `Real`

The result of reading from a `REAL` register is the `Real` variant. `data.tag` will be `DataType_Real`, and `data.real` will be populated.

### Example

Here we declare both `REAL` and `OCTET` registers which will correspond to `Real` and `Byte` variants.

```c
{{#include ../../../c-lib/tests/integration_tests.c:test_real_data}}
```

[`get_data`]: get_data.md
[`free_execution_result`]: free_execution_result.md
[`ExecutionResult`]: execution_result.md
[`wrap_in_shots`]: wrap_in_shots.md
