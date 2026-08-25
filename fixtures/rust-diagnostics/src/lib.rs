#![no_std]

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

use core::arch::wasm32::v128;

/// This function intentionally uses a SIMD value so the backend rejects it.
#[unsafe(no_mangle)]
#[target_feature(enable = "simd128")]
pub unsafe extern "C" fn unsupported_vector(value: v128) -> v128 {
    value
}
