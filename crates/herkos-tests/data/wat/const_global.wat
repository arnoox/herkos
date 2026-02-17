(module
  (global i32 (i32.const 42))
  (func (result i32)
    global.get 0)
  (export "get_answer" (func 0)))
