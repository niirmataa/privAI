#![no_main]

use libfuzzer_sys::fuzz_target;
use std::ffi::{c_char, c_int, c_void};
use std::ptr;

unsafe extern "C" {
    fn CRYPTO_malloc(num: usize, file: *const c_char, line: c_int) -> *mut c_void;
    fn CRYPTO_free(ptr: *mut c_void, file: *const c_char, line: c_int);
}

fuzz_target!(|_data: &[u8]| {
    let p = unsafe { CRYPTO_malloc(32, ptr::null(), 0) };
    assert!(!p.is_null(), "CRYPTO_malloc returned null");
    unsafe { CRYPTO_free(p, ptr::null(), 0) };
});
