/**
 * C Helper Library for Build Pipeline Example
 *
 * This demonstrates C compilation in the build process.
 * The library provides validation functions that could be used
 * by the main Rust binary.
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// Large static data to increase compilation memory usage
static const unsigned long data_table[10000] = {
    // Generated at compile time
    #include "data_table.inc"
};

/**
 * Validate data checksum
 * Allocates memory for working buffers to demonstrate C memory usage
 */
int validate_checksum(const unsigned char* data, size_t len) {
    // Allocate working buffer (demonstrates C memory allocation)
    unsigned long* buffer = (unsigned long*)malloc(len * sizeof(unsigned long));
    if (!buffer) {
        fprintf(stderr, "Failed to allocate validation buffer\n");
        return -1;
    }

    // Process data
    unsigned long checksum = 0;
    for (size_t i = 0; i < len; i++) {
        buffer[i] = data[i] ^ data_table[i % 10000];
        checksum += buffer[i];
    }

    free(buffer);
    return (int)(checksum % 256);
}

/**
 * Process large dataset
 * Demonstrates memory-intensive C operations
 */
void process_large_dataset(void) {
    const size_t size = 100000;

    // Allocate arrays
    double* input = (double*)malloc(size * sizeof(double));
    double* output = (double*)malloc(size * sizeof(double));
    double* workspace = (double*)malloc(size * sizeof(double));

    if (!input || !output || !workspace) {
        fprintf(stderr, "Failed to allocate processing buffers\n");
        goto cleanup;
    }

    // Initialize input
    for (size_t i = 0; i < size; i++) {
        input[i] = (double)i * 3.14159;
    }

    // Process (simple transformation)
    for (size_t i = 0; i < size; i++) {
        workspace[i] = input[i] * input[i];
        output[i] = workspace[i] / (i + 1.0);
    }

    printf("C helper: Processed %zu elements\n", size);

cleanup:
    free(input);
    free(output);
    free(workspace);
}

/**
 * Library initialization
 */
void lib_helper_init(void) {
    printf("C helper library initialized (data table size: %zu)\n",
           sizeof(data_table) / sizeof(data_table[0]));
}
