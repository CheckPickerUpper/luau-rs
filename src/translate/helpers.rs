//! Fixed Luau helper functions emitted at the top of every generated module.
//!
//! The helpers exist because Luau numbers are IEEE doubles while wasm programs
//! expect 32-bit integers with wrapping and truncating semantics. The pinned
//! Luau revision predates native bitwise operators, so the helpers use the
//! `bit32` library, which wraps to 32-bit and returns unsigned values.

/// The complete helper preamble, always emitted so generated output is stable.
pub const HELPER_SOURCE: &str = r#"-- Fixed helpers for wasm semantics on Luau numbers.
local function wasm_trunc(v: number): number
    if v < 0 then
        return math.ceil(v)
    end
    return math.floor(v)
end

local function wasm_nearest(v: number): number
    return math.floor(v + 0.5)
end

local function wasm_i32_wrap(v: number): number
    local r = v % 4294967296
    if r >= 2147483648 then
        return r - 4294967296
    end
    return r
end

local function wasm_i64_extend_u(v: number): number
    if v < 0 then
        return v + 4294967296
    end
    return v
end

local function wasm_i32_divs(a: number, b: number): number
    local q = a / b
    if q < 0 then
        return wasm_i32_wrap(math.ceil(q))
    end
    return wasm_i32_wrap(math.floor(q))
end

local function wasm_i32_rems(a: number, b: number): number
    local r = a % b
    if r ~= 0 and (r < 0) ~= (a < 0) then
        return r + b
    end
    return r
end

local function wasm_i32_ltu(a: number, b: number): boolean
    local ua = a < 0 and a + 4294967296 or a
    local ub = b < 0 and b + 4294967296 or b
    return ua < ub
end

local function wasm_i32_gtu(a: number, b: number): boolean
    local ua = a < 0 and a + 4294967296 or a
    local ub = b < 0 and b + 4294967296 or b
    return ua > ub
end

local function wasm_i32_leu(a: number, b: number): boolean
    local ua = a < 0 and a + 4294967296 or a
    local ub = b < 0 and b + 4294967296 or b
    return ua <= ub
end

local function wasm_i32_geu(a: number, b: number): boolean
    local ua = a < 0 and a + 4294967296 or a
    local ub = b < 0 and b + 4294967296 or b
    return ua >= ub
end

local function wasm_i32_divu(a: number, b: number): number
    local ua = a < 0 and a + 4294967296 or a
    local ub = b < 0 and b + 4294967296 or b
    return wasm_i32_wrap(math.floor(ua / ub))
end

local function wasm_i32_remu(a: number, b: number): number
    local ua = a < 0 and a + 4294967296 or a
    local ub = b < 0 and b + 4294967296 or b
    return wasm_i32_wrap(ua % ub)
end

local function wasm_shr_u(v: number, n: number): number
    n = n % 32
    if n == 0 then
        return v
    end
    return bit32.band(bit32.arshift(v, n), bit32.arshift(2147483647, n - 1))
end

local function wasm_i32_rotl(v: number, s: number): number
    return bit32.lrotate(v, s)
end

local function wasm_i32_rotr(v: number, s: number): number
    return bit32.rrotate(v, s)
end

local function wasm_i32_clz(v: number): number
    return bit32.countlz(v)
end

local function wasm_i32_ctz(v: number): number
    return bit32.countrz(v)
end

local function wasm_i32_popcnt(v: number): number
    local n = 0
    while v ~= 0 do
        v = bit32.band(v, v - 1)
        n = n + 1
    end
    return n
end

local function wasm_i32_truncs(v: number): number
    local t = wasm_trunc(v)
    if t ~= t or t < -2147483648 or t > 2147483647 then
        error("wasm trap: integer overflow")
    end
    return t
end

local function wasm_i32_truncu(v: number): number
    local t = wasm_trunc(v)
    if t ~= t or t < 0 or t > 4294967295 then
        error("wasm trap: integer overflow")
    end
    return wasm_i32_wrap(t)
end

local function wasm_i64_truncs(v: number): number
    local t = wasm_trunc(v)
    if t ~= t then
        error("wasm trap: integer overflow")
    end
    return t
end

local function wasm_i64_truncu(v: number): number
    local t = wasm_trunc(v)
    if t ~= t or t < 0 then
        error("wasm trap: integer overflow")
    end
    return t
end

local function wasm_reinterpret_i32_f32(v: number): number
    local b = buffer.create(4)
    buffer.writef32(b, 0, v)
    return buffer.readi32(b, 0)
end

local function wasm_reinterpret_f32_i32(v: number): number
    local b = buffer.create(4)
    buffer.writei32(b, 0, v)
    return buffer.readf32(b, 0)
end

local function wasm_reinterpret_i64_f64(v: number): number
    local b = buffer.create(8)
    buffer.writef64(b, 0, v)
    return buffer.readf64(b, 0)
end

local function wasm_reinterpret_f64_i64(v: number): number
    local b = buffer.create(8)
    buffer.writef64(b, 0, v)
    return buffer.readf64(b, 0)
end
"#;
