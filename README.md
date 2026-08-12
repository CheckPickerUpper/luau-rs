# luau-rs

`luau-rs` is an experimental Rust-like language compiler for Roblox Luau. It
currently supports strict Luau output for:

- typed numbers, strings, booleans, homogeneous zero-based arrays, and records;
- immutable and mutable locals, assignments, record-field and array updates;
- functions, public library functions, project imports, and Roblox service
  acquisition;
- arithmetic, comparisons, equality, boolean logic, `if`/`else`, `while`,
  `break`, and `continue`.

Every generated module begins with `--!strict`. The compiler library exposes
typed rejection reasons and preserves source byte ranges for diagnostics.

## Build the pinned Luau tools

The integration suite deliberately fails when the official Luau tools are not
available. From a shell with `git` and `make`, run:

```text
python scripts/build_pinned_luau.py
```

The command clones Luau at revision
`af6afddc651f3e8a272b1742d7f56695f9a9a278` into
`references/checkouts/luau`, then builds `luau`, `luau-analyze`, and
`luau-compile` with the same release command used by CI. The test suite finds
those binaries there, or you can set `LUAU_BIN`, `LUAU_ANALYZE_BIN`, and
`LUAU_COMPILE_BIN` explicitly.

## Run the suite

```text
cargo +stable fmt --check
cargo +stable clippy --all-targets --all-features --locked -- -D warnings
cargo +stable test --all-targets --locked
```

Run the Luau bootstrap command first on a new checkout. The official Luau
runtime, analyzer, and compiler are required test oracles rather than optional
test conveniences.
