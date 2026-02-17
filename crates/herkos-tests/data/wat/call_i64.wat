(module
  (func $add64 (param i64 i64) (result i64)
    local.get 0
    local.get 1
    i64.add)
  (func $main (param i64 i64) (result i64)
    local.get 0
    local.get 1
    call 0)
  (export "func_0" (func 0))
  (export "func_1" (func 1)))
