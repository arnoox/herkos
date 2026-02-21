(module
  (memory 1 2)

  ;; Active segment 1: four known bytes at offset 0
  (data (i32.const 0) "\01\02\03\04")

  ;; Active segment 2: four known bytes at a different offset
  (data (i32.const 16) "\0A\0B\0C\0D")

  ;; Passive segment: not copied to memory at startup.
  ;; Exercised here to verify the transpiler skips it without error.
  ;; TODO: later, add a test that explicitly loads this segment to verify it is present and correct.
  (data "passive bytes that are never loaded")

  (func (export "load_i32") (param i32) (result i32)
    local.get 0
    i32.load)

  (func (export "load_byte") (param i32) (result i32)
    local.get 0
    i32.load8_u))
