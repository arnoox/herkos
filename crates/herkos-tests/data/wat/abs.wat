(module
  (func (param i32) (result i32)
    local.get 0
    i32.const 0
    i32.lt_s
    if (result i32)
      i32.const 0
      local.get 0
      i32.sub
    else
      local.get 0
    end)
  (export "func_0" (func 0)))
