(module
  (type $binop (func (param i32 i32) (result i32)))
  (type $unop (func (param i32) (result i32)))
  (table 5 funcref)
  (elem (i32.const 0) $add $sub $mul $negate)
  (func $add (type $binop)
    local.get 0
    local.get 1
    i32.add)
  (func $sub (type $binop)
    local.get 0
    local.get 1
    i32.sub)
  (func $mul (type $binop)
    local.get 0
    local.get 1
    i32.mul)
  ;; Different signature: unary (i32) -> i32
  (func $negate (type $unop)
    i32.const 0
    local.get 0
    i32.sub)
  ;; dispatch_binop(a, b, op_index) → calls table[op_index](a, b) via $binop type
  (func (param i32 i32 i32) (result i32)
    local.get 0
    local.get 1
    local.get 2
    call_indirect (type $binop))
  ;; dispatch_unop(a, op_index) → calls table[op_index](a) via $unop type
  (func (param i32 i32) (result i32)
    local.get 0
    local.get 1
    call_indirect (type $unop))
  (export "add" (func $add))
  (export "sub" (func $sub))
  (export "mul" (func $mul))
  (export "negate" (func $negate))
  (export "dispatch_binop" (func 4))
  (export "dispatch_unop" (func 5)))
