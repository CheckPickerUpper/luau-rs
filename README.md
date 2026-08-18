# luau-rs

`luau-rs` is a **wasm → Luau compiler for Roblox**. Write real Rust, compile
it to wasm32, and luau-rs emits strict `--!strict` Luau that runs inside Roblox
Studio.

## How it works

```text
Rust source ──cargo build --target wasm32-unknown-unknown──▶ module.wasm
module.wasm ──luau-rs decode──▶ DecodedModule (validated subset)
DecodedModule ──luau-rs translate──▶ strict Luau module (instantiate factory)
```

Because the frontend is wasm, you get **100% real Rust semantics** (borrow
checker, std library, crates.io) from `rustc` for free — the compiler only
translates a stable, well-defined instruction set into Luau. The Luau backend
models wasm's linear memory as a `buffer`, globals and indirect-call tables as
Luau tables, and wasm's stack machine as an explicit per-function stack.

Roblox bindings (Instances, events, services) arrive through the **import
seam**: wasm imports become Luau callbacks resolved from the `imports` table
passed to the generated `instantiate(imports)` factory.

## Try it

```bash
# Build the fixture Rust crate to wasm and feed it through the compiler
cargo build --release --target wasm32-unknown-unknown \
    --manifest-path fixtures/rust-hello/Cargo.toml \
    --target-dir fixtures/rust-hello/target
cargo run -- build fixtures/rust-hello/target/wasm32-unknown-unknown/release/rust_hello.wasm \
    --out build --entrypoint
# → writes build/ServerScriptService/main.server.luau
```

## Validate with the official Luau tools

The integration suite treats the official `luau` / `luau-analyze` binaries as
required oracles (pinned revision, `bit32`-era, no native bitwise ops):

```bash
python scripts/build_pinned_luau.py     # one-time bootstrap
cargo test
```

## Commands

```text
cargo run -- build <module.wasm> --out <dir> [--entrypoint] [--side server|client] [--module-path <path>]
```

## Current scope

Supported: the full core wasm instruction set — numeric ops (i32/i64/f32/f64),
comparisons, conversions, locals, globals, memory load/store, `call` /
`call_indirect`, `block` / `loop` / `if` / `br` / `br_table`, `select`,
`memory.grow`, data segments, element segments, start function, exports,
function imports.

Rejected loudly (typed reasons): SIMD (`v128`), atomics, bulk memory, memory
imports, exception tags, shared/64-bit memories, table/global exports,
passive segments.

Documented approximations: `i64` beyond 53 bits (Luau numbers are doubles),
`i64` unsigned ops lower to signed, `f32` rounding is not modeled,
division-by-zero does not trap.

See `CLAUDE.md` for the full architecture and agent guidance.
