(module
  (memory 1 1)

  ;; Passive data segment (index 0): "Hello"
  (data "Hello")

  ;; Copy a sub-range of the passive segment into linear memory.
  (func (export "init_region")
    (param $dst i32) (param $src_offset i32) (param $len i32)
    local.get $dst
    local.get $src_offset
    local.get $len
    memory.init 0)

  ;; Drop the passive segment (no-op in the safe backend).
  (func (export "drop_segment")
    data.drop 0)

  ;; Load a single byte for verification.
  (func (export "load_byte") (param $addr i32) (result i32)
    local.get $addr
    i32.load8_u))
