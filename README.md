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

### Known emission cost

The emitter models the wasm operand stack as a runtime `stack` table indexed by
a moving `sp`, and lowers wasm branches into nested `while true do` loops
guarded by `exit_N` boolean flags. Both are correct and both are slow. In the
7,511-line fixture snapshot that is 2,619 `stack[sp]` accesses, 1,214 `sp += 1`
updates, 82 `while true do` loops, and 417 `exit_N` flag references. A single
`i32.sub` currently costs six table operations and four `sp` updates:

```luau
sp += 1; stack[sp] = l1
sp += 1; stack[sp] = l2
local t21 = stack[sp]; sp -= 1
local t22 = stack[sp]; sp -= 1
sp += 1; stack[sp] = wasm_i32_wrap((t22 - t21))
local t23 = stack[sp]; sp -= 1
```

Every value here has a static stack slot, so the table is avoidable: the same
operation is one local binding once slots are resolved to named locals at
translation time. Luau's 200-register-per-function limit is the reason this is
a real design question rather than a mechanical substitution — deep functions
need spilling, which is why the table exists today. This is the largest known
throughput gap against the prior art below.

## Prior art

Researched 2026-08-27; every claim links to its source.

- **[Wasynth](https://github.com/SovereignSatellite/Wasynth)** — the reference
  wasm-to-Luau translator, now archived. It did not stop over a wasm or
  performance wall; the author's note cites "accumulating technical debt and
  the rigid design that makes work on it difficult", and names
  **[Spider](https://github.com/SovereignSatellite/Spider)** (RVSDG-based,
  wasm to Luau and LuaJIT) as its successor. Spider is actively developed.
- **[wasm2lua](https://github.com/swadicalrag/wasm2lua)** — stalled since
  December 2021, before Luau gained the `buffer` type that makes linear memory
  cheap.
- **[reasm](https://github.com/AsynchronousAI/reasm)** — compiles RISC-V rather
  than wasm to Luau. Its author reports moving from roughly half native Luau
  speed to a large speedup after switching to `buffer` operations, which is the
  clearest evidence that raw throughput is no longer the blocker.
- Rust-side efforts are dormant rather than defeated:
  [roblox-rs](https://github.com/DunnoConz/roblox-rs) (v0.1.0) and
  [LoganDark/luau-rs](https://github.com/LoganDark/luau-rs) (v0.0.1, last
  touched 2023).

### Why the `i64` representation is the pivotal choice

Luau numbers are IEEE doubles, so wasm's `i64` has no native carrier. Wasynth
stores each one as a `Vector3` packing two 32-bit halves, making every 64-bit
add, shift, multiply and divide a Luau function call that unpacks and repacks
its operands. Luau's
[long-integer RFC](https://rfcs.luau.org/type-long-integer.html) names this
directly: high-low pairs are "cumbersome, requiring 16-24 bytes per number",
vector storage "lacks type system integration", and "the restriction to doubles
will be an increasing pain point". That RFC has not shipped, and reasm defers
64-bit support until it does.

This project instead carries `i64` as an exact pair of unsigned 32-bit values
through every operation, which is why the pair representation is stated as
scope above rather than left as an implementation detail.

### What actually costs speed

Two distinct costs get conflated as "it is slow", and they bite in different
phases:

- **Startup and size.** Generated modules are large — Roblox developers
  describe Wasynth output as
  "[huge and i cant understand any of it](https://devforum.roblox.com/t/how-do-i-use-converted-wasm-files-to-luau-in-roblox/2807029)".
  That is load time, script size limits, and memory, not throughput. A DOOM
  port needed 4MB of buffer allocated inside Luau.
- **Steady-state throughput.** The
  "[horrid mass of `while true do` loops](https://devforum.roblox.com/t/how-do-i-use-converted-wasm-files-to-luau-in-roblox/2807029)"
  is not only a readability complaint. Luau has no `goto`, so every wasm branch
  out of a nested block sets one flag per enclosing level and each level
  re-tests them on the way out. That is real work in the hot path, and it is
  the same pattern this emitter produces.

See `CLAUDE.md` for the full architecture and agent guidance.
