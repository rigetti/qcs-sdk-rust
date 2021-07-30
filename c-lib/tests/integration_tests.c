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

bool fail(ProgramResult programResult) {
    // The ProgramResult should always be freed to avoid memory leakage, even in case of an error!
    free_program_result(programResult);
    return false;
}

bool succeed(ProgramResult programResult) {
    free_program_result(programResult);
    return true;
}

bool test_bell_state() {
    // ANCHOR: run
    unsigned int shots = 3;
    ProgramResult response = run_program_on_qvm(BELL_STATE_PROGRAM, shots, "ro");
    // ANCHOR_END: run

    // ANCHOR: errors
    if (response.tag == ProgramResult_Error) {
        printf("❌ test_bell_state failed with %s\n", response.error);
        return fail(response);
    }
    // ANCHOR_END: errors

    if (response.tag != ProgramResult_Byte) {
        printf(
                "❌ test_bell_state failed: Expected type Byte, got tag  %d\n",
                response.tag
        );
        return fail(response);
    }

    if (response.byte.number_of_shots != shots) {
        printf(
                "❌ test_bell_state failed: Response number of shots was %d, expected %d\n",
                response.byte.number_of_shots,
                shots
        );
        return fail(response);
    }

    // ANCHOR: results
    for (int shot = 0; shot < response.byte.number_of_shots; shot++) {
        // In our case, we measured two entangled qubits, so we expect their values to be equal.
        int bit_0 = response.byte.data_per_shot[shot][0];
        int bit_1 = response.byte.data_per_shot[shot][1];
        if (bit_0 != bit_1) {
            printf(
                    "❌ test_bell_state failed: In shot %d, got |%d%d\n",
                    shot,
                    bit_0,
                    bit_1
            );
            return fail(response);
        }
    }
    // ANCHOR_END: results

    // ANCHOR: free
    free_program_result(response);
    // ANCHOR_END: free

    printf("✅ test_bell_state succeeded.\n");
    return true;
}
// ANCHOR_END: all

char *PROGRAM_WITHOUT_MEASUREMENT = "X 0";

bool test_error() {
    unsigned int shots = 1;
    ProgramResult response = run_program_on_qvm(PROGRAM_WITHOUT_MEASUREMENT, shots, "raw");

    if (response.tag != ProgramResult_Error) {
        printf("❌ test_error did not receive error response.\n");
        return fail(response);
    }

    printf("✅ test_error succeeded.\n");

    return succeed(response);
}

char *REAL_MEMORY_PROGRAM =
        "DECLARE mem REAL[1]\n"
        "MOVE mem[0] 3.141\n";

bool test_real_data() {
    unsigned int shots = 2;
    ProgramResult response = run_program_on_qvm(REAL_MEMORY_PROGRAM, shots, "mem");

    if (response.tag != ProgramResult_Real) {
        printf("❌ test_real_data failed with tag %d, error: %s\n", response.tag, response.error);
        return fail(response);
    }

    for (int shot = 0; shot < shots; shot++) {
        double *data = response.real.data_per_shot[shot];
        for (int slot = 0; slot < response.real.shot_length; slot++) {
            if (data[slot] != 3.141) {
                printf(
                        "❌ test_real_data failed: Found %f in slot %d\n",
                        data[slot],
                        slot
                );
                return fail(response);
            }
        }
    }

    printf("✅ test_real_data succeeded.\n");
    return succeed(response);
}

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
