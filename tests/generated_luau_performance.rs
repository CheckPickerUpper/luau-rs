//! Measured Luau loops for the generated-module and handwritten baselines.

mod support;

use luau_rs::{
    decode_module, translate_module, DecodeOutcome, MainInvocation, TranslateOptions,
    TranslateOutcome,
};
use rstest::rstest;
use std::io::{Error, Write};
use std::process::Command;
use support::official_luau_tool;
use walrus::ir::{BinaryOp, Block, Instr, Loop};
use walrus::{FunctionBuilder, FunctionId, Module, ValType};

const PERFORMANCE_ROBLOX_ENVIRONMENT: &str = r"
local env = {
    game = { GetService = function(self, name) return nil end },
    workspace = {},
    Instance = {
        new = function(class_name)
            return { ClassName = class_name, Anchored = 0 }
        end,
    },
    Vector3 = { new = function(x, y, z) return { x = x, y = y, z = z } end },
    print = function(...) end,
}
";

fn add_property_write_loop(module: &mut Module, property_setter: FunctionId) {
    let instance_handle = module.locals.add(ValType::I32);
    let property_pointer = module.locals.add(ValType::I32);
    let remaining_iterations = module.locals.add(ValType::I32);
    let property_value = module.locals.add(ValType::F64);
    let mut builder = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32, ValType::I32],
        &[ValType::F64],
    );

    builder.func_body().f64_const(1.0).local_set(property_value);
    let block_id;
    {
        let block_body = builder.dangling_instr_seq(None);
        block_id = block_body.id();
    }
    let loop_id;
    {
        let mut loop_body = builder.dangling_instr_seq(None);
        loop_id = loop_body.id();
        loop_body
            .local_get(instance_handle)
            .local_get(property_pointer)
            .local_get(property_value)
            .call(property_setter)
            .local_get(property_value)
            .f64_const(1.0)
            .binop(BinaryOp::F64Add)
            .local_set(property_value)
            .local_get(remaining_iterations)
            .i32_const(1)
            .binop(BinaryOp::I32Sub)
            .local_tee(remaining_iterations)
            .br_if(loop_id)
            .br(block_id);
    }
    {
        let mut block_body = builder.instr_seq(block_id);
        block_body.instr(Instr::Loop(Loop { seq: loop_id }));
    }
    builder
        .func_body()
        .instr(Instr::Block(Block { seq: block_id }))
        .local_get(property_value);
    let function = builder.finish(
        vec![instance_handle, property_pointer, remaining_iterations],
        &mut module.funcs,
    );
    module.exports.add("property_write_loop", function);
}

fn add_arithmetic_loop(module: &mut Module) {
    let remaining_iterations = module.locals.add(ValType::I32);
    let accumulator = module.locals.add(ValType::I32);
    let mut builder = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);

    builder.func_body().i32_const(0).local_set(accumulator);
    let block_id;
    {
        let block_body = builder.dangling_instr_seq(None);
        block_id = block_body.id();
    }
    let loop_id;
    {
        let mut loop_body = builder.dangling_instr_seq(None);
        loop_id = loop_body.id();
        loop_body
            .local_get(accumulator)
            .i32_const(3)
            .binop(BinaryOp::I32Add)
            .local_set(accumulator)
            .local_get(accumulator)
            .i32_const(1)
            .binop(BinaryOp::I32Sub)
            .local_set(accumulator)
            .local_get(remaining_iterations)
            .i32_const(1)
            .binop(BinaryOp::I32Sub)
            .local_tee(remaining_iterations)
            .br_if(loop_id)
            .br(block_id);
    }
    {
        let mut block_body = builder.instr_seq(block_id);
        block_body.instr(Instr::Loop(Loop { seq: loop_id }));
    }
    builder
        .func_body()
        .instr(Instr::Block(Block { seq: block_id }))
        .local_get(accumulator);
    let function = builder.finish(vec![remaining_iterations], &mut module.funcs);
    module.exports.add("arithmetic_loop", function);
}

fn performance_fixture_wasm() -> Vec<u8> {
    let mut module = Module::default();
    let memory = module.memories.add_local(false, false, 1, None, None);
    module.exports.add("memory", memory);
    let setter_type = module
        .types
        .add(&[ValType::I32, ValType::I32, ValType::F64], &[]);
    let (property_setter, _) = module.add_import_func("roblox", "roblox_set_property", setter_type);
    add_property_write_loop(&mut module, property_setter);
    add_arithmetic_loop(&mut module);
    module.emit_wasm()
}

fn generated_performance_luau() -> Result<String, Error> {
    let decoded = match decode_module(&performance_fixture_wasm()) {
        DecodeOutcome::Decoded(decoded) => decoded,
        DecodeOutcome::Rejected(rejection) => {
            return Err(Error::other(format!(
                "performance fixture was rejected: rejection={rejection:?}"
            )))
        }
    };
    match translate_module(
        &decoded,
        TranslateOptions::with_main_invocation(MainInvocation::LeaveIdle),
    ) {
        TranslateOutcome::Translated(artifact) => Ok(artifact.into_text()),
        TranslateOutcome::Rejected(rejection) => Err(Error::other(format!(
            "performance translation was rejected: rejection={rejection:?}"
        ))),
    }
}

fn performance_driver(generated: &str, runtime: &str) -> String {
    format!(
        r#"local function make_generated()
{generated}
end
local function make_runtime()
{runtime}
end
local RobloxRuntime = make_runtime()
{PERFORMANCE_ROBLOX_ENVIRONMENT}
local runtime = RobloxRuntime.new(env)
local module = make_generated()(runtime.imports)
runtime:bind_memory(module.memory)
local property_name = "Anchored"
for index = 1, #property_name do
    buffer.writeu8(module.memory, index - 1, string.byte(property_name, index))
end
buffer.writeu8(module.memory, #property_name, 0)
local part = env.Instance.new("Part")
local handle = runtime:handle_for(part)
local property_iterations = 20000000
local arithmetic_iterations = 25000000
local function handwritten_property_write(instance, iterations)
    local value = 1
    for _ = 1, iterations do
        instance.Anchored = value
        value += 1
    end
    return value
end
local function handwritten_arithmetic_loop(iterations)
    local accumulator = 0
    for _ = 1, iterations do
        accumulator += 3
        accumulator -= 1
    end
    return accumulator
end
local generated_property_start = os.clock()
local generated_property_result = module.property_write_loop(handle, 0, property_iterations)
local generated_property_seconds = os.clock() - generated_property_start
local handwritten_property_start = os.clock()
local handwritten_property_result = handwritten_property_write(part, property_iterations)
local handwritten_property_seconds = os.clock() - handwritten_property_start
assert(generated_property_result == handwritten_property_result, "property loop results diverged")
assert(part.Anchored == property_iterations, "property loop final value mismatch")
local generated_arithmetic_start = os.clock()
local generated_arithmetic_result = module.arithmetic_loop(arithmetic_iterations)
local generated_arithmetic_seconds = os.clock() - generated_arithmetic_start
local handwritten_arithmetic_start = os.clock()
local handwritten_arithmetic_result = handwritten_arithmetic_loop(arithmetic_iterations)
local handwritten_arithmetic_seconds = os.clock() - handwritten_arithmetic_start
assert(generated_arithmetic_result == handwritten_arithmetic_result, "arithmetic loop results diverged")
print(string.format("property_write generated=%.6f baseline=%.6f ratio=%.3f iterations=%d", generated_property_seconds, handwritten_property_seconds, generated_property_seconds / handwritten_property_seconds, property_iterations))
print(string.format("arithmetic generated=%.6f baseline=%.6f ratio=%.3f iterations=%d", generated_arithmetic_seconds, handwritten_arithmetic_seconds, generated_arithmetic_seconds / handwritten_arithmetic_seconds, arithmetic_iterations))
"#
    )
}

#[rstest]
fn given_generated_and_handwritten_luau_loops_when_run_then_results_match_and_ratios_are_printed(
) -> Result<(), Error> {
    // Given generated loops, handwritten baselines, and the Roblox binding runtime.
    let generated = generated_performance_luau()?;
    let runtime_path =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("runtime/roblox.luau");
    let runtime = fs_err::read_to_string(&runtime_path).map_err(|error| {
        Error::new(
            error.kind(),
            format!("could not read {}: {error}", runtime_path.display()),
        )
    })?;
    let luau = official_luau_tool(("LUAU_BIN", "luau"))?;
    let temp_dir = tempfile::Builder::new()
        .prefix("luau-rs-performance-bdd")
        .tempdir()?;
    let source_path = temp_dir.path().join("driver.luau");
    let driver = performance_driver(&generated, &runtime);
    fs_err::write(&source_path, &driver)?;

    // When official Luau measures each generated loop and its plain Luau baseline.
    let result = Command::new(luau).arg(&source_path).output()?;

    // Then both generated functions agree with their independent baselines and report timings.
    let success = result.status.success();
    if success {
        let stdout = String::from_utf8_lossy(&result.stdout);
        let mut output = std::io::stdout().lock();
        output.write_all(&result.stdout)?;
        output.flush()?;
        assert!(
            stdout.contains("property_write generated="),
            "property-write ratio was not printed: stdout={stdout}"
        );
        assert!(
            stdout.contains("arithmetic generated="),
            "arithmetic ratio was not printed: stdout={stdout}"
        );
        Ok(())
    } else {
        Err(Error::other(format!(
            "Luau performance scenario failed: success={success}, stdout={}, stderr={}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        )))
    }
}
