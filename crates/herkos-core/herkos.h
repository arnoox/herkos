/*
 * herkos.h - C FFI interface for the herkos WebAssembly to Rust transpiler
 *
 * This header provides C-compatible bindings for transpiling WebAssembly
 * modules into memory-safe Rust source code.
 */

#ifndef HERKOS_H
#define HERKOS_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Error codes */
#define HERKOS_OK 0
#define HERKOS_ERROR_TRANSPILE 1
#define HERKOS_ERROR_INVALID_INPUT 2

/**
 * Result structure returned by transpilation functions.
 *
 * On success (error_code == HERKOS_OK):
 *   - output: allocated string containing the Rust source code
 *   - output_len: length of the output string in bytes (not including null terminator)
 *   - error_msg: NULL
 *
 * On failure (error_code != HERKOS_OK):
 *   - output: NULL
 *   - output_len: 0
 *   - error_msg: allocated string describing the error
 *
 * Both output and error_msg (if non-null) must be freed using herkos_free().
 */
typedef struct {
    char *output;
    size_t output_len;
    int32_t error_code;
    char *error_msg;
} HerkosResult;

/**
 * Transpile WebAssembly bytes to Rust source code using default options.
 *
 * This is the main entry point for transpiling WASM binaries. It uses default
 * transpilation options (safe backend, 256 max pages, no optimizations).
 *
 * @param wasm_bytes Pointer to the WebAssembly binary data
 * @param wasm_len   Length of the WebAssembly binary in bytes
 * @return           HerkosResult containing output or error
 *
 * Example:
 *   uint8_t *wasm_bytes = ...; // Load WASM binary
 *   size_t wasm_len = ...;
 *
 *   HerkosResult result = herkos_transpile(wasm_bytes, wasm_len);
 *   if (result.error_code == HERKOS_OK) {
 *       // Use result.output (a null-terminated C string)
 *       printf("%s\n", result.output);
 *       herkos_free(result.output);
 *   } else {
 *       fprintf(stderr, "Transpilation failed: %s\n", result.error_msg);
 *       herkos_free(result.error_msg);
 *   }
 */
HerkosResult herkos_transpile(const uint8_t *wasm_bytes, size_t wasm_len);

/**
 * Free memory allocated by herkos functions.
 *
 * This function must be called to free:
 *   - HerkosResult.output (on success)
 *   - HerkosResult.error_msg (on failure)
 *
 * @param ptr Pointer to memory allocated by herkos
 *
 * Passing NULL is safe and does nothing.
 * Do not free the same pointer twice.
 */
void herkos_free(char *ptr);

#ifdef __cplusplus
}
#endif

#endif /* HERKOS_H */
