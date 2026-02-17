(module
  (global (mut i32) (i32.const 0))
  (func $set_global (param i32)
    local.get 0
    global.set 0)
  (func $get_global (result i32)
    global.get 0)
  (func $set_and_get (param i32) (result i32)
    local.get 0
    call 0
    call 1)
  (export "set_and_get" (func 2))
  (export "get" (func 1)))
