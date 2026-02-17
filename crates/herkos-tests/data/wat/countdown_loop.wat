(module
  (func (param i32) (result i32)
    block (result i32)
      loop
        local.get 0
        i32.const 0
        i32.gt_s
        i32.eqz
        br_if 1
        local.get 0
        i32.const 1
        i32.sub
        local.set 0
        br 0
      end
      local.get 0
    end)
  (export "func_0" (func 0)))
