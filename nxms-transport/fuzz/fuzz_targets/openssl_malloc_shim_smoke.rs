#![no_main]

use libfuzzer_sys::fuzz_target;

unsafe extern "C" {
    fn nxms_openssl_malloc_probe() -> i32;
}

fuzz_target!(|_data: &[u8]| {
    let rc = unsafe { nxms_openssl_malloc_probe() };
    assert_eq!(rc, 0, "nxms_openssl_malloc_probe failed with rc={rc}");
});
