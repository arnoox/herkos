(module
  ;; f64 division
  (func (param f64 f64) (result f64)
    local.get 0
    local.get 1
    f64.div)
  ;; f64 min
  (func (param f64 f64) (result f64)
    local.get 0
    local.get 1
    f64.min)
  ;; f64 comparison: less than → returns i32
  (func (param f64 f64) (result i32)
    local.get 0
    local.get 1
    f64.lt)
  ;; f64 sqrt
  (func (param f64) (result f64)
    local.get 0
    f64.sqrt)
  ;; f64 floor
  (func (param f64) (result f64)
    local.get 0
    f64.floor)
  ;; f64 ceil
  (func (param f64) (result f64)
    local.get 0
    f64.ceil)
  ;; f64 neg
  (func (param f64) (result f64)
    local.get 0
    f64.neg)
  (export "div" (func 0))
  (export "min" (func 1))
  (export "lt" (func 2))
  (export "sqrt" (func 3))
  (export "floor" (func 4))
  (export "ceil" (func 5))
  (export "neg" (func 6)))
