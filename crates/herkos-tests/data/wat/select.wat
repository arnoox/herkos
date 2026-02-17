(module
  ;; func_0: max(a, b) via select
  ;; select picks val1 if condition != 0, else val2
  (func (param i32 i32) (result i32)
    local.get 0         ;; val1 (a)
    local.get 1         ;; val2 (b)
    local.get 0
    local.get 1
    i32.gt_s            ;; condition: a > b
    select)

  ;; func_1: min(a, b) via select
  (func (param i32 i32) (result i32)
    local.get 0         ;; val1 (a)
    local.get 1         ;; val2 (b)
    local.get 0
    local.get 1
    i32.lt_s            ;; condition: a < b
    select)

  ;; func_2: abs(x) via select
  ;; returns x if x >= 0, else -x
  (func (param i32) (result i32)
    local.get 0           ;; val1 (x)
    i32.const 0
    local.get 0
    i32.sub               ;; val2 (0 - x = -x)
    local.get 0
    i32.const 0
    i32.ge_s              ;; condition: x >= 0
    select)

  ;; func_3: clamp to non-negative via select
  ;; returns x if x > 0, else 0
  (func (param i32) (result i32)
    local.get 0           ;; val1 (x)
    i32.const 0           ;; val2 (0)
    local.get 0
    i32.const 0
    i32.gt_s              ;; condition: x > 0
    select)

  ;; func_4: conditional increment
  ;; returns a + 1 if flag != 0, else a
  (func (param i32 i32) (result i32)
    local.get 0
    i32.const 1
    i32.add               ;; val1 (a + 1)
    local.get 0           ;; val2 (a)
    local.get 1           ;; condition (flag)
    select)

  (export "func_0" (func 0))
  (export "func_1" (func 1))
  (export "func_2" (func 2))
  (export "func_3" (func 3))
  (export "func_4" (func 4)))
