(module
  ;; i64 division (signed)
  (func (param i64 i64) (result i64)
    local.get 0
    local.get 1
    i64.div_s)
  ;; i64 bitwise and
  (func (param i64 i64) (result i64)
    local.get 0
    local.get 1
    i64.and)
  ;; i64 shift left
  (func (param i64 i64) (result i64)
    local.get 0
    local.get 1
    i64.shl)
  ;; i64 comparison: less than signed → returns i32
  (func (param i64 i64) (result i32)
    local.get 0
    local.get 1
    i64.lt_s)
  ;; i64 clz (count leading zeros)
  (func (param i64) (result i64)
    local.get 0
    i64.clz)
  ;; i64 rotate left
  (func (param i64 i64) (result i64)
    local.get 0
    local.get 1
    i64.rotl)
  ;; i64 rem_u (unsigned remainder)
  (func (param i64 i64) (result i64)
    local.get 0
    local.get 1
    i64.rem_u)
  (export "div_s" (func 0))
  (export "bitand" (func 1))
  (export "shl" (func 2))
  (export "lt_s" (func 3))
  (export "clz" (func 4))
  (export "rotl" (func 5))
  (export "rem_u" (func 6)))
