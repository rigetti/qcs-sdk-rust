#include <stdio.h>
#include <string.h>
#include "../libqcs.h"

int test_list_quantum_processors() {
    ListQuantumProcessorResponse response = list_quantum_processors();

    if (response.result != ListQuantumProcessorsResult_Success) {
        printf("Failed with result code %d\n", response.result);
        return -1;
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
        return 0;
    }
    printf("\ntest_list_quantum_processors.c failed: expected Aspen-9 when listing processors.\n\n");
    return -1;
}

char* BELL_STATE_PROGRAM =
        "DECLARE ro BIT[2]\n"
        "H 0\n"
        "CNOT 0 1\n"
        "MEASURE 0 ro[0]\n"
        "MEASURE 1 ro[1]\n";


int test_run_program_on_qvm() {
    uint8_t shots = 2;
    QVMResponse response = run_program_on_qvm(BELL_STATE_PROGRAM, shots);

    if (response.status_code != QVMStatus_Success) {
        printf("\ntest_run_program_on_qvm failed: Response status code was %d\n\n", response.status_code);
        return -1;
    }

    if (response.number_of_shots != shots) {
        printf(
            "\ntest_run_program_on_qvm failed: Response number of shots was %d, expected %d\n\n",
            response.status_code,
            response.number_of_shots
        );
        return -1;
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
            return -1;
        }
    }

    free_qvm_response(response);

    return 0;
}

int main() {
//    if (test_list_quantum_processors() != 0) {
//        return -1;
//    }
    if (test_run_program_on_qvm() != 0) {
        return -1;
    }
    return 0;
}
