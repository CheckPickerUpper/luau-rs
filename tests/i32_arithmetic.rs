//! Behavior scenario for WebAssembly i32 wrapping semantics in Luau.

mod support;

use luau_rs::{
    decode_module, translate_module, DecodeOutcome, MainInvocation, TranslateOptions,
    TranslateOutcome,
};
use rstest::rstest;
use std::io::Error;
use std::process::Command;
use support::official_luau_tool;
use walrus::ir::BinaryOp;
use walrus::{FunctionBuilder, Module, ValType};

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
    // Given a generated module containing overflowing i32 operations.
    let decoded = match decode_module(&arithmetic_fixture_wasm()) {
        DecodeOutcome::Decoded(decoded) => decoded,
        DecodeOutcome::Rejected(rejection) => {
            return Err(Error::other(format!(
                "arithmetic fixture was rejected: rejection={rejection:?}"
            )))
        }
    };
    let options = TranslateOptions::with_main_invocation(MainInvocation::LeaveIdle);
    match translate_module(&decoded, options) {
        TranslateOutcome::Translated(artifact) => Ok(artifact.into_text()),
        TranslateOutcome::Rejected(rejection) => Err(Error::other(format!(
            "arithmetic translation was rejected: rejection={rejection:?}"
        ))),
    }
}

#[rstest]
fn given_overflowing_i32_module_when_run_in_luau_then_results_wrap_like_webassembly(
) -> Result<(), Error> {
    // Given a module whose add, subtract, multiply, and chained operations overflow i32.
    let generated = generate_arithmetic_luau()?;
    let luau_path = official_luau_tool(("LUAU_BIN", "luau"))?;
    let temp_dir = tempfile::Builder::new()
        .prefix("luau-rs-i32-arithmetic-bdd")
        .tempdir()?;
    let source_path = temp_dir.path().join("driver.luau");
    let driver = format!(
        "local function make()\n{generated}\nend\n\
         local m = make()({{}})\n\
         assert(m.add(2147483647, 1) == -2147483648, \"i32 add overflow mismatch\")\n\
         assert(m.sub(-2147483648, 1) == 2147483647, \"i32 sub overflow mismatch\")\n\
         assert(m.mul(1073741824, 4) == 0, \"i32 mul overflow mismatch\")\n\
         assert(m.chain(2147483647, 1, 2) == 0, \"i32 chained overflow mismatch\")\n\
         assert(m.chain(-2147483648, -1, 2) == -2, \"i32 negative chain overflow mismatch\")\n"
    );
    fs_err::write(&source_path, &driver)?;

    // When official Luau evaluates the translated module.
    let result = Command::new(luau_path).arg(&source_path).output()?;

    // Then every wrapping result agrees with WebAssembly semantics.
    let success = result.status.success();
    if success {
        Ok(())
    } else {
        Err(Error::other(format!(
            "official Luau reported a wrapping mismatch: success={success}, stderr={}",
            String::from_utf8_lossy(&result.stderr)
        )))
    }
}
