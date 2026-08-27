//! Safe Rust wrappers for the small Roblox import surface used by the fixture.

use core::ffi::CStr;
use core::str::Utf8Error;

#[link(wasm_import_module = "roblox")]
unsafe extern "C" {
    fn roblox_print(message_ptr: i32);
    fn roblox_get_service(name_ptr: i32) -> i32;
    fn roblox_instance_new(class_ptr: i32) -> i32;
    fn roblox_set_parent(handle: i32, parent_handle: i32);
    fn roblox_set_property(handle: i32, property_ptr: i32, value: f64);
    fn roblox_set_string_property(handle: i32, property_ptr: i32, value_ptr: i32);
    fn roblox_set_vector3(handle: i32, property_ptr: i32, x: f64, y: f64, z: f64);
    fn roblox_get_property_kind(handle: i32, property_ptr: i32) -> i32;
    fn roblox_get_property(handle: i32, property_ptr: i32) -> f64;
    fn roblox_get_string_property(
        handle: i32,
        property_ptr: i32,
        output_ptr: i32,
        output_capacity: i32,
    ) -> i32;
    fn roblox_destroy(handle: i32);
    fn roblox_connect(handle: i32, event_ptr: i32, callback: extern "C" fn(i32) -> i32) -> i32;
    fn roblox_disconnect(connection_handle: i32) -> i32;
    fn roblox_remote_event_fire_server(handle: i32, remote_ptr: i32, payload: i32);
    fn roblox_remote_event_fire_client(
        handle: i32,
        player_handle: i32,
        remote_ptr: i32,
        payload: i32,
    );
    fn roblox_remote_function_invoke_server(handle: i32, remote_ptr: i32, payload: i32) -> i32;
    fn roblox_remote_function_invoke_client(
        handle: i32,
        player_handle: i32,
        remote_ptr: i32,
        payload: i32,
    ) -> i32;
}

const KIND_MISSING: i32 = 0;
const KIND_NUMBER: i32 = 1;
const KIND_STRING: i32 = 2;
const KIND_VECTOR3: i32 = 3;

/// The result of asking the runtime what type a covered property currently has.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PropertyKind {
    Missing,
    Number,
    String,
    Vector3,
    Unsupported,
}

/// A property read failed at the binding boundary.
///
/// Each variant carries what its own source established, so a caller can tell a
/// value the runtime refused to write from one this side could not describe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PropertyError {
    /// The runtime reported the property as absent.
    Missing,
    /// The property is covered, but currently holds a different type.
    WrongType,
    /// The runtime does not cover the property at all.
    Unsupported,
    /// The bytes the runtime wrote were not UTF-8; the error locates the byte.
    InvalidUtf8(Utf8Error),
    /// The runtime declined to write, answering this negative length instead.
    RuntimeRejectedOutputBuffer(i32),
    /// The output buffer is longer than the `i32` capacity the import carries.
    OutputCapacityExceedsImport(usize),
}

/// The components the runtime uses to build a `Vector3` property value.
///
/// The three axes travel together so a property write names its instance, its
/// property, and one value, rather than five loose positional arguments.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Vector3Components {
    x_component: f64,
    y_component: f64,
    z_component: f64,
}

/// Builds a `Vector3` value. Every triple of finite components is meaningful to
/// the runtime, so there is nothing here to reject.
impl Vector3Components {
    pub(crate) fn new(x_component: f64, y_component: f64, z_component: f64) -> Self {
        Self {
            x_component,
            y_component,
            z_component,
        }
    }
}

// The Roblox imports carry addresses as `i32`, which is exactly how a 32-bit
// target spells a pointer. Building for a wider target would silently change
// what a pointer means at this boundary, so the build stops instead.
#[cfg(not(target_pointer_width = "32"))]
compile_error!(
    "the Roblox fixture bindings require a 32-bit target such as wasm32-unknown-unknown"
);

/// Reinterprets an address as the `i32` the Roblox imports carry.
///
/// On a 32-bit target an address and an `i32` are the same four bytes, so every
/// address has exactly one spelling here and none can be rejected. The byte
/// widths make that total conversion the only one that compiles, which is why
/// there is no failure arm: the previous range check would have hung the module
/// forever on any address past 2GiB, under a panic handler that only loops.
fn pointer<T>(value: *const T) -> i32 {
    i32::from_ne_bytes(value.addr().to_ne_bytes())
}

pub(crate) fn print(message: &CStr) {
    unsafe { roblox_print(pointer(message.as_ptr())) };
}

pub(crate) fn get_service(name: &CStr) -> i32 {
    unsafe { roblox_get_service(pointer(name.as_ptr())) }
}

pub(crate) fn new(class_name: &CStr) -> i32 {
    unsafe { roblox_instance_new(pointer(class_name.as_ptr())) }
}

pub(crate) fn set_parent(handle: i32, parent: i32) {
    unsafe { roblox_set_parent(handle, parent) };
}

pub(crate) fn set_number(handle: i32, property: &CStr, value: f64) {
    unsafe { roblox_set_property(handle, pointer(property.as_ptr()), value) };
}

pub(crate) fn set_string(handle: i32, property: &CStr, value: &CStr) {
    unsafe {
        roblox_set_string_property(handle, pointer(property.as_ptr()), pointer(value.as_ptr()))
    };
}

pub(crate) fn set_vector3(handle: i32, property: &CStr, components: Vector3Components) {
    unsafe {
        roblox_set_vector3(
            handle,
            pointer(property.as_ptr()),
            components.x_component,
            components.y_component,
            components.z_component,
        )
    };
}

pub(crate) fn property_kind(handle: i32, property: &CStr) -> PropertyKind {
    match unsafe { roblox_get_property_kind(handle, pointer(property.as_ptr())) } {
        KIND_MISSING => PropertyKind::Missing,
        KIND_NUMBER => PropertyKind::Number,
        KIND_STRING => PropertyKind::String,
        KIND_VECTOR3 => PropertyKind::Vector3,
        _ => PropertyKind::Unsupported,
    }
}

pub(crate) fn get_number(handle: i32, property: &CStr) -> Result<f64, PropertyError> {
    match property_kind(handle, property) {
        PropertyKind::Number => {
            Ok(unsafe { roblox_get_property(handle, pointer(property.as_ptr())) })
        }
        PropertyKind::Missing => Err(PropertyError::Missing),
        PropertyKind::Unsupported => Err(PropertyError::Unsupported),
        PropertyKind::String | PropertyKind::Vector3 => Err(PropertyError::WrongType),
    }
}

pub(crate) fn get_string<'a>(
    handle: i32,
    property: &CStr,
    output: &'a mut [u8],
) -> Result<&'a str, PropertyError> {
    match property_kind(handle, property) {
        PropertyKind::String => {}
        PropertyKind::Missing => return Err(PropertyError::Missing),
        PropertyKind::Unsupported => return Err(PropertyError::Unsupported),
        PropertyKind::Number | PropertyKind::Vector3 => return Err(PropertyError::WrongType),
    }
    let capacity = match i32::try_from(output.len()) {
        Ok(capacity) => capacity,
        Err(_too_large_for_import) => {
            return Err(PropertyError::OutputCapacityExceedsImport(output.len()))
        }
    };
    let written = unsafe {
        roblox_get_string_property(
            handle,
            pointer(property.as_ptr()),
            pointer(output.as_mut_ptr()),
            capacity,
        )
    };
    // The runtime answers with a negative length when the value did not fit,
    // which is the only way this conversion fails.
    let written = match usize::try_from(written) {
        Ok(byte_count) => byte_count,
        Err(_negative_length) => return Err(PropertyError::RuntimeRejectedOutputBuffer(written)),
    };
    match core::str::from_utf8(&output[..written]) {
        Ok(text) => Ok(text),
        Err(utf8_error) => Err(PropertyError::InvalidUtf8(utf8_error)),
    }
}

pub(crate) fn destroy(handle: i32) {
    unsafe { roblox_destroy(handle) };
}

/// Connects a callback to an event and returns the runtime's connection handle.
pub(crate) fn connect(handle: i32, event: &CStr, callback: extern "C" fn(i32) -> i32) -> i32 {
    unsafe { roblox_connect(handle, pointer(event.as_ptr()), callback) }
}

/// Drops a connection, reporting whether a live one was removed.
pub(crate) fn disconnect(connection_handle: i32) -> i32 {
    unsafe { roblox_disconnect(connection_handle) }
}

pub(crate) fn remote_event_fire_server(handle: i32, remote: &CStr, payload: i32) {
    unsafe { roblox_remote_event_fire_server(handle, pointer(remote.as_ptr()), payload) };
}

pub(crate) fn remote_event_fire_client(
    handle: i32,
    player_handle: i32,
    remote: &CStr,
    payload: i32,
) {
    unsafe {
        roblox_remote_event_fire_client(handle, player_handle, pointer(remote.as_ptr()), payload)
    };
}

pub(crate) fn remote_function_invoke_server(handle: i32, remote: &CStr, payload: i32) -> i32 {
    unsafe { roblox_remote_function_invoke_server(handle, pointer(remote.as_ptr()), payload) }
}

pub(crate) fn remote_function_invoke_client(
    handle: i32,
    player_handle: i32,
    remote: &CStr,
    payload: i32,
) -> i32 {
    unsafe {
        roblox_remote_function_invoke_client(
            handle,
            player_handle,
            pointer(remote.as_ptr()),
            payload,
        )
    }
}
