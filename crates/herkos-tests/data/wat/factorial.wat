(module
  (func (param i32) (result i32)
    (local i32)
    ;; local 0 = n (param), local 1 = result
    i32.const 1
    local.set 1
    block
      loop
        ;; if n <= 1, break
        local.get 0
        i32.const 1
        i32.le_s
        br_if 1
        ;; result = result * n
        local.get 1
        local.get 0
        i32.mul
        local.set 1
        ;; n = n - 1
        local.get 0
        i32.const 1
        i32.sub
        local.set 0
        br 0
      end
    end
    local.get 1)
  (export "func_0" (func 0)))
