//! Official-Luau proof for logical i64 values in select and drop.

mod support;

use luau_rs::{
    decode_module, translate_module, DecodeOutcome, MainInvocation, TranslateOptions,
    TranslateOutcome,
};
use std::io::Error;
use std::process::Command;
use support::official_luau_tool;
use walrus::ir::Select;
use walrus::{FunctionBuilder, Module, ValType};

fn stack_shape_fixture_wasm() -> Vec<u8> {
    let mut module = Module::default();
    let condition = module.locals.add(ValType::I32);
    let mut builder = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I64]);
    builder
        .func_body()
        .i64_const(0x1_0000_0001)
        .i64_const(-1)
        .local_get(condition)
        .instr(Select { ty: None })
        .i64_const(99)
        .drop();
    let function = builder.finish(vec![condition], &mut module.funcs);
    module.exports.add("choose", function);
    module.emit_wasm()
}

fn generated_stack_shape_luau() -> Result<String, Error> {
    let decoded = match decode_module(&stack_shape_fixture_wasm()) {
        DecodeOutcome::Decoded(decoded) => decoded,
        DecodeOutcome::Rejected(rejection) => {
            return Err(Error::other(format!(
                "stack-shape fixture was rejected: {rejection:?}"
            )))
        }
    };
    match translate_module(
        &decoded,
        TranslateOptions::with_main_invocation(MainInvocation::LeaveIdle),
    ) {
        TranslateOutcome::Translated(artifact) => Ok(artifact.into_text()),
        TranslateOutcome::Rejected(rejection) => Err(Error::other(format!(
            "stack-shape translation was rejected: {rejection:?}"
        ))),
    }
}

fn run_official_luau(source: &str, driver: &str) -> Result<(), Error> {
    let luau_path = official_luau_tool(("LUAU_BIN", "luau"))?;
    let temp_directory = tempfile::Builder::new()
        .prefix("luau-rs-i64-stack-shape")
        .tempdir()?;
    let source_path = temp_directory.path().join("driver.luau");
    fs_err::write(
        &source_path,
        format!("local function make()\n{source}\nend\n{driver}"),
    )?;
    let output = Command::new(luau_path).arg(source_path).output()?;
    if output.status.success() {
        Ok(())
    } else {
        Err(Error::other(format!(
            "official Luau rejected the i64 stack-shape proof with status {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )))
    }
}

#[test]
fn given_i64_select_and_drop_when_run_in_official_luau_then_stack_width_is_preserved(
) -> Result<(), Error> {
    let generated = generated_stack_shape_luau()?;
    let driver = r#"local m = make()({})
local low_true, high_true = m.choose(1)
assert(low_true == 1 and high_true == 1, string.format("true select result mismatch: %s, %s", low_true, high_true))
local low_false, high_false = m.choose(0)
assert(low_false == 4294967295 and high_false == 4294967295, string.format("false select result mismatch: %s, %s", low_false, high_false))
"#;
    run_official_luau(&generated, driver)
}
