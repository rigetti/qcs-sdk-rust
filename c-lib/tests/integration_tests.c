// ANCHOR: all
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
        ProgramResult programResult
) {
    printf("❌ %s failed: %s\n", testName, message);
    free_program_result(programResult);
    return false;
}

bool succeed(const char *testName, ProgramResult programResult) {
    printf("✅ %s succeeded.\n", testName);
    // ANCHOR: free
    free_program_result(programResult);
    // ANCHOR_END: free
    return true;
}

bool test_bell_state() {
    const char *TEST_NAME = "test_bell_state";

    // ANCHOR: run
    unsigned int shots = 3;
    ProgramResult result = run_program_on_qvm(BELL_STATE_PROGRAM, shots, "ro");
    // ANCHOR_END: run

    // ANCHOR: errors
    if (result.tag == ProgramResult_Error) {
        return fail(
                TEST_NAME,
                result.error,
                result
        );
    }
    // ANCHOR_END: errors

    // ANCHOR: byte_check
    if (result.tag != ProgramResult_Byte) {
        char message[50];
        sprintf(message, "Expected type Byte, got tag  %d", result.tag);
        return fail(
                TEST_NAME,
                message,
                result
        );
    }
    // ANCHOR_END: byte_check

    if (result.byte.number_of_shots != shots) {
        char message[50];
        sprintf(message, "Response number of shots was %d, expected %d", result.byte.number_of_shots, shots);
        return fail(
                TEST_NAME,
                message,
                result
        );
    }

    // ANCHOR: results
    for (int shot = 0; shot < result.byte.number_of_shots; shot++) {
        // In our case, we measured two entangled qubits, so we expect their values to be equal.
        int bit_0 = result.byte.data_per_shot[shot][0];
        int bit_1 = result.byte.data_per_shot[shot][1];
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
                    result
            );
        }
    }
    // ANCHOR_END: results

    return succeed(TEST_NAME, result);
}
// ANCHOR_END: all

char *PROGRAM_WITHOUT_MEASUREMENT = "X 0";

bool test_error() {
    const char *TEST_NAME = "test_error";

    unsigned int shots = 1;
    ProgramResult result = run_program_on_qvm(PROGRAM_WITHOUT_MEASUREMENT, shots, "raw");

    if (result.tag != ProgramResult_Error) {
        return fail(
                TEST_NAME,
                "did not receive error result.",
                result
        );
    }

    return succeed(TEST_NAME, result);
}

// ANCHOR: test_real_data
char *REAL_MEMORY_PROGRAM =
        "DECLARE mem REAL[1]\n"
        "MOVE mem[0] 3.141\n";

bool test_real_data() {
    const char *TEST_NAME = "test_real_data";

    unsigned int shots = 2;
    ProgramResult result = run_program_on_qvm(REAL_MEMORY_PROGRAM, shots, "mem");

    if (result.tag != ProgramResult_Real) {
        char message[50];
        sprintf(message, "Got incorrect tag %d", result.tag);
        return fail(
                TEST_NAME,
                message,
                result
        );
    }

    // ANCHOR: real_shot_check
    for (int shot = 0; shot < result.real.number_of_shots; shot++) {
        double *data = result.real.data_per_shot[shot];
        for (int slot = 0; slot < result.real.shot_length; slot++) {
            if (data[slot] != 3.141) {
                char message[50];
                sprintf(
                        message,
                        "Found %f in slot %d, expected 3.141",
                        data[slot],
                        slot
                );
                return fail(
                        TEST_NAME,
                        message,
                        result
                );
            }
        }
    }
    // ANCHOR_END: real_shot_check

    return succeed(TEST_NAME, result);
}
// ANCHOR_END: test_real_data

int main() {
    bool failing = false;

    typedef bool (*test_func)(void);

    static test_func tests[] = {
            test_bell_state,
            test_error,
            test_real_data
    };

    printf("\n\n🧪RUNNING C INTEGRATION TESTS🧪\n\n");

    for (int i = 0; i < sizeof(tests) / sizeof(test_func); i++) {
        failing |= !tests[i]();
    }

    printf("\n\n");

    return failing;
}
