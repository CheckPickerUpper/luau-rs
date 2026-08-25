//! Wasm numeric, unary, and binary operations lowered to Luau expressions.

use walrus::ir::{BinaryOp, UnaryOp, Value};

use super::problem::TranslationProblemReason;
use super::writer::luau_number_literal;

/// Splits a wasm i64 constant into the unsigned halves used by generated Luau.
pub fn luau_i64_parts(value: i64) -> (String, String) {
    let bits = u64::from_ne_bytes(value.to_ne_bytes());
    let low = bits & 0xffff_ffff;
    let high = bits >> 32;
    (low.to_string(), high.to_string())
}

/// Renders a wasm constant as a Luau number literal.
pub fn luau_constant(value: Value) -> Result<String, TranslationProblemReason> {
    match value {
        Value::I32(value) => Ok(value.to_string()),
        Value::I64(value) => Ok(value.to_string()),
        Value::F32(value) => Ok(luau_number_literal(f64::from(value))),
        Value::F64(value) => Ok(luau_number_literal(value)),
        Value::V128(_) => Err(TranslationProblemReason::UnsupportedInstruction {
            instruction: "v128 constant".into(),
        }),
    }
}

/// Lowers one wasm unary operation to a Luau expression over an operand name.
pub fn unop_expression(op: UnaryOp, operand: &str) -> Result<String, TranslationProblemReason> {
    let expression = match op {
        UnaryOp::I32Eqz | UnaryOp::I64Eqz => format!("({operand} == 0)"),
        UnaryOp::I32Clz | UnaryOp::I64Clz => format!("wasm_i32_clz({operand})"),
        UnaryOp::I32Ctz | UnaryOp::I64Ctz => format!("wasm_i32_ctz({operand})"),
        UnaryOp::I32Popcnt | UnaryOp::I64Popcnt => format!("wasm_i32_popcnt({operand})"),
        UnaryOp::F32Abs | UnaryOp::F64Abs => format!("math.abs({operand})"),
        UnaryOp::F32Neg | UnaryOp::F64Neg => format!("(-{operand})"),
        UnaryOp::F32Ceil | UnaryOp::F64Ceil => format!("math.ceil({operand})"),
        UnaryOp::F32Floor | UnaryOp::F64Floor => format!("math.floor({operand})"),
        UnaryOp::F32Trunc | UnaryOp::F64Trunc => format!("wasm_trunc({operand})"),
        UnaryOp::F32Nearest | UnaryOp::F64Nearest => format!("wasm_nearest({operand})"),
        UnaryOp::F32Sqrt | UnaryOp::F64Sqrt => format!("math.sqrt({operand})"),
        UnaryOp::I32WrapI64
        | UnaryOp::I32Extend8S
        | UnaryOp::I32Extend16S
        | UnaryOp::I64Extend8S
        | UnaryOp::I64Extend16S
        | UnaryOp::I64Extend32S => format!("wasm_i32_wrap({operand})"),
        UnaryOp::I64ExtendSI32
        | UnaryOp::F32ConvertSI32
        | UnaryOp::F32ConvertSI64
        | UnaryOp::F32DemoteF64
        | UnaryOp::F64ConvertSI32
        | UnaryOp::F64ConvertSI64
        | UnaryOp::F64PromoteF32 => operand.to_string(),
        UnaryOp::I64ExtendUI32
        | UnaryOp::F32ConvertUI32
        | UnaryOp::F32ConvertUI64
        | UnaryOp::F64ConvertUI32
        | UnaryOp::F64ConvertUI64 => format!("wasm_i64_extend_u({operand})"),
        UnaryOp::I32TruncSF32 | UnaryOp::I32TruncSF64 => format!("wasm_i32_truncs({operand})"),
        UnaryOp::I32TruncUF32 | UnaryOp::I32TruncUF64 => format!("wasm_i32_truncu({operand})"),
        UnaryOp::I64TruncSF32 | UnaryOp::I64TruncSF64 => format!("wasm_i64_truncs({operand})"),
        UnaryOp::I64TruncUF32 | UnaryOp::I64TruncUF64 => format!("wasm_i64_truncu({operand})"),
        UnaryOp::I32ReinterpretF32 => format!("wasm_reinterpret_i32_f32({operand})"),
        UnaryOp::I64ReinterpretF64 => format!("wasm_reinterpret_i64_f64({operand})"),
        UnaryOp::F32ReinterpretI32 => format!("wasm_reinterpret_f32_i32({operand})"),
        UnaryOp::F64ReinterpretI64 => format!("wasm_reinterpret_f64_i64({operand})"),
        unsupported => {
            return Err(TranslationProblemReason::UnsupportedInstruction {
                instruction: format!("unary op {unsupported:?}"),
            });
        }
    };
    Ok(expression)
}

/// Lowers one wasm binary operation to a Luau expression over operand names.
pub fn binop_expression(
    op: BinaryOp,
    left: &str,
    right: &str,
) -> Result<String, TranslationProblemReason> {
    let expression = match op {
        BinaryOp::I32Eq | BinaryOp::I64Eq | BinaryOp::F32Eq | BinaryOp::F64Eq => {
            format!("({left} == {right})")
        }
        BinaryOp::I32Ne | BinaryOp::I64Ne | BinaryOp::F32Ne | BinaryOp::F64Ne => {
            format!("({left} ~= {right})")
        }
        BinaryOp::I32LtS
        | BinaryOp::I64LtS
        | BinaryOp::F32Lt
        | BinaryOp::F64Lt
        | BinaryOp::I64LtU
        | BinaryOp::I64GtU
        | BinaryOp::I64LeU
        | BinaryOp::I64GeU => format!("({left} < {right})"),
        BinaryOp::I32GtS | BinaryOp::I64GtS | BinaryOp::F32Gt | BinaryOp::F64Gt => {
            format!("({left} > {right})")
        }
        BinaryOp::I32LeS | BinaryOp::I64LeS | BinaryOp::F32Le | BinaryOp::F64Le => {
            format!("({left} <= {right})")
        }
        BinaryOp::I32GeS | BinaryOp::I64GeS | BinaryOp::F32Ge | BinaryOp::F64Ge => {
            format!("({left} >= {right})")
        }
        BinaryOp::I32LtU => format!("wasm_i32_ltu({left}, {right})"),
        BinaryOp::I32GtU => format!("wasm_i32_gtu({left}, {right})"),
        BinaryOp::I32LeU => format!("wasm_i32_leu({left}, {right})"),
        BinaryOp::I32GeU => format!("wasm_i32_geu({left}, {right})"),
        BinaryOp::I32Add => format!("wasm_i32_wrap(({left} + {right}))"),
        BinaryOp::I32Sub => format!("wasm_i32_wrap(({left} - {right}))"),
        BinaryOp::I32Mul => format!("wasm_i32_wrap(({left} * {right}))"),
        BinaryOp::I64Add | BinaryOp::F32Add | BinaryOp::F64Add => {
            format!("({left} + {right})")
        }
        BinaryOp::I64Sub | BinaryOp::F32Sub | BinaryOp::F64Sub => {
            format!("({left} - {right})")
        }
        BinaryOp::I64Mul | BinaryOp::F32Mul | BinaryOp::F64Mul => {
            format!("({left} * {right})")
        }
        BinaryOp::F32Div | BinaryOp::F64Div => format!("({left} / {right})"),
        BinaryOp::I32DivS | BinaryOp::I64DivS => format!("wasm_i32_divs({left}, {right})"),
        BinaryOp::I32DivU | BinaryOp::I64DivU => format!("wasm_i32_divu({left}, {right})"),
        BinaryOp::I32RemS | BinaryOp::I64RemS => format!("wasm_i32_rems({left}, {right})"),
        BinaryOp::I32RemU | BinaryOp::I64RemU => format!("wasm_i32_remu({left}, {right})"),
        BinaryOp::I32And => format!("bit32.band({left}, {right})"),
        BinaryOp::I64And => format!("({left} & {right})"),
        BinaryOp::I32Or => format!("bit32.bor({left}, {right})"),
        BinaryOp::I64Or => format!("({left} | {right})"),
        BinaryOp::I32Xor => format!("bit32.bxor({left}, {right})"),
        BinaryOp::I64Xor => format!("({left} ~ {right})"),
        BinaryOp::I32Shl => format!("bit32.lshift({left}, {right})"),
        BinaryOp::I64Shl => format!("({left} << {right})"),
        BinaryOp::I32ShrS => format!("bit32.arshift({left}, {right})"),
        BinaryOp::I64ShrS => format!("({left} >> {right})"),
        BinaryOp::I32ShrU | BinaryOp::I64ShrU => format!("wasm_shr_u({left}, {right})"),
        BinaryOp::I32Rotl | BinaryOp::I64Rotl => format!("wasm_i32_rotl({left}, {right})"),
        BinaryOp::I32Rotr | BinaryOp::I64Rotr => format!("wasm_i32_rotr({left}, {right})"),
        BinaryOp::F32Min | BinaryOp::F64Min => format!("math.min({left}, {right})"),
        BinaryOp::F32Max | BinaryOp::F64Max => format!("math.max({left}, {right})"),
        BinaryOp::F32Copysign | BinaryOp::F64Copysign => {
            format!("math.copysign({left}, {right})")
        }
        unsupported => {
            return Err(TranslationProblemReason::UnsupportedInstruction {
                instruction: format!("binary op {unsupported:?}"),
            });
        }
    };
    Ok(expression)
}
