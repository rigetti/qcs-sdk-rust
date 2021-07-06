#include <stdio.h>
#include <string.h>
#include "../libqcs.h"

int main() {
    ListQuantumProcessorResponse response = list_quantum_processors();

    if (response.result != Success) {
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
