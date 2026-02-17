(module
  (memory 1 1)
  (func (param i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.store
    local.get 0
    i32.load)
  (export "func_0" (func 0)))
