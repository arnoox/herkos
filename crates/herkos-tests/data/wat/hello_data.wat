(module
  (memory 1 1)
  (data (i32.const 0) "Hello")
  (func (param i32) (result i32)
    local.get 0
    i32.load)
  (export "load_word" (func 0)))
