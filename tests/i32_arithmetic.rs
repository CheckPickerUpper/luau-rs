//! Behaviour-driven proof that i32 arithmetic keeps WebAssembly wrapping semantics.

mod support;

use luau_rs::{
    decode_module, translate_module, DecodeOutcome, MainInvocation, TranslateOptions,
    TranslateOutcome,
};
use rstest::fixture;
use rstest_bdd::Slot;
use rstest_bdd_macros::{given, scenario, then, when, ScenarioState};
use std::io::{Error, ErrorKind};
use std::process::{Command, Output};
use support::official_luau_tool;
use tempfile::TempDir;
use walrus::ir::BinaryOp;
use walrus::{FunctionBuilder, Module, ValType};

#[derive(Default, ScenarioState)]
struct ArithmeticState {
    generated: Slot<String>,
    result: Slot<Output>,
    root: Slot<TempDir>,
}

#[fixture]
fn state() -> ArithmeticState {
    ArithmeticState::default()
}

fn arithmetic_fixture_wasm() -> Vec<u8> {
    let mut module = Module::default();
    add_binary_export(&mut module, "add", BinaryOp::I32Add);
    add_binary_export(&mut module, "sub", BinaryOp::I32Sub);
    add_binary_export(&mut module, "mul", BinaryOp::I32Mul);

    let first = module.locals.add(ValType::I32);
    let second = module.locals.add(ValType::I32);
    let factor = module.locals.add(ValType::I32);
    let mut builder = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32, ValType::I32],
        &[ValType::I32],
    );
    builder
        .func_body()
        .local_get(first)
        .local_get(second)
        .binop(BinaryOp::I32Add)
        .local_get(factor)
        .binop(BinaryOp::I32Mul);
    let chain = builder.finish(vec![first, second, factor], &mut module.funcs);
    module.exports.add("chain", chain);

    module.emit_wasm()
}

fn add_binary_export(module: &mut Module, name: &str, operation: BinaryOp) {
    let left = module.locals.add(ValType::I32);
    let right = module.locals.add(ValType::I32);
    let mut builder = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32],
        &[ValType::I32],
    );
    builder
        .func_body()
        .local_get(left)
        .local_get(right)
        .binop(operation);
    let function = builder.finish(vec![left, right], &mut module.funcs);
    module.exports.add(name, function);
}

fn generate_arithmetic_luau() -> Result<String, Error> {
    let decoded = match decode_module(&arithmetic_fixture_wasm()) {
        DecodeOutcome::Decoded(decoded) => decoded,
        DecodeOutcome::Rejected(rejection) => {
            return Err(Error::other(format!(
                "arithmetic fixture was rejected: {rejection:?}"
            )))
        }
    };
    let options = TranslateOptions::with_main_invocation(MainInvocation::LeaveIdle);
    match translate_module(&decoded, options) {
        TranslateOutcome::Translated(artifact) => Ok(artifact.into_text()),
        TranslateOutcome::Rejected(rejection) => Err(Error::other(format!(
            "arithmetic translation was rejected: {rejection:?}"
        ))),
    }
}

#[given("a module whose arithmetic operations overflow i32")]
fn overflowing_arithmetic_module(state: &ArithmeticState) -> Result<(), Error> {
    state.generated.set(generate_arithmetic_luau()?);
    Ok(())
}

#[when("I evaluate it with official Luau")]
fn run_arithmetic_with_luau(state: &ArithmeticState) -> Result<(), Error> {
    let generated = state.generated.get().ok_or_else(|| {
        Error::new(
            ErrorKind::NotFound,
            "the arithmetic module was not translated before it was run",
        )
    })?;
    let luau_path = official_luau_tool(("LUAU_BIN", "luau"))?;
    let root = tempfile::Builder::new()
        .prefix("luau-rs-i32-arithmetic-bdd")
        .tempdir()?;
    let source_path = root.path().join("driver.luau");
    let driver = format!(
        "local function make()\n{generated}\nend\n\
         local m = make()({{}})\n\
         assert(m.add(2147483647, 1) == -2147483648, \"i32 add overflow mismatch\")\n\
         assert(m.sub(-2147483648, 1) == 2147483647, \"i32 sub overflow mismatch\")\n\
         assert(m.mul(1073741824, 4) == 0, \"i32 mul overflow mismatch\")\n\
         assert(m.chain(2147483647, 1, 2) == 0, \"i32 chained overflow mismatch\")\n"
    );
    fs_err::write(&source_path, &driver)?;
    let result = Command::new(luau_path).arg(&source_path).output()?;
    state.result.set(result);
    state.root.set(root);
    Ok(())
}

#[then("Luau returns the WebAssembly-wrapped results")]
fn wrapping_results_are_correct(state: &ArithmeticState) -> Result<(), Error> {
    let status = state
        .result
        .with_ref(|output| output.status)
        .ok_or_else(|| Error::new(ErrorKind::NotFound, "official Luau did not run"))?;
    if status.success() {
        Ok(())
    } else {
        let stderr = state
            .result
            .with_ref(|output| String::from_utf8_lossy(&output.stderr).into_owned())
            .ok_or_else(|| Error::new(ErrorKind::NotFound, "official Luau result disappeared"))?;
        Err(Error::other(format!(
            "official Luau rejected a wrapping result: success={}, stderr={stderr}",
            status.success()
        )))
    }
}

#[scenario(path = "tests/features/i32_arithmetic.feature")]
fn wrap_overflowing_i32_arithmetic(_state: ArithmeticState) {}
