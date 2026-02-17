(module
  (memory 1 4)
  ;; get_size: returns current memory size in pages
  (func (result i32)
    memory.size)
  ;; grow: grows memory by delta pages, returns previous size (or -1)
  (func (param i32) (result i32)
    local.get 0
    memory.grow)
  ;; store_and_load: store a value then load it back (for testing grown memory)
  (func (param i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.store
    local.get 0
    i32.load)
  (export "get_size" (func 0))
  (export "grow" (func 1))
  (export "store_and_load" (func 2)))
