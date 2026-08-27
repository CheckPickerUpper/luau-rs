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

local function wasm_i32_ltu(a: number, b: number): number
    local ua = a < 0 and a + 4294967296 or a
    local ub = b < 0 and b + 4294967296 or b
    return ua < ub and 1 or 0
end

local function wasm_i32_gtu(a: number, b: number): number
    local ua = a < 0 and a + 4294967296 or a
    local ub = b < 0 and b + 4294967296 or b
    return ua > ub and 1 or 0
end

local function wasm_i32_leu(a: number, b: number): number
    local ua = a < 0 and a + 4294967296 or a
    local ub = b < 0 and b + 4294967296 or b
    return ua <= ub and 1 or 0
end

local function wasm_i32_geu(a: number, b: number): number
    local ua = a < 0 and a + 4294967296 or a
    local ub = b < 0 and b + 4294967296 or b
    return ua >= ub and 1 or 0
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
local function wasm_i64_u32(v: number): number
    local r = v % 4294967296
    if r < 0 then
        return r + 4294967296
    end
    return r
end

local function wasm_i64_from_i32s(v: number): (number, number)
    local low = v
    if v < 0 then
        low = v + 4294967296
        return low, 4294967295
    end
    return low, 0
end

local function wasm_i64_from_i32u(v: number): (number, number)
    if v < 0 then
        return v + 4294967296, 0
    end
    return v, 0
end

local function wasm_i64_add(al: number, ah: number, bl: number, bh: number): (number, number)
    local low = al + bl
    local carry = 0
    if low >= 4294967296 then
        low = low - 4294967296
        carry = 1
    end
    return low, wasm_i64_u32(ah + bh + carry)
end

local function wasm_i64_sub(al: number, ah: number, bl: number, bh: number): (number, number)
    local low = al - bl
    local borrow = 0
    if low < 0 then
        low = low + 4294967296
        borrow = 1
    end
    return low, wasm_i64_u32(ah - bh - borrow)
end

local function wasm_i64_neg(low: number, high: number): (number, number)
    return wasm_i64_add(bit32.bnot(low), bit32.bnot(high), 1, 0)
end

local function wasm_i64_mul(al: number, ah: number, bl: number, bh: number): (number, number)
    local a0 = al % 65536
    local a1 = math.floor(al / 65536)
    local a2 = ah % 65536
    local a3 = math.floor(ah / 65536)
    local b0 = bl % 65536
    local b1 = math.floor(bl / 65536)
    local b2 = bh % 65536
    local b3 = math.floor(bh / 65536)
    local product = a0 * b0
    local r0 = product % 65536
    local carry = math.floor(product / 65536)
    product = a1 * b0 + a0 * b1 + carry
    local r1 = product % 65536
    carry = math.floor(product / 65536)
    product = a2 * b0 + a1 * b1 + a0 * b2 + carry
    local r2 = product % 65536
    carry = math.floor(product / 65536)
    product = a3 * b0 + a2 * b1 + a1 * b2 + a0 * b3 + carry
    local r3 = product % 65536
    return r0 + r1 * 65536, r2 + r3 * 65536
end

local function wasm_i64_unsigned_lt(al: number, ah: number, bl: number, bh: number): boolean
    return ah < bh or (ah == bh and al < bl)
end
local function wasm_i64_signed_lt(al: number, ah: number, bl: number, bh: number): boolean
    local aneg = ah >= 2147483648
    local bneg = bh >= 2147483648
    if aneg ~= bneg then return aneg end
    return wasm_i64_unsigned_lt(al, ah, bl, bh)
end
local function wasm_i64_compare(al: number, ah: number, bl: number, bh: number, signed: boolean): number
    if al == bl and ah == bh then return 0 end
    if signed then
        if wasm_i64_signed_lt(al, ah, bl, bh) then return -1 end
    elseif wasm_i64_unsigned_lt(al, ah, bl, bh) then
        return -1
    end
    return 1
end
local function wasm_i64_rel(al: number, ah: number, bl: number, bh: number, relation: number, signed: boolean): number
    local comparison = wasm_i64_compare(al, ah, bl, bh, signed)
    if relation == 0 and comparison == 0 then return 1 end
    if relation == 1 and comparison ~= 0 then return 1 end
    if relation == 2 and comparison < 0 then return 1 end
    if relation == 3 and comparison > 0 then return 1 end
    if relation == 4 and comparison <= 0 then return 1 end
    if relation == 5 and comparison >= 0 then return 1 end
    return 0
end
local function wasm_i64_eq(al: number, ah: number, bl: number, bh: number): number
    return wasm_i64_rel(al, ah, bl, bh, 0, false)
end
local function wasm_i64_ne(al: number, ah: number, bl: number, bh: number): number
    return wasm_i64_rel(al, ah, bl, bh, 1, false)
end
local function wasm_i64_lt_u(al: number, ah: number, bl: number, bh: number): number
    return wasm_i64_rel(al, ah, bl, bh, 2, false)
end
local function wasm_i64_gt_u(al: number, ah: number, bl: number, bh: number): number
    return wasm_i64_rel(al, ah, bl, bh, 3, false)
end
local function wasm_i64_le_u(al: number, ah: number, bl: number, bh: number): number
    return wasm_i64_rel(al, ah, bl, bh, 4, false)
end
local function wasm_i64_ge_u(al: number, ah: number, bl: number, bh: number): number
    return wasm_i64_rel(al, ah, bl, bh, 5, false)
end
local function wasm_i64_lt_s(al: number, ah: number, bl: number, bh: number): number
    return wasm_i64_rel(al, ah, bl, bh, 2, true)
end
local function wasm_i64_gt_s(al: number, ah: number, bl: number, bh: number): number
    return wasm_i64_rel(al, ah, bl, bh, 3, true)
end
local function wasm_i64_le_s(al: number, ah: number, bl: number, bh: number): number
    return wasm_i64_rel(al, ah, bl, bh, 4, true)
end
local function wasm_i64_ge_s(al: number, ah: number, bl: number, bh: number): number
    return wasm_i64_rel(al, ah, bl, bh, 5, true)
end

local function wasm_i64_and(al: number, ah: number, bl: number, bh: number): (number, number)
    return wasm_i64_u32(bit32.band(al, bl)), wasm_i64_u32(bit32.band(ah, bh))
end
local function wasm_i64_or(al: number, ah: number, bl: number, bh: number): (number, number)
    return wasm_i64_u32(bit32.bor(al, bl)), wasm_i64_u32(bit32.bor(ah, bh))
end
local function wasm_i64_xor(al: number, ah: number, bl: number, bh: number): (number, number)
    return wasm_i64_u32(bit32.bxor(al, bl)), wasm_i64_u32(bit32.bxor(ah, bh))
end

local function wasm_i64_shl(al: number, ah: number, bl: number): (number, number)
    local shift = bl % 64
    if shift == 0 then return al, ah end
    if shift < 32 then
        return wasm_i64_u32(bit32.lshift(al, shift)),
            wasm_i64_u32(bit32.lshift(ah, shift) + bit32.rshift(al, 32 - shift))
    end
    return 0, wasm_i64_u32(bit32.lshift(al, shift - 32))
end
local function wasm_i64_shr_u(al: number, ah: number, bl: number): (number, number)
    local shift = bl % 64
    if shift == 0 then return al, ah end
    if shift < 32 then
        return wasm_i64_u32(bit32.rshift(al, shift) + bit32.lshift(ah, 32 - shift)),
            bit32.rshift(ah, shift)
    end
    return bit32.rshift(ah, shift - 32), 0
end
local function wasm_i64_shr_s(al: number, ah: number, bl: number): (number, number)
    local shift = bl % 64
    if shift == 0 then return al, ah end
    if shift < 32 then
        return wasm_i64_u32(bit32.rshift(al, shift) + bit32.lshift(ah, 32 - shift)),
            wasm_i64_u32(bit32.arshift(ah, shift))
    end
    local high = 0
    if ah >= 2147483648 then high = 4294967295 end
    return wasm_i64_u32(bit32.arshift(ah, shift - 32)), high
end
local function wasm_i64_rotl(al: number, ah: number, bl: number): (number, number)
    local shift = bl % 64
    if shift == 0 then return al, ah end
    if shift < 32 then
        return wasm_i64_u32(bit32.lshift(al, shift) + bit32.rshift(ah, 32 - shift)),
            wasm_i64_u32(bit32.lshift(ah, shift) + bit32.rshift(al, 32 - shift))
    end
    if shift == 32 then return ah, al end
    return wasm_i64_rotl(ah, al, shift - 32)
end
local function wasm_i64_rotr(al: number, ah: number, bl: number): (number, number)
    local shift = bl % 64
    if shift == 0 then return al, ah end
    return wasm_i64_rotl(al, ah, 64 - shift)
end

local function wasm_i64_eqz(al: number, ah: number): number
    if al == 0 and ah == 0 then return 1 end
    return 0
end
local function wasm_i64_clz(al: number, ah: number): number
    if ah ~= 0 then return wasm_i32_clz(ah) end
    return 32 + wasm_i32_clz(al)
end
local function wasm_i64_ctz(al: number, ah: number): number
    if al ~= 0 then return wasm_i32_ctz(al) end
    return 32 + wasm_i32_ctz(ah)
end
local function wasm_i64_popcnt(al: number, ah: number): number
    return wasm_i32_popcnt(al) + wasm_i32_popcnt(ah)
end

local function wasm_i64_divmod_u(al: number, ah: number, bl: number, bh: number): (number, number, number, number)
    if bl == 0 and bh == 0 then error("wasm trap: integer divide by zero") end
    local ql, qh, rl, rh = 0, 0, 0, 0
    for bit = 63, 0, -1 do
        local incoming = 0
        if bit >= 32 then
            incoming = bit32.rshift(bit32.band(ah, bit32.lshift(1, bit - 32)), bit - 32)
        else
            incoming = bit32.rshift(bit32.band(al, bit32.lshift(1, bit)), bit)
        end
        local carry = bit32.rshift(rl, 31)
        rl = wasm_i64_u32(bit32.lshift(rl, 1) + incoming)
        rh = wasm_i64_u32(bit32.lshift(rh, 1) + carry)
        if not wasm_i64_unsigned_lt(rl, rh, bl, bh) then
            rl, rh = wasm_i64_sub(rl, rh, bl, bh)
            if bit >= 32 then
                qh = wasm_i64_u32(qh + 2 ^ (bit - 32))
            else
                ql = wasm_i64_u32(ql + 2 ^ bit)
            end
        end
    end
    return ql, qh, rl, rh
end
local function wasm_i64_div_u(al: number, ah: number, bl: number, bh: number): (number, number)
    local ql, qh = wasm_i64_divmod_u(al, ah, bl, bh)
    return ql, qh
end
local function wasm_i64_rem_u(al: number, ah: number, bl: number, bh: number): (number, number)
    local _, _, rl, rh = wasm_i64_divmod_u(al, ah, bl, bh)
    return rl, rh
end
local function wasm_i64_div_s(al: number, ah: number, bl: number, bh: number): (number, number)
    if al == 0 and ah == 2147483648 and bl == 4294967295 and bh == 4294967295 then
        error("wasm trap: integer overflow")
    end
    local aneg = ah >= 2147483648
    local bneg = bh >= 2147483648
    if aneg then al, ah = wasm_i64_neg(al, ah) end
    if bneg then bl, bh = wasm_i64_neg(bl, bh) end
    local ql, qh = wasm_i64_div_u(al, ah, bl, bh)
    if aneg ~= bneg then return wasm_i64_neg(ql, qh) end
    return ql, qh
end
local function wasm_i64_rem_s(al: number, ah: number, bl: number, bh: number): (number, number)
    local aneg = ah >= 2147483648
    local bneg = bh >= 2147483648
    if aneg then al, ah = wasm_i64_neg(al, ah) end
    if bneg then bl, bh = wasm_i64_neg(bl, bh) end
    local _, _, rl, rh = wasm_i64_divmod_u(al, ah, bl, bh)
    if aneg then return wasm_i64_neg(rl, rh) end
    return rl, rh
end

local function wasm_i64_truncs_pair(v: number): (number, number)
    local t = wasm_trunc(v)
    if t ~= t or t < -9223372036854775808 or t >= 9223372036854775808 then
        error("wasm trap: integer overflow")
    end
    if t < 0 then t = t + 18446744073709551616 end
    local high = math.floor(t / 4294967296)
    return t - high * 4294967296, high
end
local function wasm_i64_truncu_pair(v: number): (number, number)
    local t = wasm_trunc(v)
    if t ~= t or t < 0 or t >= 18446744073709551616 then
        error("wasm trap: integer overflow")
    end
    local high = math.floor(t / 4294967296)
    return t - high * 4294967296, high
end
local function wasm_i64_to_f64s(low: number, high: number): number
    local value = high * 4294967296 + low
    if high >= 2147483648 then return value - 18446744073709551616 end
    return value
end
local function wasm_i64_to_f64u(low: number, high: number): number
    return high * 4294967296 + low
end
local function wasm_i64_extend8s(low: number): (number, number)
    local value = bit32.band(low, 255)
    if value >= 128 then value = value - 256 end
    return wasm_i64_from_i32s(value)
end
local function wasm_i64_extend16s(low: number): (number, number)
    local value = bit32.band(low, 65535)
    if value >= 32768 then value = value - 65536 end
    return wasm_i64_from_i32s(value)
end
local function wasm_i64_extend32s(low: number): (number, number)
    return wasm_i64_from_i32s(wasm_i32_wrap(low))
end
local function wasm_reinterpret_i64_f64_pair(v: number): (number, number)
    local b = buffer.create(8)
    buffer.writef64(b, 0, v)
    return buffer.readu32(b, 0), buffer.readu32(b, 4)
end
local function wasm_reinterpret_f64_i64_pair(low: number, high: number): number
    local b = buffer.create(8)
    buffer.writeu32(b, 0, low)
    buffer.writeu32(b, 4, high)
    return buffer.readf64(b, 0)
end
-- i64 helpers follow.
"#;
