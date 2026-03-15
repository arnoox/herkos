/*
 * example_c_usage.c - Example C program using the herkos transpiler
 *
 * Compile with:
 *   gcc example_c_usage.c -L./target/debug -lherkos_core -o example_c_usage
 *
 * Usage:
 *   ./example_c_usage input.wasm output.rs
 */

#include "herkos.h"
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>

/**
 * Read a file into memory
 * Returns allocated buffer or NULL on error
 */
static uint8_t *read_file(const char *filename, size_t *out_len) {
    FILE *f = fopen(filename, "rb");
    if (!f) {
        perror("fopen");
        return NULL;
    }

    if (fseek(f, 0, SEEK_END) != 0) {
        perror("fseek");
        fclose(f);
        return NULL;
    }

    long file_size = ftell(f);
    if (file_size < 0) {
        perror("ftell");
        fclose(f);
        return NULL;
    }

    if (fseek(f, 0, SEEK_SET) != 0) {
        perror("fseek");
        fclose(f);
        return NULL;
    }

    uint8_t *buffer = malloc((size_t)file_size);
    if (!buffer) {
        fprintf(stderr, "malloc failed\n");
        fclose(f);
        return NULL;
    }

    size_t bytes_read = fread(buffer, 1, (size_t)file_size, f);
    if (bytes_read != (size_t)file_size) {
        fprintf(stderr, "fread failed: expected %ld bytes, got %zu\n", file_size, bytes_read);
        free(buffer);
        fclose(f);
        return NULL;
    }

    fclose(f);
    *out_len = (size_t)file_size;
    return buffer;
}

/**
 * Write buffer to file
 */
static int write_file(const char *filename, const char *data, size_t len) {
    FILE *f = fopen(filename, "wb");
    if (!f) {
        perror("fopen");
        return 1;
    }

    size_t bytes_written = fwrite(data, 1, len, f);
    if (bytes_written != len) {
        fprintf(stderr, "fwrite failed: expected %zu bytes, wrote %zu\n", len, bytes_written);
        fclose(f);
        return 1;
    }

    fclose(f);
    return 0;
}

int main(int argc, char **argv) {
    if (argc != 3) {
        fprintf(stderr, "Usage: %s <input.wasm> <output.rs>\n", argv[0]);
        return 1;
    }

    const char *input_file = argv[1];
    const char *output_file = argv[2];

    printf("Reading WASM from: %s\n", input_file);
    size_t wasm_len = 0;
    uint8_t *wasm_bytes = read_file(input_file, &wasm_len);
    if (!wasm_bytes) {
        fprintf(stderr, "Failed to read input file\n");
        return 1;
    }

    printf("Read %zu bytes, transpiling...\n", wasm_len);
    HerkosResult result = herkos_transpile(wasm_bytes, wasm_len);
    free(wasm_bytes);

    if (result.error_code != HERKOS_OK) {
        fprintf(stderr, "Transpilation failed (error code %d):\n%s\n",
                result.error_code, result.error_msg);
        herkos_free(result.error_msg);
        return 1;
    }

    printf("Transpilation successful, writing output to: %s\n", output_file);
    if (write_file(output_file, result.output, result.output_len) != 0) {
        fprintf(stderr, "Failed to write output file\n");
        herkos_free(result.output);
        return 1;
    }

    printf("Success! Generated %zu bytes of Rust code\n", result.output_len);
    herkos_free(result.output);
    return 0;
}
