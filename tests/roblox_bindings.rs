//! Behaviour-driven Roblox binding coverage using the official Luau tools.

mod support;

use rstest::fixture;
use rstest_bdd::Slot;
use rstest_bdd_macros::{given, scenario, then, when, ScenarioState};
use std::io::{Error, ErrorKind};
use std::process::{Command, Output};
use support::official_luau_tool;
use tempfile::TempDir;

#[derive(Default, ScenarioState)]
struct RobloxState {
    generated: Slot<String>,
    runtime: Slot<String>,
    result: Slot<Output>,
    root: Slot<TempDir>,
}

#[fixture]
fn state() -> RobloxState {
    RobloxState::default()
}

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
                    "fixture translation was rejected: {rejection:?}"
                ))),
            }
        }
        luau_rs::DecodeOutcome::Rejected(rejection) => {
            Err(Error::other(format!("fixture was rejected: {rejection:?}")))
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
         m.subscribe(handle)\n\
         part.Clicked:Fire(21)\n\
         assert(m.get_last_click() == 42, \"event callback mismatch\")\n\
         assert(m.add(20, 22) == 42, \"add mismatch\")\n\
         assert(m.fib(9) == 34, \"fib mismatch\")\n"
    )
}

fn required_source(state: &RobloxState) -> Result<(String, String), Error> {
    let generated = state.generated.get().ok_or_else(|| {
        Error::new(
            ErrorKind::NotFound,
            "the generated fixture was not prepared before the binding step",
        )
    })?;
    let runtime = state.runtime.get().ok_or_else(|| {
        Error::new(
            ErrorKind::NotFound,
            "the Roblox runtime was not prepared before the binding step",
        )
    })?;
    Ok((generated, runtime))
}

#[given("a generated Rust module with its Roblox runtime and a test world")]
fn translated_fixture_and_runtime(state: &RobloxState) -> Result<(), Error> {
    state.generated.set(generated_fixture_luau()?);
    state.runtime.set(read_repo_text("runtime/roblox.luau")?);
    Ok(())
}

#[when("Luau analysis checks the module and runtime together")]
fn analyze_binding_driver(state: &RobloxState) -> Result<(), Error> {
    let (generated, runtime) = required_source(state)?;
    let analyzer = official_luau_tool(("LUAU_ANALYZE_BIN", "luau-analyze"))?;
    let root = tempfile::Builder::new()
        .prefix("luau-rs-bindings-bdd-analyze")
        .tempdir()?;
    let source_path = root.path().join("driver.luau");
    fs_err::write(&source_path, driver_source(&generated, &runtime))?;
    let result = Command::new(analyzer).arg(&source_path).output()?;
    state.result.set(result);
    state.root.set(root);
    Ok(())
}

#[when("the module creates a Part at position (1, 2, 3) and handles click 21")]
fn run_binding_driver(state: &RobloxState) -> Result<(), Error> {
    let (generated, runtime) = required_source(state)?;
    let luau = official_luau_tool(("LUAU_BIN", "luau"))?;
    let root = tempfile::Builder::new()
        .prefix("luau-rs-bindings-bdd-run")
        .tempdir()?;
    let source_path = root.path().join("driver.luau");
    fs_err::write(&source_path, driver_source(&generated, &runtime))?;
    let result = Command::new(luau).arg(&source_path).output()?;
    state.result.set(result);
    state.root.set(root);
    Ok(())
}

#[then("the combined program passes analysis without errors")]
fn analyzer_accepts_binding_driver(state: &RobloxState) -> Result<(), Error> {
    let success = state
        .result
        .with_ref(|output| output.status.success())
        .ok_or_else(|| Error::new(ErrorKind::NotFound, "Luau analysis did not run"))?;
    if success {
        Ok(())
    } else {
        let stderr = state
            .result
            .with_ref(|output| String::from_utf8_lossy(&output.stderr).into_owned())
            .ok_or_else(|| Error::new(ErrorKind::NotFound, "analysis result disappeared"))?;
        Err(Error::other(format!(
            "binding driver failed Luau analysis: success={success}, stderr={stderr}"
        )))
    }
}

#[then("the Roblox test world contains an anchored Part of size (1, 2, 3), and the module reports click 42 with add 42 and fib 34")]
fn binding_driver_completes(state: &RobloxState) -> Result<(), Error> {
    let success = state
        .result
        .with_ref(|output| output.status.success())
        .ok_or_else(|| Error::new(ErrorKind::NotFound, "the binding driver did not run"))?;
    if success {
        Ok(())
    } else {
        let stderr = state
            .result
            .with_ref(|output| String::from_utf8_lossy(&output.stderr).into_owned())
            .ok_or_else(|| Error::new(ErrorKind::NotFound, "binding result disappeared"))?;
        Err(Error::other(format!(
            "binding driver failed against the Roblox mock: success={success}, stderr={stderr}"
        )))
    }
}

#[scenario(path = "tests/features/roblox_bindings.feature")]
fn analyze_roblox_binding_driver(_state: RobloxState) {}

#[scenario(path = "tests/features/roblox_bindings.feature")]
fn execute_roblox_binding_driver(_state: RobloxState) {}
