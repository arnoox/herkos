(module
  ;; Test 1: Swap two values using locals
  ;; Takes two i32 parameters, swaps them using a local, returns swapped pair as first+second
  (func (param i32 i32) (result i32)
    (local i32)
    ;; local 0 = param 0 (first)
    ;; local 1 = param 1 (second)
    ;; local 2 = temp

    ;; Save first parameter to temp
    local.get 0
    local.set 2

    ;; Save second to first position (conceptually)
    ;; For return, we compute: temp + (param1 * 10)
    local.get 2
    local.get 1
    i32.const 10
    i32.mul
    i32.add
    return)

  ;; Test 2: Simple local usage
  ;; Takes one i32 parameter and adds a constant from a local
  (func (param i32) (result i32)
    (local i32)
    ;; local 0 = param
    ;; local 1 = constant local

    ;; Initialize local to 100
    i32.const 100
    local.set 1

    ;; Return param + local
    local.get 0
    local.get 1
    i32.add
    return)

  ;; Test 3: local.tee preserves value on stack
  ;; Demonstrates that local.tee properly stores and keeps value on stack
  (func (param i32) (result i32)
    (local i32)
    ;; local 0 = param
    ;; local 1 = accumulator

    i32.const 0
    local.set 1

    ;; Add 5 to param, store in local 1 (tee keeps it on stack)
    local.get 0
    i32.const 5
    i32.add
    local.tee 1

    ;; Verify we can still use the value from tee
    i32.const 2
    i32.mul
    return)

  ;; Test 4: Locals are zero-initialized (Wasm spec requirement)
  ;; If we don't set a local before using it, it must be zero
  (func (param i32) (result i32)
    (local i32 i32)
    ;; local 0 = param
    ;; local 1 = initialized to 0 (not set, so should be zero)
    ;; local 2 = initialized to 0 (not set, so should be zero)

    ;; Get uninitialized local (should be 0)
    local.get 1
    ;; Should equal 0, so param + 0 = param
    local.get 0
    i32.add
    return)

  ;; Test 5: Complex local manipulations - running sum
  ;; Uses locals to accumulate a sum
  (func (param i32 i32 i32) (result i32)
    (local i32)
    ;; local 0, 1, 2 = params
    ;; local 3 = sum accumulator

    i32.const 0
    local.set 3

    ;; sum += param0
    local.get 3
    local.get 0
    i32.add
    local.set 3

    ;; sum += param1
    local.get 3
    local.get 1
    i32.add
    local.set 3

    ;; sum += param2
    local.get 3
    local.get 2
    i32.add
    local.set 3

    ;; return sum
    local.get 3
    return)

  ;; Export test functions
  (export "swap" (func 0))
  (export "simple_local" (func 1))
  (export "tee_test" (func 2))
  (export "zero_init" (func 3))
  (export "running_sum" (func 4)))
