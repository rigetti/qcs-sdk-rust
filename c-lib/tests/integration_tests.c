#include <stdio.h>
#include <string.h>
#include "../libqcs.h"

bool test_list_quantum_processors() {
    ListQuantumProcessorResponse response = list_quantum_processors();

    if (response.result != ListQuantumProcessorsResult_Success) {
        printf("test_list_quantum_processors: Failed with result code %d\n", response.result);
        return false;
    }

    bool aspen_9_found = false;

    for (int ii = 0; ii < response.len; ii++) {
        if (strcmp(response.processors[ii].id, "Aspen-9")) {
            aspen_9_found = true;
            break;
        }
    }

    free_quantum_processors(response);

    if (aspen_9_found) {
        return true;
    }
    printf("\ntest_list_quantum_processors.c failed: expected Aspen-9 when listing processors.\n\n");
    return false;
}

char* BELL_STATE_PROGRAM =
        "DECLARE ro BIT[2]\n"
        "H 0\n"
        "CNOT 0 1\n"
        "MEASURE 0 ro[0]\n"
        "MEASURE 1 ro[1]\n";


bool test_run_program_on_qvm() {
    uint8_t shots = 2;
    QVMResponse response = run_program_on_qvm(BELL_STATE_PROGRAM, shots, "ro");

    if (response.status_code != QVMStatus_Success) {
        printf("\ntest_run_program_on_qvm failed: Response status code was %d\n\n", response.status_code);
        return false;
    }

    if (response.number_of_shots != shots) {
        printf(
            "\ntest_run_program_on_qvm failed: Response number of shots was %d, expected %d\n\n",
            response.status_code,
            response.number_of_shots
        );
        return false;
    }

    for (int shot = 0; shot < response.number_of_shots; shot++) {
        int bit_0 = response.results_by_shot[shot][0];
        int bit_1 = response.results_by_shot[shot][1];
        if (bit_0 != bit_1) {
            printf(
                "\ntest_run_program_on_qvm failed: In shot %d, got |%d%d\n\n",
                shot,
                bit_0,
                bit_1
            );
            return false;
        }
    }

    free_qvm_response(response);

    return true;
}

int main() {
    bool failing = false;

    typedef bool (*test_func)(void);

    static test_func tests[] = {
        test_list_quantum_processors,
        test_run_program_on_qvm
    };

    for (int i = 0; i < sizeof(tests) / sizeof(test_func); i++) {
        failing |= !tests[i]();
    }

    return failing;
}
