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

For a multi-module project, `luau-rs` reads `luau-rs.toml`, discovers wasm
inputs recursively, validates the complete project in memory, and publishes a
fresh Roblox layout only after every module succeeds.

Because the frontend is wasm, you get **100% real Rust semantics** (borrow
checker, std library, crates.io) from `rustc` for free — the compiler only
translates a stable, well-defined instruction set into Luau. The Luau backend
models wasm's linear memory as a `buffer`, globals and indirect-call tables as
Luau tables, and wasm's stack machine as an explicit per-function stack.

Roblox bindings arrive through the **import seam** plus the bundled
`runtime/roblox.luau` binding layer: wasm imports become Luau callbacks
resolved from the `imports` table, and Rust receives Roblox objects as
integer handles (0 is null) that the runtime resolves to real Instances.

```bash
# Try the Roblox binding layer with a mock environment
cargo test --test roblox_bindings   # after python scripts/build_pinned_luau.py
```

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

The complete integration suite is written as Rust-native behavior scenarios
using `rstest`. Each case has an explicit Given/When/Then structure, so the
behavior and its evidence live together without a second feature-file DSL.
Scenario names follow `given_<context>_when_<action>_then_<outcome>` so the
test list itself explains the contract:

```bash
cargo test --all-targets
```

The scenarios use the existing `tempfile`, `assert_cmd`, `predicates`, and
`insta` crates for project setup, CLI execution, assertions, and snapshots.

## Commands

```text
cargo run -- build <module.wasm> --out <dir> [--entrypoint] [--side server|client] [--module-path <path>]
luau-rs check --manifest-path <path>
luau-rs compile --manifest-path <path>
```

The manifest defaults to `luau-rs.toml` when `--manifest-path` is omitted. Its
paths are relative to the manifest file:

```toml
[project]
source_root = "wasm"
output_root = "build"
```

Under `source_root`, modules use this convention:

```text
<source_root>/<server|client|shared>/<entrypoint|library>/<module-path>.wasm
```

For example, `wasm/server/entrypoint/game/main.wasm` becomes
`ServerScriptService/game/main.server.luau`, while
`wasm/shared/library/math/core.wasm` becomes
`ReplicatedStorage/math/core.luau`. `check` performs discovery, decoding, and
translation without writing output. `compile` stages every generated file and
atomically replaces the previous `output_root` only after the complete tree is
ready.

## Current scope

Supported: the full core wasm instruction set — numeric ops (i32/i64/f32/f64),
comparisons, conversions, locals, globals, memory load/store, **bulk memory
(copy/fill/init + passive data segments)**, `call` / `call_indirect`,
`block` / `loop` / `if` / `br` / `br_table`, `select`, `memory.grow`, data
segments, element segments, start function, exports, function imports.

The bundled `runtime/roblox.luau` provides a handle-based Roblox binding layer
(get service, instance creation, number + Vector3 properties, print, destroy),
tested against a mock Roblox environment.

Rejected loudly (typed reasons): SIMD (`v128`), atomics, memory imports,
exception tags, shared/64-bit memories, table/global exports, and
passive/declarative element segments.

`i64` values are represented exactly as two unsigned 32-bit Luau number
values. The pair representation is used across locals, globals, calls,
results, arithmetic, comparisons, bitwise operations, conversions, and
memory access. `f32` rounding is not modeled, and division-by-zero does not
trap.

See `CLAUDE.md` for the full architecture and agent guidance.
