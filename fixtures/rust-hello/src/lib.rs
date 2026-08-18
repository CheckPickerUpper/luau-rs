//! Fixture crate compiled to wasm32 and fed through the luau-rs compiler.

/// Adds two 32-bit integers.
#[unsafe(no_mangle)]
pub extern "C" fn add(left: i32, right: i32) -> i32 {
    left + right
}

/// Computes the 9th Fibonacci number with a loop and a branch.
#[unsafe(no_mangle)]
pub extern "C" fn fib(index: i32) -> i32 {
    let mut a = 0;
    let mut b = 1;
    let mut current = 0;
    let mut i = 0;
    while i < index {
        current = a + b;
        a = b;
        b = current;
        i += 1;
    }
    a
}

/// Doubles the value stored at the given memory address.
#[unsafe(no_mangle)]
pub extern "C" fn double_at(address: *mut i32) {
    unsafe {
        *address *= 2;
    }
}
