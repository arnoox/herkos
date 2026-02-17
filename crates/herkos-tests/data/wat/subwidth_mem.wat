(module
  (memory 1 1)
  ;; store_byte: i32.store8 at addr
  (func (param i32 i32)
    local.get 0
    local.get 1
    i32.store8)
  ;; load_byte_u: i32.load8_u from addr
  (func (param i32) (result i32)
    local.get 0
    i32.load8_u)
  ;; load_byte_s: i32.load8_s from addr (sign-extend)
  (func (param i32) (result i32)
    local.get 0
    i32.load8_s)
  ;; store_i16: i32.store16 at addr
  (func (param i32 i32)
    local.get 0
    local.get 1
    i32.store16)
  ;; load_i16_u: i32.load16_u from addr
  (func (param i32) (result i32)
    local.get 0
    i32.load16_u)
  ;; load_i16_s: i32.load16_s from addr (sign-extend)
  (func (param i32) (result i32)
    local.get 0
    i32.load16_s)
  ;; i64_store8: i64.store8 at addr
  (func (param i32 i64)
    local.get 0
    local.get 1
    i64.store8)
  ;; i64_load8_u: i64.load8_u from addr
  (func (param i32) (result i64)
    local.get 0
    i64.load8_u)
  ;; i64_load8_s: i64.load8_s from addr
  (func (param i32) (result i64)
    local.get 0
    i64.load8_s)
  ;; i64_store32: i64.store32 at addr
  (func (param i32 i64)
    local.get 0
    local.get 1
    i64.store32)
  ;; i64_load32_u: i64.load32_u from addr
  (func (param i32) (result i64)
    local.get 0
    i64.load32_u)
  ;; i64_load32_s: i64.load32_s from addr
  (func (param i32) (result i64)
    local.get 0
    i64.load32_s)
  (export "store_byte" (func 0))
  (export "load_byte_u" (func 1))
  (export "load_byte_s" (func 2))
  (export "store_i16" (func 3))
  (export "load_i16_u" (func 4))
  (export "load_i16_s" (func 5))
  (export "i64_store8" (func 6))
  (export "i64_load8_u" (func 7))
  (export "i64_load8_s" (func 8))
  (export "i64_store32" (func 9))
  (export "i64_load32_u" (func 10))
  (export "i64_load32_s" (func 11)))
