(module
  ;; func_0: always traps
  (func (result i32)
    unreachable)

  ;; func_1: traps if n == 0, else returns n
  (func (param i32) (result i32)
    local.get 0
    i32.eqz
    if
      unreachable
    end
    local.get 0)

  ;; func_2: safe division — traps on zero divisor via unreachable
  (func (param i32 i32) (result i32)
    local.get 1
    i32.eqz
    if
      unreachable
    end
    local.get 0
    local.get 1
    i32.div_s)

  ;; func_3: dead unreachable after early return (never executes)
  ;; Verifies that unreachable in dead code doesn't affect execution.
  (func (param i32) (result i32)
    local.get 0
    return
    unreachable)

  (export "func_0" (func 0))
  (export "func_1" (func 1))
  (export "func_2" (func 2))
  (export "func_3" (func 3)))
