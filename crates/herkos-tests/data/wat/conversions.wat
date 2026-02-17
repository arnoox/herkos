(module
  ;; i32.wrap_i64: truncate i64 to low 32 bits
  (func (param i64) (result i32)
    local.get 0
    i32.wrap_i64)
  ;; i64.extend_i32_s: sign-extend i32 to i64
  (func (param i32) (result i64)
    local.get 0
    i64.extend_i32_s)
  ;; i64.extend_i32_u: zero-extend i32 to i64
  (func (param i32) (result i64)
    local.get 0
    i64.extend_i32_u)
  ;; f64.convert_i32_s: signed i32 to f64
  (func (param i32) (result f64)
    local.get 0
    f64.convert_i32_s)
  ;; i32.trunc_f64_s: f64 to signed i32 (trapping)
  (func (param f64) (result i32)
    local.get 0
    i32.trunc_f64_s)
  ;; f32.demote_f64: f64 to f32
  (func (param f64) (result f32)
    local.get 0
    f32.demote_f64)
  ;; f64.promote_f32: f32 to f64
  (func (param f32) (result f64)
    local.get 0
    f64.promote_f32)
  ;; i32.reinterpret_f32: bitcast f32 to i32
  (func (param f32) (result i32)
    local.get 0
    i32.reinterpret_f32)
  ;; f32.reinterpret_i32: bitcast i32 to f32
  (func (param i32) (result f32)
    local.get 0
    f32.reinterpret_i32)
  (export "wrap_i64" (func 0))
  (export "extend_i32_s" (func 1))
  (export "extend_i32_u" (func 2))
  (export "convert_i32_s" (func 3))
  (export "trunc_f64_s" (func 4))
  (export "demote_f64" (func 5))
  (export "promote_f32" (func 6))
  (export "reinterpret_f32" (func 7))
  (export "reinterpret_i32" (func 8)))
