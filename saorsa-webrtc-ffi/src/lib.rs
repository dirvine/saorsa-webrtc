//! FFI bindings for mobile and desktop applications

use std::ffi::{c_char, CStr, CString};

/// Initialize the library
#[no_mangle]
pub extern "C" fn saorsa_init(identity: *const c_char) -> *mut std::ffi::c_void {
    // TODO: Implement
    std::ptr::null_mut()
}

/// Start a call
#[no_mangle]
pub extern "C" fn saorsa_call(
    handle: *mut std::ffi::c_void,
    peer: *const c_char,
) -> *mut c_char {
    // TODO: Implement
    std::ptr::null_mut()
}

/// Free resources
#[no_mangle]
pub extern "C" fn saorsa_free(handle: *mut std::ffi::c_void) {
    // TODO: Implement
}
