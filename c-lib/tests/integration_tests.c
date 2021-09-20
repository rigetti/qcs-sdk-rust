// ANCHOR: all
#include <math.h>
#include <stdbool.h>
#include <stdio.h>
#include <string.h>
// ANCHOR: include
#include "../libqcs.h"
// ANCHOR_END: include

// ANCHOR: program
char *BELL_STATE_PROGRAM =
        "DECLARE ro BIT[2]\n"
        "H 0\n"
        "CNOT 0 1\n"
        "MEASURE 0 ro[0]\n"
        "MEASURE 1 ro[1]\n";
// ANCHOR_END: program

bool fail(
        const char *testName,
        char *message,
        Executable *exe,
        ExecutionResult *executionResult
) {
    printf("❌ %s failed: %s\n", testName, message);
    free_executable(exe);
    if (executionResult != NULL) {
        free_execution_result(*executionResult);
    }

    return false;
}

bool succeed(const char *testName, Executable *exe, ExecutionResult *executionResult) {
    printf("✅ %s succeeded.\n", testName);
    // ANCHOR: free
    free_executable(exe);
    if (executionResult != NULL) {
        free_execution_result(*executionResult);
    }
    // ANCHOR_END: free
    return true;
}

bool test_bell_state() {
    const char *TEST_NAME = "test_bell_state";

    // ANCHOR: run
    unsigned int shots = 3;
    Executable *exe = executable_from_quil(BELL_STATE_PROGRAM);
    wrap_in_shots(exe, shots);
    ExecutionResult result = execute_on_qvm(exe);
    // ANCHOR_END: run

    // ANCHOR: errors
    if (result.tag == ExecutionResult_Error) {
        return fail(
                TEST_NAME,
                result.error,
                exe,
                &result
        );
    }
    // ANCHOR_END: errors

    //ANCHOR: get_data
    const ExecutionData *ro = get_data(result.handle, "ro");
    if (ro == NULL) {
        return fail(
                TEST_NAME,
                "ro register was not in result",
                exe,
                &result
        );
    }
    // ANCHOR_END: get_data

    // ANCHOR: byte_check
    if (ro->data.tag != DataType_Byte) {
        char message[50];
        sprintf(message, "Expected type Byte, got tag  %d", ro->data.tag);
        return fail(
                TEST_NAME,
                message,
                exe,
                &result
        );
    }
    // ANCHOR_END: byte_check

    if (ro->number_of_shots != shots) {
        char message[50];
        sprintf(message, "Response number of shots was %d, expected %d", ro->number_of_shots, shots);
        return fail(
                TEST_NAME,
                message,
                exe,
                &result
        );
    }

    if (ro->shot_length != 2) {
        char message[50];
        sprintf(message, "expected shot_length of 2, got %d", ro->shot_length);
        return fail(
                TEST_NAME,
                message,
                exe,
                &result
        );
    }

    // ANCHOR: results
    for (int shot = 0; shot < ro->number_of_shots; shot++) {
        // In our case, we measured two entangled qubits, so we expect their values to be equal.
        int bit_0 = ro->data.byte[shot][0];
        int bit_1 = ro->data.byte[shot][0];
        if (bit_0 != bit_1) {
            char message[50];
            sprintf(
                    message,
                    "in shot %d, got |%d%d",
                    shot,
                    bit_0,
                    bit_1
            );
            return fail(
                    TEST_NAME,
                    message,
                    exe,
                    &result
            );
        }
    }
    // ANCHOR_END: results

    return succeed(TEST_NAME, exe, &result);
}
// ANCHOR_END: all

char *PROGRAM_WITHOUT_MEASUREMENT = "X 0";

bool test_error() {
    const char *TEST_NAME = "test_error";

    Executable *exe = executable_from_quil(PROGRAM_WITHOUT_MEASUREMENT);
    ExecutionResult result = execute_on_qvm(exe);

    if (result.tag != ExecutionResult_Error) {
        return fail(
                TEST_NAME,
                "did not receive error result.",
                exe,
                &result
        );
    }

    return succeed(TEST_NAME, exe, &result);
}

// ANCHOR: test_real_data
// ANCHOR: real_memory_program
char *REAL_MEMORY_PROGRAM =
        "DECLARE first REAL[1]\n"
        "DECLARE second OCTET[1]\n"
        "MOVE first[0] 3.141\n"
        "MOVE second[0] 2\n";
// ANCHOR_END: real_memory_program

bool test_real_data_type() {
    const char *TEST_NAME = "test_real_data_type";

    // ANCHOR: read_from
    Executable *exe = executable_from_quil(REAL_MEMORY_PROGRAM);
    read_from(exe, "first");
    read_from(exe, "second");
    ExecutionResult result = execute_on_qvm(exe);
    // ANCHOR_END: read_from

    if (result.tag == ExecutionResult_Error) {
        return fail(
                TEST_NAME,
                result.error,
                exe,
                &result
        );
    }

    // ANCHOR: get_multiple
    const ExecutionData *first = get_data(result.handle, "first");
    const ExecutionData *second = get_data(result.handle, "second");
    // ANCHOR_END: get_multiple

    if (first == NULL || first->data.tag != DataType_Real) {
        return fail(
                TEST_NAME,
                "first register did not contain real data",
                exe,
                &result
        );
    }
    if (second == NULL || second->data.tag != DataType_Byte) {
        return fail(
                TEST_NAME,
                "second register did not contain byte data",
                exe,
                &result
        );
    }

    if (first->data.real[0][0] != 3.141) {
        char message[50];
        sprintf(
                message,
                "Found %f in first, expected 3.141",
                first->data.real[0][0]
        );
        return fail(
                TEST_NAME,
                message,
                exe,
                &result
        );
    }
    if (second->data.byte[0][0] != 2) {
        char message[50];
        sprintf(
                message,
                "Found %d in first, expected 2",
                second->data.byte[0][0]
        );
        return fail(
                TEST_NAME,
                message,
                exe,
                &result
        );
    }

    return succeed(TEST_NAME, exe, &result);
}
// ANCHOR_END: test_real_data

bool test_read_from_nonexistent_register() {
    const char *TEST_NAME = "test_read_from_nonexistent_register";

    Executable *exe = executable_from_quil(REAL_MEMORY_PROGRAM);
    read_from(exe, "nonexistent");
    ExecutionResult result = execute_on_qvm(exe);

    if (result.tag != ExecutionResult_Error) {
        return fail(
                TEST_NAME,
                "expected an error but did not receive one",
                exe,
                &result
        );
    }
    return succeed(TEST_NAME, exe, &result);
}

bool test_get_data_from_nonexistent_register() {
    const char *TEST_NAME = "test_get_data_from_nonexistent_register";
    Executable *exe = executable_from_quil(REAL_MEMORY_PROGRAM);
    read_from(exe, "first");
    ExecutionResult result = execute_on_qvm(exe);

    if (result.tag == ExecutionResult_Error) {
        return fail(
                TEST_NAME,
                result.error,
                exe,
                &result
        );
    }

    const ExecutionData *nonexistent = get_data(result.handle, "nonexistent");

    if (nonexistent != NULL) {
        return fail(
                TEST_NAME,
                "expected NULL when reading a register that doesn't exist",
                exe,
                &result
        );
    }
    return succeed(TEST_NAME, exe, &result);
}

// ANCHOR: parametrization
// ANCHOR: parametrized_program
char *PARAMETRIZED_PROGRAM =
        "DECLARE ro BIT\n"
        "DECLARE theta REAL\n"

        "RX(pi / 2) 0\n"
        "RZ(theta) 0\n"
        "RX(-pi / 2) 0\n"

        "MEASURE 0 ro[0]\n";
// ANCHOR_END: parametrized_program

bool test_parametrization() {
    const char *TEST_NAME = "test_parametrization";

    Executable *exe = executable_from_quil(PARAMETRIZED_PROGRAM);
    int STEPS = 200;
    double step_size = M_2_PI / STEPS;
    double theta;
    bool found_one = false;

    // ANCHOR: set_param
    for (int step = 0; step < STEPS; step++) {
        theta = step * step_size;
        set_param(exe, "theta", 0, theta);

        ExecutionResult result = execute_on_qvm(exe);
    // ANCHOR_END: set_param

        if (result.tag == ExecutionResult_Error) {
            return fail(
                    TEST_NAME,
                    result.error,
                    exe,
                    &result
            );
        }
        const ExecutionData *ro = get_data(result.handle, "ro");
        found_one |= ro->data.byte[0][0];
        // Free intermediate results
        // ANCHOR: free_execution_result
        free_execution_result(result);
    }
    // ANCHOR_END: free_execution_result

    if (found_one) {
        return succeed(TEST_NAME, exe, NULL);
    } else {
        return fail(TEST_NAME, "Got all zeroes, must not have parametrized", exe, NULL);
    }
}
// ANCHOR_END: parametrization

bool test_param_does_not_exist() {
    const char *TEST_NAME = "test_param_does_not_exist";

    Executable *exe = executable_from_quil(PARAMETRIZED_PROGRAM);
    set_param(exe, "doesnt_exist", 0, 0.0);
    ExecutionResult result = execute_on_qvm(exe);

    if (result.tag == ExecutionResult_Error) {
        return succeed(
                TEST_NAME,
                exe,
                &result
        );
    } else {
        return fail(TEST_NAME, "Expected an error, got none", exe, &result);
    }
}

bool test_param_wrong_size() {
    const char *TEST_NAME = "test_param_wrong_size";

    Executable *exe = executable_from_quil(PARAMETRIZED_PROGRAM);
    set_param(exe, "theta", 1, 0.0);
    ExecutionResult result = execute_on_qvm(exe);
    if (result.tag == ExecutionResult_Error) {
        return succeed(
                TEST_NAME,
                exe,
                &result
        );
    } else {
        return fail(TEST_NAME, "Expected an error, got none", exe, &result);
    }
}

int main() {
    bool failing = false;

    typedef bool (*test_func)(void);

    static test_func tests[] = {
            test_bell_state,
            test_error,
            test_real_data_type,
            test_parametrization,
            test_param_does_not_exist,
            test_param_wrong_size,
            test_read_from_nonexistent_register,
            test_get_data_from_nonexistent_register
    };

    printf("\n\n🧪RUNNING C INTEGRATION TESTS🧪\n\n");

    for (int i = 0; i < sizeof(tests) / sizeof(test_func); i++) {
        failing |= !tests[i]();
    }

    printf("\n\n");

    return failing;
}
