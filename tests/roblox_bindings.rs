//! Roblox binding layer: the wasm fixture drives `runtime/roblox.luau`
//! against a mock Roblox environment under the official Luau tools.

mod support;

use assert_cmd::Command;
use support::official_luau_tool;

/// Reads a text file relative to the crate root.
fn read_repo_text(relative_path: &str) -> String {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative_path);
    match fs_err::read_to_string(&path) {
        Ok(text) => text,
        Err(error) => {
            assert!(false, "could not read {}: {error}", path.display());
            String::new()
        }
    }
}

/// Decodes and translates the committed fixture into Luau text.
fn generated_fixture_luau() -> String {
    let fixture_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/rust-hello/rust_hello.wasm");
    let wasm_bytes = match fs_err::read(&fixture_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            assert!(false, "could not read {}: {error}", fixture_path.display());
            return String::new();
        }
    };
    match luau_rs::decode_module(&wasm_bytes) {
        luau_rs::DecodeOutcome::Decoded(decoded) => {
            let options =
                luau_rs::TranslateOptions::with_main_invocation(luau_rs::MainInvocation::LeaveIdle);
            match luau_rs::translate_module(&decoded, options) {
                luau_rs::TranslateOutcome::Translated(artifact) => artifact.into_text(),
                luau_rs::TranslateOutcome::Rejected(rejection) => {
                    assert!(false, "fixture translation rejected: {rejection:?}");
                    String::new()
                }
            }
        }
        luau_rs::DecodeOutcome::Rejected(rejection) => {
            assert!(false, "fixture rejected: {rejection:?}");
            String::new()
        }
    }
}

/// A mock Roblox environment whose instances record what the runtime does.
const MOCK_ROBLOX_ENVIRONMENT: &str = r#"
local function make_event()
    local connections = {}
    return {
        Connect = function(self, callback)
            table.insert(connections, callback)
            return { Disconnect = function(self) end }
        end,
        Fire = function(self, ...)
            for _, callback in ipairs(connections) do
                callback(...)
            end
        end,
    }
end
local services = {
    Workspace = { Name = "Workspace", Children = {}, Destroy = function(self) end },
}
local game = { GetService = function(self, name) return services[name] end }
local Instance = {
    new = function(className)
        return {
            ClassName = className,
            Children = {},
            Parent = nil,
            Anchored = nil,
            Clicked = make_event(),
            Destroy = function(self) end,
        }
    end,
}
local Vector3 = { new = function(x, y, z) return { x = x, y = y, z = z } end }
local printed = {}
local env = {
    game = game,
    workspace = services.Workspace,
    Instance = Instance,
    Vector3 = Vector3,
    print = function(...) table.insert(printed, table.concat({ ... }, " ")) end,
}
"#;

/// Assembles the driver that instantiates the runtime + module and asserts
/// the `make_part` behavior against the mock environment.
fn driver_source(generated: &str, runtime: &str) -> String {
    let mock = MOCK_ROBLOX_ENVIRONMENT;
    format!(
        "local function make()\n{generated}\nend\n\
         local function make_runtime()\n{runtime}\nend\n\
         local RobloxRuntime = make_runtime()\n\
         {mock}\n\
         local runtime = RobloxRuntime.new(env)\n\
         local m = make()(runtime.imports)\n\
         runtime:bind_memory(m.memory)
         runtime:bind_exports(m)\n\
         local handle = m.make_part(1, 2, 3)\n\
         local part = runtime:instance_at(handle)\n\
         assert(handle == 2, \"part handle should be 2, got \" .. tostring(handle))\n\
         assert(part.ClassName == \"Part\", \"className should be Part\")\n\
         assert(part.Parent == services.Workspace, \"parent should be Workspace\")\n\
         assert(part.Size.x == 1 and part.Size.y == 2 and part.Size.z == 3, \"size mismatch\")\n\
         assert(part.Anchored == 1, \"anchored should be 1\")\n\
         assert(printed[1] == \"part created\", \"print import mismatch\")\n\
         m.subscribe(handle)\n\
         part.Clicked:Fire(21)\n\
         assert(m.get_last_click() == 42, \"event callback mismatch\")\n\
         assert(m.add(20, 22) == 42, \"add mismatch\")\n\
         assert(m.fib(9) == 34, \"fib mismatch\")\n"
    )
}

/// The binding driver must pass the official analyzer.
#[test]
fn roblox_binding_driver_passes_luau_analyze() -> std::result::Result<(), std::io::Error> {
    let generated = generated_fixture_luau();
    let runtime = read_repo_text("runtime/roblox.luau");
    let driver = driver_source(&generated, &runtime);
    let luau_analyze_path = official_luau_tool(("LUAU_ANALYZE_BIN", "luau-analyze"));

    let temp_dir = tempfile::Builder::new()
        .prefix("luau-rs-bindings-analyze")
        .tempdir()?;
    let source_path = temp_dir.path().join("driver.luau");
    fs_err::write(&source_path, &driver)?;

    Command::new(luau_analyze_path)
        .arg(&source_path)
        .assert()
        .success();
    Ok(())
}

/// The binding driver must execute: Rust `make_part` creates a Part through the
/// mock environment exactly as the assertions describe.
#[test]
fn roblox_binding_driver_executes_correctly() -> std::result::Result<(), std::io::Error> {
    let generated = generated_fixture_luau();
    let runtime = read_repo_text("runtime/roblox.luau");
    let driver = driver_source(&generated, &runtime);
    let luau_path = official_luau_tool(("LUAU_BIN", "luau"));

    let temp_dir = tempfile::Builder::new()
        .prefix("luau-rs-bindings-execute")
        .tempdir()?;
    let source_path = temp_dir.path().join("driver.luau");
    fs_err::write(&source_path, &driver)?;

    Command::new(luau_path).arg(&source_path).assert().success();
    Ok(())
}
