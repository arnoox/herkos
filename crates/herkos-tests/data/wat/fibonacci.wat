(module
  (func (param i32) (result i32)
    (local i32 i32 i32)
    ;; local 0 = n (param), local 1 = a, local 2 = b, local 3 = i
    i32.const 0
    local.set 1
    i32.const 1
    local.set 2
    i32.const 0
    local.set 3
    block
      loop
        local.get 3
        local.get 0
        i32.ge_s
        br_if 1
        ;; tmp = a + b (stays on stack)
        local.get 1
        local.get 2
        i32.add
        ;; a = b
        local.get 2
        local.set 1
        ;; b = tmp (from stack)
        local.set 2
        ;; i++
        local.get 3
        i32.const 1
        i32.add
        local.set 3
        br 0
      end
    end
    local.get 1)
  (export "func_0" (func 0)))
