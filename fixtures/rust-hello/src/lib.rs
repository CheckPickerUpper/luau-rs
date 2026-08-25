//! Fixture crate compiled to wasm32 and fed through the luau-rs compiler.
#![no_std]

use core::ffi::CStr;

/// Panic handler for the panic-abort profile; never called in practice.
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

// Imports provided by `runtime/roblox.luau` through the instantiation seam.
#[link(wasm_import_module = "roblox")]
unsafe extern "C" {
    fn roblox_print(message_ptr: i32);
    fn roblox_get_service(name_ptr: i32) -> i32;
    fn roblox_instance_new(class_ptr: i32, parent_handle: i32) -> i32;
    fn roblox_set_property(handle: i32, property_ptr: i32, value: f64);
    fn roblox_set_vector3(handle: i32, property_ptr: i32, x: f64, y: f64, z: f64);
    fn roblox_connect(handle: i32, event_ptr: i32, callback: extern "C" fn(i32) -> i32);
}

/// Casts a static C string's pointer to the wasm32 `i32` address space.
///
/// wasm32-unknown-unknown is a 32-bit address space, so the conversion is
/// total; the failure arm exists only to name it under `TryFrom`.
fn c_str_pointer(text: &CStr) -> i32 {
    let address = text.as_ptr().addr();
    match i32::try_from(address) {
        Ok(pointer) => pointer,
        Err(conversion_error) => {
            assert!(
                false,
                "wasm32 pointer conversion failed: {conversion_error}"
            );
            0
        }
    }
}

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
    let mut i = 0;
    while i < index {
        let current = a + b;
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

/// The most recent value the event callback observed, observable by tests.
static mut LAST_CLICK: i32 = 0;

/// Event callback registered through `roblox_connect`; doubles its input.
extern "C" fn on_clicked(value: i32) -> i32 {
    unsafe {
        LAST_CLICK = value * 2;
    }
    value * 2
}

/// Returns the last value the event callback observed.
#[unsafe(no_mangle)]
pub extern "C" fn get_last_click() -> i32 {
    unsafe { LAST_CLICK }
}

/// Connects a Rust callback to a mock event on the given instance handle.
#[unsafe(no_mangle)]
pub extern "C" fn subscribe(handle: i32) {
    unsafe {
        roblox_connect(handle, c_str_pointer(c"Clicked"), on_clicked);
    }
}

/// Creates a Part under Workspace with the given Size and returns its handle.
#[unsafe(no_mangle)]
pub extern "C" fn make_part(x: f64, y: f64, z: f64) -> i32 {
    let workspace_name = c"Workspace";
    let part_class = c"Part";
    let size_property = c"Size";

    let workspace = unsafe { roblox_get_service(c_str_pointer(workspace_name)) };
    let part = unsafe { roblox_instance_new(c_str_pointer(part_class), workspace) };
    unsafe {
        roblox_set_vector3(part, c_str_pointer(size_property), x, y, z);
        roblox_set_property(part, c_str_pointer(c"Anchored"), 1.0);
    }
    unsafe {
        roblox_print(c_str_pointer(c"part created"));
    }
    part
}
