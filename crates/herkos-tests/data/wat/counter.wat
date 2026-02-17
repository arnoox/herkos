(module
  (global (mut i32) (i32.const 0))
  (func (result i32)
    global.get 0
    i32.const 1
    i32.add
    global.set 0
    global.get 0)
  (func (result i32)
    global.get 0)
  (export "increment" (func 0))
  (export "get_count" (func 1)))
