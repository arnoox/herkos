(module
  (memory 1 1)
  (func (param i32 i32) (result i32)
    (local i32 i32)
    ;; local 0 = addr, local 1 = count, local 2 = sum, local 3 = i
    i32.const 0
    local.set 2
    i32.const 0
    local.set 3
    block
      loop
        ;; if i >= count, break
        local.get 3
        local.get 1
        i32.ge_s
        br_if 1
        ;; sum += memory[addr + i*4]
        local.get 2
        local.get 0
        local.get 3
        i32.const 4
        i32.mul
        i32.add
        i32.load
        i32.add
        local.set 2
        ;; i++
        local.get 3
        i32.const 1
        i32.add
        local.set 3
        br 0
      end
    end
    local.get 2)
  ;; Helper: store an i32 at an address
  (func (param i32 i32)
    local.get 0
    local.get 1
    i32.store)
  (export "func_0" (func 0))
  (export "func_1" (func 1)))
