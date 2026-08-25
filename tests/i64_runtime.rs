//! Official-Luau proof for exact two-half i64 lowering.

mod support;

use luau_rs::{
    decode_module, translate_module, DecodeOutcome, MainInvocation, TranslateOptions,
    TranslateOutcome,
};
use std::io::Error;
use std::process::Command;
use support::official_luau_tool;
use walrus::ir::{BinaryOp, LoadKind, MemArg, StoreKind, UnaryOp, Value};
use walrus::{ConstExpr, ElementItems, ElementKind, FunctionBuilder, Module, RefType, ValType};

fn add_binary_i64(module: &mut Module, name: &str, operation: BinaryOp) {
    let left = module.locals.add(ValType::I64);
    let right = module.locals.add(ValType::I64);
    let mut builder = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I64, ValType::I64],
        &[ValType::I64],
    );
    builder
        .func_body()
        .local_get(left)
        .local_get(right)
        .binop(operation);
    let function = builder.finish(vec![left, right], &mut module.funcs);
    module.exports.add(name, function);
}

fn add_compare_i64(module: &mut Module, name: &str, operation: BinaryOp) {
    let left = module.locals.add(ValType::I64);
    let right = module.locals.add(ValType::I64);
    let mut builder = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I64, ValType::I64],
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

fn add_unary_i64(module: &mut Module, name: &str, operation: UnaryOp, result: ValType) {
    let value = module.locals.add(ValType::I64);
    let mut builder = FunctionBuilder::new(&mut module.types, &[ValType::I64], &[result]);
    builder.func_body().local_get(value).unop(operation);
    let function = builder.finish(vec![value], &mut module.funcs);
    module.exports.add(name, function);
}

fn add_const_binary_i64(
    module: &mut Module,
    name: &str,
    left: i64,
    right: i64,
    operation: BinaryOp,
) {
    let mut builder = FunctionBuilder::new(&mut module.types, &[], &[ValType::I64]);
    builder
        .func_body()
        .i64_const(left)
        .i64_const(right)
        .binop(operation);
    let function = builder.finish(Vec::new(), &mut module.funcs);
    module.exports.add(name, function);
}

fn i64_fixture_wasm() -> Vec<u8> {
    let mut module = Module::default();
    add_const_binary_i64(&mut module, "max_plus_one", -1, 1, BinaryOp::I64Add);
    add_const_binary_i64(&mut module, "zero_minus_one", 0, 1, BinaryOp::I64Sub);
    add_const_binary_i64(
        &mut module,
        "wide_mul",
        0x1_0000_0001,
        0x1_0000_0001,
        BinaryOp::I64Mul,
    );
    add_const_binary_i64(&mut module, "signed_div", -7, 3, BinaryOp::I64DivS);
    add_const_binary_i64(&mut module, "signed_rem", -7, 3, BinaryOp::I64RemS);
    add_const_binary_i64(&mut module, "unsigned_div", -1, 2, BinaryOp::I64DivU);
    add_const_binary_i64(&mut module, "unsigned_rem", -1, 2, BinaryOp::I64RemU);
    add_const_binary_i64(
        &mut module,
        "signed_overflow",
        i64::MIN,
        -1,
        BinaryOp::I64DivS,
    );

    add_binary_i64(&mut module, "add", BinaryOp::I64Add);
    add_binary_i64(&mut module, "bit_and", BinaryOp::I64And);
    add_binary_i64(&mut module, "bit_or", BinaryOp::I64Or);
    add_binary_i64(&mut module, "xor", BinaryOp::I64Xor);
    add_binary_i64(&mut module, "shl", BinaryOp::I64Shl);
    add_binary_i64(&mut module, "shr_s", BinaryOp::I64ShrS);
    add_binary_i64(&mut module, "shr_u", BinaryOp::I64ShrU);
    add_binary_i64(&mut module, "rotl", BinaryOp::I64Rotl);
    add_binary_i64(&mut module, "rotr", BinaryOp::I64Rotr);
    add_compare_i64(&mut module, "lt_u", BinaryOp::I64LtU);
    add_compare_i64(&mut module, "lt_s", BinaryOp::I64LtS);
    add_compare_i64(&mut module, "ge_u", BinaryOp::I64GeU);
    add_compare_i64(&mut module, "ge_s", BinaryOp::I64GeS);
    add_unary_i64(&mut module, "to_i32", UnaryOp::I32WrapI64, ValType::I32);

    let signed_value = module.locals.add(ValType::I32);
    let mut signed_builder =
        FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I64]);
    signed_builder
        .func_body()
        .local_get(signed_value)
        .unop(UnaryOp::I64ExtendSI32);
    let signed_function = signed_builder.finish(vec![signed_value], &mut module.funcs);
    module.exports.add("from_i32_s", signed_function);

    let unsigned_value = module.locals.add(ValType::I32);
    let mut unsigned_builder =
        FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I64]);
    unsigned_builder
        .func_body()
        .local_get(unsigned_value)
        .unop(UnaryOp::I64ExtendUI32);
    let unsigned_function = unsigned_builder.finish(vec![unsigned_value], &mut module.funcs);
    module.exports.add("from_i32_u", unsigned_function);

    let memory = module.memories.add_local(false, false, 1, None, None);
    let memory_value = module.locals.add(ValType::I64);
    let mut memory_builder =
        FunctionBuilder::new(&mut module.types, &[ValType::I64], &[ValType::I64]);
    memory_builder
        .func_body()
        .i32_const(0)
        .local_get(memory_value)
        .store(
            memory,
            StoreKind::I64 { atomic: false },
            MemArg {
                align: 8,
                offset: 0,
            },
        )
        .i32_const(0)
        .load(
            memory,
            LoadKind::I64 { atomic: false },
            MemArg {
                align: 8,
                offset: 0,
            },
        );
    let memory_function = memory_builder.finish(vec![memory_value], &mut module.funcs);
    module.exports.add("memory_round_trip", memory_function);

    let direct_value = module.locals.add(ValType::I64);
    let direct_type = module.types.add(&[ValType::I64], &[ValType::I64]);
    let mut direct_builder =
        FunctionBuilder::new(&mut module.types, &[ValType::I64], &[ValType::I64]);
    direct_builder
        .func_body()
        .local_get(direct_value)
        .i64_const(1)
        .binop(BinaryOp::I64Add);
    let direct_target = direct_builder.finish(vec![direct_value], &mut module.funcs);
    let call_value = module.locals.add(ValType::I64);
    let mut call_builder =
        FunctionBuilder::new(&mut module.types, &[ValType::I64], &[ValType::I64]);
    call_builder
        .func_body()
        .local_get(call_value)
        .call(direct_target);
    let call_function = call_builder.finish(vec![call_value], &mut module.funcs);
    module.exports.add("direct_call", call_function);

    let table = module.tables.add_local(false, 1, None, RefType::FUNCREF);
    module.elements.add(
        ElementKind::Active {
            table,
            offset: ConstExpr::Value(Value::I32(0)),
        },
        ElementItems::Functions(vec![direct_target]),
    );
    let indirect_value = module.locals.add(ValType::I64);
    let indirect_index = module.locals.add(ValType::I32);
    let mut indirect_builder = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I64, ValType::I32],
        &[ValType::I64],
    );
    indirect_builder
        .func_body()
        .local_get(indirect_value)
        .local_get(indirect_index)
        .call_indirect(direct_type, table);
    let indirect_function =
        indirect_builder.finish(vec![indirect_value, indirect_index], &mut module.funcs);
    module.exports.add("indirect_call", indirect_function);

    module.emit_wasm()
}

fn generated_i64_luau() -> Result<String, Error> {
    let decoded = match decode_module(&i64_fixture_wasm()) {
        DecodeOutcome::Decoded(decoded) => decoded,
        DecodeOutcome::Rejected(rejection) => {
            return Err(Error::other(format!(
                "i64 fixture was rejected: {rejection:?}"
            )))
        }
    };
    match translate_module(
        &decoded,
        TranslateOptions::with_main_invocation(MainInvocation::LeaveIdle),
    ) {
        TranslateOutcome::Translated(artifact) => Ok(artifact.into_text()),
        TranslateOutcome::Rejected(rejection) => Err(Error::other(format!(
            "i64 translation was rejected: {rejection:?}"
        ))),
    }
}

fn run_luau(source: &str, driver: &str, prefix: &str) -> Result<bool, Error> {
    let luau_path = official_luau_tool(("LUAU_BIN", "luau"))?;
    let temp_dir = tempfile::Builder::new().prefix(prefix).tempdir()?;
    let source_path = temp_dir.path().join("driver.luau");
    fs_err::write(
        &source_path,
        format!("local function make()\n{source}\nend\n{driver}"),
    )?;
    let output = Command::new(luau_path).arg(source_path).output()?;
    if !output.status.success() {
        eprintln!(
            "official Luau stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(output.status.success())
}

#[test]
fn given_i64_module_when_run_in_official_luau_then_exact_halves_survive_every_boundary(
) -> Result<(), Error> {
    let generated = generated_i64_luau()?;
    let driver = r#"local m = make()({})
local function assert_pair(name, fn, expected_low, expected_high)
    local low, high = fn()
    assert(low == expected_low and high == expected_high, name .. " mismatch")
end
assert_pair("max plus one", function() return m.max_plus_one() end, 0, 0)
assert_pair("zero minus one", function() return m.zero_minus_one() end, 4294967295, 4294967295)
assert_pair("wide multiply", function() return m.wide_mul() end, 1, 2)
assert_pair("signed division", function() return m.signed_div() end, 4294967294, 4294967295)
assert_pair("signed remainder", function() return m.signed_rem() end, 4294967295, 4294967295)
assert_pair("unsigned division", function() return m.unsigned_div() end, 4294967295, 2147483647)
assert_pair("unsigned remainder", function() return m.unsigned_rem() end, 1, 0)
assert_pair("direct call", function() return m.direct_call(4294967295, 0) end, 0, 1)
assert_pair("indirect call", function() return m.indirect_call(4294967295, 0, 0) end, 0, 1)
assert_pair("memory round trip", function() return m.memory_round_trip(4294967295, 2147483648) end, 4294967295, 2147483648)
assert(m.lt_u(4294967295, 0, 0, 1) == 1)
assert(m.lt_s(4294967295, 4294967295, 0, 0) == 1)
assert(m.ge_u(0, 1, 4294967295, 0) == 1)
assert(m.ge_s(0, 0, 4294967295, 4294967295) == 1)
assert_pair("bit and", function() return m.bit_and(4294967295, 4294967295, 65535, 0) end, 65535, 0)
assert_pair("bit or", function() return m.bit_or(0, 1, 4294967295, 0) end, 4294967295, 1)
assert_pair("xor", function() return m.xor(4294967295, 0, 4294967295, 4294967295) end, 0, 4294967295)
assert_pair("shift left", function() return m.shl(1, 0, 32, 0) end, 0, 1)
assert_pair("shift right signed", function() return m.shr_s(0, 2147483648, 32, 0) end, 2147483648, 4294967295)
assert_pair("shift right unsigned", function() return m.shr_u(0, 2147483648, 32, 0) end, 2147483648, 0)
assert_pair("rotate left", function() return m.rotl(1, 0, 32, 0) end, 0, 1)
assert_pair("rotate right", function() return m.rotr(1, 0, 32, 0) end, 0, 1)
assert_pair("signed i32 conversion", function() return m.from_i32_s(-1) end, 4294967295, 4294967295)
assert_pair("unsigned i32 conversion", function() return m.from_i32_u(-1) end, 4294967295, 0)
assert(m.to_i32(4294967295, 0) == -1)
"#;
    let status = run_luau(&generated, driver, "luau-rs-i64-runtime")?;
    if !status {
        return Err(Error::other(
            "official Luau rejected the exact i64 runtime proof",
        ));
    }

    let trap_driver = "local m = make()({})\nm.signed_overflow()\n";
    let trap_status = run_luau(&generated, trap_driver, "luau-rs-i64-trap")?;
    if trap_status {
        return Err(Error::other(
            "official Luau did not trap signed i64 division overflow",
        ));
    }
    Ok(())
}
