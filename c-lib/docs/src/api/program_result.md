# `ProgramResult`

A `ProgramResult` is the struct you receive from a call to either [`run_program_on_qvm`] or [`run_program_on_qpu`]. This struct is implemented as a tagged union, describing the possible outcomes of program execution.

## SAFETY

**You must pass this struct to [`free_program_result`] when you're done with it to avoid leaking the memory.**


## Definition

```c
{{#include ../../../libqcs.h:ProgramResult}}
```

## Variants

There are multiple variants of `ProgramResult`. Which is in use is determined by the `tag` attribute which is of type `ProgramResult_Tag` as defined below.

```c
{{#include ../../../libqcs.h:ProgramResult_Tag}}
```

The [`Error`] variant is special in that it indicates a failed program execution. All other variants are used to differentiate between the _type_ of data that is returned.

### Common Fields

Except for [`Error`], all variants have these attributes in common:

1. `number_of_shots`: This should be equal to the `shots` parameter provided to [`run_program_on_qpu`] or [`run_program_on_qvm`]. It's included in this structure to keep track of the actual memory allocated in the underlying arrays. This attribute will always equal the outer dimension of `data_per_shot`.
2. `shot_length`: This equals the length of the memory vector that was read out. If you had `DECLARE ro BIT[2]` in your quil program, and you passed `"ro"` to the `register` param of [`run_program_on_qpu`] or [`run_program_on_qvm`], then `shot_length` would be 2 (the size of that register). This is the inner dimension of `data_per_shot`.
3. `data_per_shot`: This is a 2-Dimensional array containing the actual result of the program execution. There is one array per shot, each containing one entry per register slot.

#### Example

Here we see the example for the [`Real`] variant, but these three attributes have the same relationship in every non-[`Error`] variant.

```c
{{#include ../../../tests/integration_tests.c:real_shot_check}}
```

### `Error`

If something goes wrong, `ProgramResult.tag` will be `ProgramResult_Error`, indicating it is the `Error` variant. This variant is a human-readable string of the error that occurred.

#### `Error` Example

Here, `result.error` is that string:

```c
{{#include ../../../tests/integration_tests.c:run}}

{{#include ../../../tests/integration_tests.c:errors}}
```

#### `Error` Definition

```c
{{#include ../../../libqcs.h:error}}
```

### `Byte`

The result of reading from a `BIT` or `OCTET` register is the `Byte` variant. `ProgramResult.tag` will be `ProgramResult_Byte`, and the `byte` attribute will be populated.

#### `Byte` Example

```c
    {{#include ../../../tests/integration_tests.c:program}}

{{#include ../../../tests/integration_tests.c:run}}

{{#include ../../../tests/integration_tests.c:byte_check}}

{{#include ../../../tests/integration_tests.c:results}}
```

#### `Byte` Definition

```c
{{#include ../../../libqcs.h:ProgramResult_Byte_Body}}
```

### `Real`

The result of reading from a `REAL` register is the `Real` variant. `ProgramResult.tag` will be `ProgramResult_Real`, and the `real` attribute will be populated.

#### `Real` Example

```c
{{#include ../../../tests/integration_tests.c:test_real_data}}
```

#### `Real` Definition

```c
{{#include ../../../libqcs.h:ProgramResult_Real_Body}}
```


[`run_program_on_qvm`]: ./run_program_on_qvm.md
[`run_program_on_qpu`]: ./run_program_on_qpu.md
[`free_program_result`]: ./free_program_result.md
[`Error`]: #error
[`Real`]: #real
