# Getting Started

## Basics

The point of `libqcs` is to allow you to run a [quil] program on [QCS]. This program will be a string that you pass to either [`run_program_on_qvm`] (for simulating results) or [`run_program_on_qpu`] (for running against a real Quantum Computer).

In addition to the program itself, you'll need to specify how many "shots" the program should be run for. This lets you run an experiment multiple times _much_ quicker than looping over calls to the function manually.

You also must indicate which "register" to read results out of. This string must match a `DECLARE` statement in your program.

In the case of `run_program_on_qpu` you must also specify the name of the QPU to run on.

## An Example

Let's walk through an example by reviewing some code used to test this library:

### Step 1: Include `libqcs.h`

```c
{{#include ../../tests/integration_tests.c:include}}
```

### Step 2: Define a Quil Program

In this case we have a constant program, but this could just as easily be dynamically written at runtime.

```c
{{#include ../../tests/integration_tests.c:program}}
```

### Step 3: Run the Program

Here we run the program 3 times (shots) on a QVM (simulated quantum computer). We measured to memory called "ro" in the program, so that's where we tell QCS to read the results from.

```c
{{#include ../../tests/integration_tests.c:run}}
```

If we want to run on a real QPU, we swap out the function and add a parameter specifying which QPU to run against:

```c
    ProgramResult result = run_program_on_qpu(BELL_STATE_PROGRAM, shots, "ro", "Aspen-9");
```

### Step 4: Handle Errors

If something goes wrong [`ProgramResult`] will be the [`Error`] variant. This field contains a human-readable description of the error.

```c
{{#include ../../tests/integration_tests.c:errors}}
```

For the sake of our test cases, we have  defined `fail` function which frees the memory of `result` for us. **Make sure to always free a [`ProgramResult`] using [`free_program_result`]**. 

### Step 5: Process Results

If there were no errors, you can safely read your results out of `results_by_shot`! In this case, we know a successful response will be a [`Byte`] variant since we read out of `BIT` memory.

```c
{{#include ../../tests/integration_tests.c:results}}
```

It's a 2D array of bytes. There is an array representing the requested register per shot. In this case, there are 2 bits to read and three shots, so the data looks something like:

```c
[[0, 0],[1, 1],[0, 0]]
```

### Step 6: Free the Memory

[`ProgramResult`] was allocated for you, so you must call [`free_program_result`] to deallocated it safely when you're done if you don't want to leak that memory:

```c
{{#include ../../tests/integration_tests.c:free}}
```

### All Together

Here's what the full integration test looks like from our test suite:

```c
{{#include ../../tests/integration_tests.c:all}}
```

[quil]: https://github.com/quil-lang/quil
[qcs]: https://docs.rigetti.com/qcs/
[`ProgramResult`]: ./api/program_result.md
[`Byte`]: ./api/program_result.md#byte
[`Error`]: ./api/program_result.md#error
[`free_program_result`]: ./api/free_program_result.md
[`run_program_on_qvm`]: ./api/run_program_on_qvm.md
[`run_program_on_qpu`]: ./api/run_program_on_qpu.md