//! C FFI interface for the herkos transpiler.
//!
//! This module exports C-compatible functions for transpiling WebAssembly binaries.
//! The caller is responsible for freeing allocated memory using `herkos_free`.

use std::os::raw::{c_char, c_int, c_uchar};
use std::ptr;
use std::slice;

use crate::{transpile, TranspileOptions};

/// Result structure for C FFI calls
#[repr(C)]
pub struct HerkosResult {
    /// Pointer to the output Rust string (must be freed with herkos_free)
    pub output: *mut c_char,
    /// Length of the output string in bytes (not including null terminator)
    pub output_len: usize,
    /// Error code: 0 on success, non-zero on failure
    pub error_code: c_int,
    /// Error message (null if no error)
    pub error_msg: *mut c_char,
}

/// Error codes
const HERKOS_OK: c_int = 0;
const HERKOS_ERROR_TRANSPILE: c_int = 1;
const HERKOS_ERROR_INVALID_INPUT: c_int = 2;

/// Transpile WebAssembly bytes to Rust source code using default options.
///
/// # Arguments
/// * `wasm_bytes` - Pointer to the WebAssembly binary data
/// * `wasm_len` - Length of the WebAssembly binary in bytes
///
/// # Returns
/// A `HerkosResult` struct containing:
/// - `output`: Pointer to allocated Rust source code (must be freed with `herkos_free`)
/// - `output_len`: Length of the output string in bytes
/// - `error_code`: 0 on success, non-zero on error
/// - `error_msg`: Null on success, error message string on failure
///
/// # Safety
/// - The caller must ensure `wasm_bytes` points to valid memory of at least `wasm_len` bytes
/// - The returned `output` pointer must be freed using `herkos_free`
/// - The returned `error_msg` pointer (if not null) must be freed using `herkos_free`
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn herkos_transpile(wasm_bytes: *const c_uchar, wasm_len: usize) -> HerkosResult {
    // Validate input pointers
    if wasm_bytes.is_null() {
        let msg = "wasm_bytes pointer is null";
        return HerkosResult {
            output: ptr::null_mut(),
            output_len: 0,
            error_code: HERKOS_ERROR_INVALID_INPUT,
            error_msg: allocate_string(msg),
        };
    }

    if wasm_len == 0 {
        let msg = "wasm_len is 0";
        return HerkosResult {
            output: ptr::null_mut(),
            output_len: 0,
            error_code: HERKOS_ERROR_INVALID_INPUT,
            error_msg: allocate_string(msg),
        };
    }

    // Convert raw pointer to safe slice
    let wasm_slice = unsafe { slice::from_raw_parts(wasm_bytes, wasm_len) };

    // Use default transpilation options
    let options = TranspileOptions::default();

    // Perform transpilation
    match transpile(wasm_slice, &options) {
        Ok(rust_code) => {
            let output = allocate_string(&rust_code);
            let output_len = rust_code.len();
            HerkosResult {
                output,
                output_len,
                error_code: HERKOS_OK,
                error_msg: ptr::null_mut(),
            }
        }
        Err(e) => {
            let error_msg = format!("{:#}", e);
            HerkosResult {
                output: ptr::null_mut(),
                output_len: 0,
                error_code: HERKOS_ERROR_TRANSPILE,
                error_msg: allocate_string(&error_msg),
            }
        }
    }
}

/// Free memory allocated by herkos functions.
///
/// # Arguments
/// * `ptr` - Pointer to memory allocated by herkos (e.g., output from `herkos_transpile`)
///
/// # Safety
/// - The pointer must have been returned by a herkos function
/// - The pointer must not have been freed already
/// - After calling this function, the pointer must not be used
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn herkos_free(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = std::ffi::CString::from_raw(ptr);
        }
    }
}

/// Helper function to allocate a Rust string as C-compatible memory
fn allocate_string(s: &str) -> *mut c_char {
    match std::ffi::CString::new(s) {
        Ok(cstring) => cstring.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}
