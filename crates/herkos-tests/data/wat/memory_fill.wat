(module
  (memory 1 1)

  ;; Fill a region of memory with a byte value.
  (func (export "fill_region") (param $dst i32) (param $val i32) (param $len i32)
    local.get $dst
    local.get $val
    local.get $len
    memory.fill)

  ;; Load a single byte (i32.load8_u) for verification.
  (func (export "load_byte") (param $addr i32) (result i32)
    local.get $addr
    i32.load8_u))
