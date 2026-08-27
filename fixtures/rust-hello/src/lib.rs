//! Fixture crate compiled to wasm32 and fed through the luau-rs compiler.
#![no_std]

mod roblox;

/// Panic handler for the panic-abort profile; never called in practice.
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
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

/// Connects a Rust callback to a mock event and returns the connection handle.
#[unsafe(no_mangle)]
pub extern "C" fn subscribe(handle: i32) -> i32 {
    roblox::connect(handle, c"Clicked", on_clicked)
}

/// Disconnects an event callback and reports whether a live connection was removed.
#[unsafe(no_mangle)]
pub extern "C" fn unsubscribe(connection_handle: i32) -> i32 {
    roblox::disconnect(connection_handle)
}

/// Creates a player-shaped Instance for client-directed remote calls.
#[unsafe(no_mangle)]
pub extern "C" fn make_player() -> i32 {
    roblox::new(c"Player")
}

/// Creates a RemoteEvent under ReplicatedStorage and returns its handle.
#[unsafe(no_mangle)]
pub extern "C" fn make_remote_event() -> i32 {
    let service = roblox::get_service(c"ReplicatedStorage");
    let remote = roblox::new(c"RemoteEvent");
    roblox::set_parent(remote, service);
    remote
}

/// Sends a numeric payload through a server-directed RemoteEvent import.
#[unsafe(no_mangle)]
pub extern "C" fn fire_remote_event(handle: i32, payload: i32) {
    roblox::remote_event_fire_server(handle, c"RemoteEvent", payload);
}

/// Sends a numeric payload through a client-directed RemoteEvent import.
#[unsafe(no_mangle)]
pub extern "C" fn fire_remote_event_to_client(handle: i32, player_handle: i32, payload: i32) {
    roblox::remote_event_fire_client(handle, player_handle, c"RemoteEvent", payload);
}

/// Creates a RemoteFunction under ReplicatedStorage and returns its handle.
#[unsafe(no_mangle)]
pub extern "C" fn make_remote_function() -> i32 {
    let service = roblox::get_service(c"ReplicatedStorage");
    let remote = roblox::new(c"RemoteFunction");
    roblox::set_parent(remote, service);
    remote
}

/// Invokes a numeric server-directed RemoteFunction and returns its result.
#[unsafe(no_mangle)]
pub extern "C" fn invoke_remote_function(handle: i32, payload: i32) -> i32 {
    roblox::remote_function_invoke_server(handle, c"RemoteFunction", payload)
}

/// Invokes a numeric client-directed RemoteFunction and returns its result.
#[unsafe(no_mangle)]
pub extern "C" fn invoke_remote_function_on_client(
    handle: i32,
    player_handle: i32,
    payload: i32,
) -> i32 {
    roblox::remote_function_invoke_client(handle, player_handle, c"RemoteFunction", payload)
}

/// Creates a Part under Workspace with the given Size and returns its handle.
#[unsafe(no_mangle)]
pub extern "C" fn make_part(x: f64, y: f64, z: f64) -> i32 {
    let workspace_name = c"Workspace";
    let part_class = c"Part";
    let size_property = c"Size";

    let workspace = roblox::get_service(workspace_name);
    let part = roblox::new(part_class);
    roblox::set_vector3(part, size_property, x, y, z);
    roblox::set_number(part, c"Anchored", 1.0);
    roblox::set_string(part, c"Name", c"GeneratedPart");
    roblox::set_parent(part, workspace);
    roblox::print(c"part created");
    part
}

/// Reads the configured part name through the safe binding wrapper.
#[unsafe(no_mangle)]
pub extern "C" fn part_name_is_generated(handle: i32) -> i32 {
    let mut output = [0_u8; 32];
    match roblox::get_string(handle, c"Name", &mut output) {
        Ok(name) if name == "GeneratedPart" => 1,
        Ok(_) | Err(_) => 0,
    }
}

/// Reads the configured numeric property through the safe binding wrapper.
#[unsafe(no_mangle)]
pub extern "C" fn part_is_anchored(handle: i32) -> i32 {
    match roblox::get_number(handle, c"Anchored") {
        Ok(value) if value == 1.0 => 1,
        Ok(_) | Err(_) => 0,
    }
}

/// Exposes the missing-property distinction to the integration oracle.
#[unsafe(no_mangle)]
pub extern "C" fn unset_property_kind(handle: i32) -> i32 {
    match roblox::property_kind(handle, c"CanCollide") {
        roblox::PropertyKind::Missing => 0,
        roblox::PropertyKind::Number => 1,
        roblox::PropertyKind::String => 2,
        roblox::PropertyKind::Vector3 => 3,
        roblox::PropertyKind::Unsupported => 4,
    }
}

/// Destroys an instance through the safe binding wrapper.
#[unsafe(no_mangle)]
pub extern "C" fn destroy_part(handle: i32) {
    roblox::destroy(handle);
}
