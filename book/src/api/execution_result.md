# `ExecutionResult`

An `ExecutionResult` is the struct you receive from a call to either [`execute_on_qvm`] or [`execute_on_qpu`]. This struct is implemented as a tagged union, describing the possible outcomes of program execution.

## SAFETY

**You must pass this struct to [`free_execution_result`] when you're done with it to avoid leaking the memory.**


## Definition

```c
{{#include ../../../c-lib/libqcs.h:ExecutionResult}}
```

## Variants

There are multiple variants of `ExecutionResult`. The `tag` attribute determines which is in use via an enum:

```c
{{#include ../../../c-lib/libqcs.h:ExecutionResult_Tag}}
```

The [`Error`] variant is special in that it indicates a failed program execution. All other variants are used to differentiate between the _type_ of data that is returned.

### Common Fields

Except for [`Error`], all variants have these attributes in common:

1. `number_of_shots`: This should be equal to the `shots` parameter provided to [`wrap_in_shots`], or 1 if no shots were specified. It's included in this structure to keep track of the actual memory allocated in the underlying arrays. This attribute will always equal the outer dimension of `data_per_shot`.
2. `shot_length`: This equals the length of the memory vector that was read out. If you had `DECLARE ro BIT[2]` in your quil program, then `shot_length` would be 2 (the size of that register). This is the inner dimension of `data_per_shot`. Note that "ro" is the default which can be overridden by calling [`read_from`].
3. `data_per_shot`: This is a 2-Dimensional array containing the actual result of the execution. There is one array per shot, each containing one entry per register slot.

#### Example

Here we see the example for the [`Real`] variant, but these three attributes have the same relationship in every non-[`Error`] variant.

```c
{{#include ../../../c-lib/tests/integration_tests.c:real_shot_check}}
```

### `Error`

If something goes wrong, `tag` will be `ExecutionResult_Error`, indicating it is the `Error` variant. This variant is a human-readable string of the error that occurred.

#### `Error` Example

Here, `result.error` is that string:

```c
{{#include ../../../c-lib/tests/integration_tests.c:run}}

{{#include ../../../c-lib/tests/integration_tests.c:errors}}
```

#### `Error` Definition

```c
{{#include ../../../c-lib/libqcs.h:error}}
```

### `Byte`

The result of reading from a `BIT` or `OCTET` register is the `Byte` variant. `tag` will be `ExecutionResult_Byte`, and the `byte` attribute will be populated.

#### `Byte` Example

```c
    {{#include ../../../c-lib/tests/integration_tests.c:program}}

{{#include ../../../c-lib/tests/integration_tests.c:run}}

{{#include ../../../c-lib/tests/integration_tests.c:byte_check}}

{{#include ../../../c-lib/tests/integration_tests.c:results}}
```

#### `Byte` Definition

```c
{{#include ../../../c-lib/libqcs.h:ExecutionResult_Byte_Body}}
```

### `Real`

The result of reading from a `REAL` register is the `Real` variant. `tag` will be `ExecutionResult_Real`, and the `real` attribute will be populated.

#### `Real` Example

```c
{{#include ../../../c-lib/tests/integration_tests.c:test_real_data}}
```

#### `Real` Definition

```c
{{#include ../../../c-lib/libqcs.h:ExecutionResult_Real_Body}}
```


[`execute_on_qvm`]: execute_on_qvm.md
[`execute_on_qpu`]: execute_on_qpu.md
[`free_execution_result`]: free_execution_result.md
[`Error`]: #error
[`Real`]: #real
[`wrap_in_shots`]: wrap_in_shots.md
[`read_from`]: read_from.md
