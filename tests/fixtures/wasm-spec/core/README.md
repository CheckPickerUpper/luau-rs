# Official WebAssembly core-suite fixture

`i32-arithmetic.wast` is a bounded, checked-in slice of the official
WebAssembly core test suite. It is copied from
[`test/core/i32.wast`](https://github.com/WebAssembly/spec/blob/fc209c5ed8afc4dfeb9252024d217da3376c7a6f/test/core/i32.wast)
at source revision `fc209c5ed8afc4dfeb9252024d217da3376c7a6f`.

The slice covers signed i32 add, subtract, multiply, and divide, including
wraparound values and the official divide-by-zero and signed-overflow trap
assertions. It is intentionally not presented as the whole specification
suite: the current compiler can only execute the subset represented by this
fixture through its existing decode, translation, and pinned Luau runtime path.

`tests/wasm_spec.rs` parses this fixture with the `wast` parser, encodes each
module, sends it through `decode_module` and `translate_module`, and executes
the generated Luau with the pinned official Luau binary. The harness reports
passed, failed, and skipped-by-scope assertions separately. Return assertions
for unsupported shapes are skipped by scope; a trap assertion that cannot be
compiled or executed is a failure, never a pass.

At the current compiler revision this slice reports 17 passed and 4 failed:
the four failures are the deliberately exercised `div_s` trap assertions.
That red result is the regression signal for missing trap semantics, not a
reason to widen the fixture or count unsupported behavior as success.
