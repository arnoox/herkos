;; Test memory import functionality
;; This module imports memory from the host, making it a LibraryModule.
;; The host provides memory via a trait, and all memory operations use it.
(module
  ;; Import memory from the host (not local memory definition)
  (import "env" "memory" (memory 1 256))

  ;; Import a helper function
  (import "env" "print_i32" (func $print_i32 (param i32)))

  ;; Function that reads from imported memory
  (func (export "read_at") (param i32) (result i32)
    ;; Load i32 at the offset given by the parameter
    local.get 0
    i32.load
  )

  ;; Function that writes to imported memory
  (func (export "write_at") (param i32 i32)
    ;; Store i32 at offset (param 0), value (param 1)
    local.get 0
    local.get 1
    i32.store
  )

  ;; Function that uses both import: calls host function and uses memory
  (func (export "process") (param i32)
    (local i32)
    ;; Load from memory at offset param into a local
    local.get 0
    i32.load
    local.set 1

    ;; Call imported function with the loaded value
    local.get 1
    call $print_i32

    ;; Store the loaded value back at offset 0
    i32.const 0
    local.get 1
    i32.store
  )

  ;; Test memory size operation with imported memory
  (func (export "memory_size") (result i32)
    ;; Get the current memory size in pages
    memory.size
  )

  ;; Test memory.grow with imported memory
  (func (export "try_grow") (param i32) (result i32)
    ;; Try to grow memory by the given number of pages
    local.get 0
    memory.grow
  )
)
