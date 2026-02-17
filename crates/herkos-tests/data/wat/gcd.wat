(module
  (func (param i32 i32) (result i32)
    ;; gcd(a, b): while b != 0 { tmp = b; b = a % b; a = tmp }; return a
    block
      loop
        ;; if b == 0, break
        local.get 1
        i32.eqz
        br_if 1
        ;; tmp = a % b (on stack)
        local.get 0
        local.get 1
        i32.rem_s
        ;; a = b
        local.get 1
        local.set 0
        ;; b = tmp
        local.set 1
        br 0
      end
    end
    local.get 0)
  (export "func_0" (func 0)))
