//! Behavior scenarios for using generated Rust modules with Roblox objects.

mod support;

use rstest::rstest;
use std::io::Error;
use std::process::Command;
use support::official_luau_tool;

fn read_repo_text(relative_path: &str) -> Result<String, Error> {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative_path);
    fs_err::read_to_string(&path).map_err(|error| {
        Error::new(
            error.kind(),
            format!("could not read {}: {error}", path.display()),
        )
    })
}

fn generated_fixture_luau() -> Result<String, Error> {
    let fixture_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/rust-hello/rust_hello.wasm");
    let wasm_bytes = fs_err::read(&fixture_path).map_err(|error| {
        Error::new(
            error.kind(),
            format!("could not read {}: {error}", fixture_path.display()),
        )
    })?;
    match luau_rs::decode_module(&wasm_bytes) {
        luau_rs::DecodeOutcome::Decoded(decoded) => {
            let options =
                luau_rs::TranslateOptions::with_main_invocation(luau_rs::MainInvocation::LeaveIdle);
            match luau_rs::translate_module(&decoded, options) {
                luau_rs::TranslateOutcome::Translated(artifact) => Ok(artifact.into_text()),
                luau_rs::TranslateOutcome::Rejected(rejection) => Err(Error::other(format!(
                    "fixture translation was rejected: rejection={rejection:?}"
                ))),
            }
        }
        luau_rs::DecodeOutcome::Rejected(rejection) => Err(Error::other(format!(
            "fixture was rejected: rejection={rejection:?}"
        ))),
    }
}

/// The mock Roblox environment used by the behavior scenario.
const MOCK_ROBLOX_ENVIRONMENT: &str = r#"
local function make_event()
    local callbacks = {}
    local next_connection = 1
    return {
        Connect = function(self, callback)
            local connection_id = next_connection
            next_connection += 1
            callbacks[connection_id] = callback
            return {
                Disconnect = function(self)
                    callbacks[connection_id] = nil
                end,
            }
        end,
        Fire = function(self, ...)
            for _, callback in pairs(callbacks) do
                callback(...)
            end
        end,
    }
end
local services = {
    Workspace = { Name = "Workspace", Children = {}, Destroy = function(self) end },
    ReplicatedStorage = { Name = "ReplicatedStorage", Children = {}, Destroy = function(self) end },
}
local game = { GetService = function(self, name) return services[name] end }
local Instance = {
    new = function(className)
        local instance = {
            ClassName = className,
            Children = {},
            Parent = nil,
            Anchored = nil,
            Clicked = make_event(),
            Destroy = function(self) end,
        }
        if className == "RemoteEvent" then
            instance.FireServer = function(self, payload)
                self.LastServerPayload = payload
            end
            instance.FireClient = function(self, player, payload)
                self.LastClientPlayer = player
                self.LastClientPayload = payload
            end
        elseif className == "RemoteFunction" then
            instance.InvokeServer = function(self, payload)
                return payload * 2
            end
            instance.InvokeClient = function(self, player, payload)
                self.LastClientPlayer = player
                return payload * 3
            end
        end
        return instance
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

fn driver_source(generated: &str, runtime: &str) -> String {
    format!(
        "local function make()\n{generated}\nend\n\
         local function make_runtime()\n{runtime}\nend\n\
         local RobloxRuntime = make_runtime()\n\
         {MOCK_ROBLOX_ENVIRONMENT}\
         local runtime = RobloxRuntime.new(env)\n\
         local m = make()(runtime.imports)\n\
         runtime:bind_memory(m.memory)\n\
         runtime:bind_exports(m)\n\
         local handle = m.make_part(1, 2, 3)\n\
         local part = runtime:instance_at(handle)\n\
         assert(handle == 2, \"part handle should be 2, got \" .. tostring(handle))\n\
         assert(part.ClassName == \"Part\", \"className should be Part\")\n\
         assert(part.Parent == services.Workspace, \"parent should be Workspace\")\n\
         assert(part.Size.x == 1 and part.Size.y == 2 and part.Size.z == 3, \"size mismatch\")\n\
         assert(part.Anchored == 1, \"anchored should be 1\")\n\
         assert(printed[1] == \"part created\", \"print import mismatch\")\n\
         local connection = m.subscribe(handle)\n\
         part.Clicked:Fire(21)\n\
         assert(m.get_last_click() == 42, \"event callback mismatch\")\n\
         assert(m.unsubscribe(connection) == 1, \"disconnect mismatch\")\n\
         part.Clicked:Fire(9)\n\
         assert(m.get_last_click() == 42, \"disconnected callback fired\")\n\
         local remote_event = m.make_remote_event()\n\
         m.fire_remote_event(remote_event, 17)\n\
         local remote_event_instance = runtime:instance_at(remote_event)\n\
         assert(remote_event_instance.LastServerPayload == 17, \"RemoteEvent payload mismatch\")\n\
         local player = m.make_player()\n\
         m.fire_remote_event_to_client(remote_event, player, 19)\n\
         assert(remote_event_instance.LastClientPlayer == runtime:instance_at(player), \"RemoteEvent player mismatch\")\n\
         assert(remote_event_instance.LastClientPayload == 19, \"RemoteEvent client payload mismatch\")\n\
         local remote_function = m.make_remote_function()\n\
         assert(m.invoke_remote_function(remote_function, 21) == 42, \"RemoteFunction result mismatch\")\n\
         assert(m.invoke_remote_function_on_client(remote_function, player, 14) == 42, \"RemoteFunction client result mismatch\")\n\
         assert(m.add(20, 22) == 42, \"add mismatch\")\n\
         assert(m.fib(9) == 34, \"fib mismatch\")\n"
    )
}

#[rstest]
fn given_generated_module_when_analyzed_with_roblox_runtime_then_luau_is_accepted(
) -> Result<(), Error> {
    // Given a generated Rust module, its Roblox runtime, and a test world.
    let generated = generated_fixture_luau()?;
    let runtime = read_repo_text("runtime/roblox.luau")?;
    let analyzer = official_luau_tool(("LUAU_ANALYZE_BIN", "luau-analyze"))?;
    let temp_dir = tempfile::Builder::new()
        .prefix("luau-rs-bindings-bdd-analyze")
        .tempdir()?;
    let source_path = temp_dir.path().join("driver.luau");
    fs_err::write(&source_path, driver_source(&generated, &runtime))?;

    // When Luau analysis checks the module and runtime together.
    let result = Command::new(analyzer).arg(&source_path).output()?;

    // Then the combined program passes analysis without errors.
    let success = result.status.success();
    if success {
        Ok(())
    } else {
        Err(Error::other(format!(
            "Roblox Luau analysis failed: success={success}, stderr={}",
            String::from_utf8_lossy(&result.stderr)
        )))
    }
}

#[rstest]
fn given_generated_module_when_run_against_roblox_world_then_part_event_and_exports_behave(
) -> Result<(), Error> {
    // Given a generated Rust module, its Roblox runtime, and a test world.
    let generated = generated_fixture_luau()?;
    let runtime = read_repo_text("runtime/roblox.luau")?;
    let luau = official_luau_tool(("LUAU_BIN", "luau"))?;
    let temp_dir = tempfile::Builder::new()
        .prefix("luau-rs-bindings-bdd-run")
        .tempdir()?;
    let source_path = temp_dir.path().join("driver.luau");
    fs_err::write(&source_path, driver_source(&generated, &runtime))?;

    // When the module creates a Part at (1, 2, 3), subscribes to its click event,
    // and handles click 21.
    let result = Command::new(luau).arg(&source_path).output()?;

    // Then the test world has the anchored Part and the module reports click 42,
    // add 42, and fib 34. The Luau assertions above are the independent oracle.
    let success = result.status.success();
    if success {
        Ok(())
    } else {
        Err(Error::other(format!(
            "Roblox behavior failed: success={success}, stderr={}",
            String::from_utf8_lossy(&result.stderr)
        )))
    }
}
