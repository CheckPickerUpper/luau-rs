;; This fixture is copied from the official WebAssembly core suite's i32.wast.
;; Keep the source revision and selected assertions documented alongside it.

(module
  (func (export "add") (param $x i32) (param $y i32) (result i32) (i32.add (local.get $x) (local.get $y)))
  (func (export "sub") (param $x i32) (param $y i32) (result i32) (i32.sub (local.get $x) (local.get $y)))
  (func (export "mul") (param $x i32) (param $y i32) (result i32) (i32.mul (local.get $x) (local.get $y)))
  (func (export "div_s") (param $x i32) (param $y i32) (result i32) (i32.div_s (local.get $x) (local.get $y)))
)

(assert_return (invoke "add" (i32.const 1) (i32.const 1)) (i32.const 2))
(assert_return (invoke "add" (i32.const -1) (i32.const 1)) (i32.const 0))
(assert_return (invoke "add" (i32.const 0x7fffffff) (i32.const 1)) (i32.const 0x80000000))
(assert_return (invoke "add" (i32.const 0x80000000) (i32.const -1)) (i32.const 0x7fffffff))

(assert_return (invoke "sub" (i32.const 1) (i32.const 1)) (i32.const 0))
(assert_return (invoke "sub" (i32.const 0x7fffffff) (i32.const -1)) (i32.const 0x80000000))
(assert_return (invoke "sub" (i32.const 0x80000000) (i32.const 1)) (i32.const 0x7fffffff))
(assert_return (invoke "sub" (i32.const 0x80000000) (i32.const 0x80000000)) (i32.const 0))

(assert_return (invoke "mul" (i32.const 1) (i32.const 1)) (i32.const 1))
(assert_return (invoke "mul" (i32.const -1) (i32.const -1)) (i32.const 1))
(assert_return (invoke "mul" (i32.const 0x10000000) (i32.const 4096)) (i32.const 0))
(assert_return (invoke "mul" (i32.const 0x80000000) (i32.const -1)) (i32.const 0x80000000))
(assert_return (invoke "mul" (i32.const 0x7fffffff) (i32.const -1)) (i32.const 0x80000001))
(assert_return (invoke "mul" (i32.const 0x01234567) (i32.const 0x76543210)) (i32.const 0x358e7470))

;; These trap assertions must fail until the generated helper models the
;; specification's divide-by-zero and signed-overflow traps.
(assert_trap (invoke "div_s" (i32.const 1) (i32.const 0)) "integer divide by zero")
(assert_trap (invoke "div_s" (i32.const 0) (i32.const 0)) "integer divide by zero")
(assert_trap (invoke "div_s" (i32.const 0x80000000) (i32.const -1)) "integer overflow")
(assert_trap (invoke "div_s" (i32.const 0x80000000) (i32.const 0)) "integer divide by zero")
(assert_return (invoke "div_s" (i32.const 1) (i32.const 1)) (i32.const 1))
(assert_return (invoke "div_s" (i32.const -5) (i32.const 2)) (i32.const -2))
(assert_return (invoke "div_s" (i32.const 5) (i32.const -2)) (i32.const -2))
