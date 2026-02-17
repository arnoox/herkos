(module
  (func $helper (param i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.add)
  (func $main (param i32 i32) (result i32)
    local.get 0
    local.get 1
    call 0)
  (export "func_0" (func 0))
  (export "func_1" (func 1)))
