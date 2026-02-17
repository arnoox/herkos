(module
  (func (param i32) (result i32)
    local.get 0
    if (result i32)
      i32.const 42
    else
      i32.const 99
    end)
  (export "func_0" (func 0)))
