// ANCHOR: all
// ANCHOR: include
#include <stdio.h>
#include <string.h>
#include "../libqcs.h"
// ANCHOR_END: include

// ANCHOR: program
char* BELL_STATE_PROGRAM =
        "DECLARE ro BIT[2]\n"
        "H 0\n"
        "CNOT 0 1\n"
        "MEASURE 0 ro[0]\n"
        "MEASURE 1 ro[1]\n";
// ANCHOR_END: program

int main() {
    // ANCHOR: run
    unsigned int shots = 3;
    ProgramResult response = run_program_on_qvm(BELL_STATE_PROGRAM, shots, "ro");
    // ANCHOR_END: run

    // ANCHOR: errors
    if (response.error != NULL) {
        printf("\ntest_run_program_on_qvm failed with %s\n\n", response.error);

        // The ProgramResult should always be freed to avoid memory leakage, even in case of an error!
        free_program_result(response);

        return 1;
    }
    // ANCHOR_END: errors

    if (response.number_of_shots != shots) {
        printf(
            "\ntest_run_program_on_qvm failed: Response number of shots was %d, expected %d\n\n",
            response.number_of_shots,
            shots
        );
        free_program_result(response);
        return 1;
    }

    // ANCHOR: results
    for (int shot = 0; shot < response.number_of_shots; shot++) {
        // In our case, we measured two entangled qubits, so we expect their values to be equal.
        int bit_0 = response.results_by_shot[shot][0];
        int bit_1 = response.results_by_shot[shot][1];
        if (bit_0 != bit_1) {
            printf(
                "\ntest_run_program_on_qvm failed: In shot %d, got |%d%d\n\n",
                shot,
                bit_0,
                bit_1
            );
            free_program_result(response);
            return 1;
        }
    }
    // ANCHOR_END: results

    // ANCHOR: free
    free_program_result(response);
    // ANCHOR_END: free

    return 0;
}
// ANCHOR_END: all
