;; Math library module — imports memory from host, provides utility functions.
;; Demonstrates the LibraryModule pattern: this module has no owned memory.
;;
;; Pipeline: math_library.wat → math_library.wasm → src/math_library_wasm.rs
(module
  ;; Import memory from the host (makes this a LibraryModule)
  (import "env" "memory" (memory 1 16))

  ;; Import a logging function from the host
  (import "env" "log_result" (func $log_result (param i32)))

  ;; sum_array(offset, count): sums count i32 values starting at offset
  (func (export "sum_array") (param $offset i32) (param $count i32) (result i32)
    (local $sum i32)
    (local $i i32)
    (local.set $sum (i32.const 0))
    (local.set $i (i32.const 0))
    (block $done
      (loop $loop
        (br_if $done (i32.ge_u (local.get $i) (local.get $count)))
        (local.set $sum
          (i32.add
            (local.get $sum)
            (i32.load
              (i32.add
                (local.get $offset)
                (i32.mul (local.get $i) (i32.const 4))))))
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $loop)))
    (local.get $sum))

  ;; double_array(offset, count): doubles each i32 value in place
  (func (export "double_array") (param $offset i32) (param $count i32)
    (local $i i32)
    (local $addr i32)
    (local.set $i (i32.const 0))
    (block $done
      (loop $loop
        (br_if $done (i32.ge_u (local.get $i) (local.get $count)))
        (local.set $addr
          (i32.add
            (local.get $offset)
            (i32.mul (local.get $i) (i32.const 4))))
        (i32.store
          (local.get $addr)
          (i32.mul (i32.load (local.get $addr)) (i32.const 2)))
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $loop))))

  ;; sum_and_log(offset, count): sums values and calls log_result with the sum
  (func (export "sum_and_log") (param $offset i32) (param $count i32) (result i32)
    (local $sum i32)
    (local $i i32)
    (local.set $sum (i32.const 0))
    (local.set $i (i32.const 0))
    (block $done
      (loop $loop
        (br_if $done (i32.ge_u (local.get $i) (local.get $count)))
        (local.set $sum
          (i32.add
            (local.get $sum)
            (i32.load
              (i32.add
                (local.get $offset)
                (i32.mul (local.get $i) (i32.const 4))))))
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $loop)))
    (call $log_result (local.get $sum))
    (local.get $sum))

  ;; memory_info: returns current memory size in pages
  (func (export "memory_info") (result i32)
    memory.size)
)
