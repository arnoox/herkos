(module
  (memory 1 1)
  (func $store_value (param i32 i32)
    local.get 0
    local.get 1
    i32.store)
  (func $store_via_call (param i32 i32)
    local.get 0
    local.get 1
    call 0)
  (func $load_value (param i32) (result i32)
    local.get 0
    i32.load)
  (export "func_0" (func 0))
  (export "func_1" (func 1))
  (export "func_2" (func 2)))
