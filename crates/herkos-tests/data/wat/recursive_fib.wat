(module
  (func $fib (param i32) (result i32)
    (if (result i32) (i32.lt_s (local.get 0) (i32.const 2))
      (then (local.get 0))
      (else
        (i32.add
          (call 0 (i32.sub (local.get 0) (i32.const 1)))
          (call 0 (i32.sub (local.get 0) (i32.const 2)))))))
  (export "func_0" (func 0)))
