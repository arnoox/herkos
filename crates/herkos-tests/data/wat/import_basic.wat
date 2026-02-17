;; Test basic import functionality with trait generation
(module
  (import "env" "print_i32" (func $print_i32 (param i32)))
  (import "env" "read_i32" (func $read_i32 (result i32)))
  (import "wasi_snapshot_preview1" "fd_write" (func $fd_write (param i32 i32 i32 i32) (result i32)))

  ;; Add a mutable global to trigger wrapper generation
  (global $counter (mut i32) (i32.const 0))

  (func (export "test_imports") (param i32) (result i32)
    ;; Increment counter
    global.get $counter
    i32.const 1
    i32.add
    global.set $counter

    ;; Call imported functions
    local.get 0
    call $print_i32

    call $read_i32
    i32.const 10
    i32.add
  )

  (func (export "test_wasi") (result i32)
    i32.const 1
    i32.const 0
    i32.const 1
    i32.const 16
    call $fd_write
  )

  (func (export "get_counter") (result i32)
    global.get $counter
  )
)
