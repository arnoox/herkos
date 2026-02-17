(module
  ;; func_0: early return if negative, else double
  ;; return -1 if n < 0, else n * 2
  (func (param i32) (result i32)
    local.get 0
    i32.const 0
    i32.lt_s
    if
      i32.const -1
      return
    end
    local.get 0
    i32.const 2
    i32.mul)

  ;; func_1: first positive — scan through a block/loop and return early
  ;; counts up from n; returns the first value > 0
  ;; (for n > 0 returns n; for n <= 0 returns 1)
  (func (param i32) (result i32)
    block
      loop
        local.get 0
        i32.const 0
        i32.gt_s
        if
          local.get 0
          return
        end
        local.get 0
        i32.const 1
        i32.add
        local.set 0
        br 0
      end
    end
    local.get 0)

  ;; func_2: sum with early exit
  ;; sum 1..n, but return -1 immediately if n <= 0
  (func (param i32) (result i32)
    (local i32)  ;; accumulator
    local.get 0
    i32.const 0
    i32.le_s
    if
      i32.const -1
      return
    end
    i32.const 0
    local.set 1
    block
      loop
        local.get 0
        i32.const 0
        i32.le_s
        br_if 1
        ;; acc += n
        local.get 1
        local.get 0
        i32.add
        local.set 1
        ;; n--
        local.get 0
        i32.const 1
        i32.sub
        local.set 0
        br 0
      end
    end
    local.get 1)

  ;; func_3: nested return — return from within nested blocks
  ;; returns 10 if a > 0, 20 if b > 0, else 30
  (func (param i32 i32) (result i32)
    block
      block
        local.get 0
        i32.const 0
        i32.gt_s
        if
          i32.const 10
          return
        end
        local.get 1
        i32.const 0
        i32.gt_s
        if
          i32.const 20
          return
        end
      end
    end
    i32.const 30)

  (export "func_0" (func 0))
  (export "func_1" (func 1))
  (export "func_2" (func 2))
  (export "func_3" (func 3)))
