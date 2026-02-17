;; Test multiple imports from different modules
;; This module tests:
;; 1. Imports from multiple host modules (env, wasi, custom)
;; 2. Multiple function imports of different types
;; 3. Mixed calls to imports and local functions
(module
  ;; Imports from 'env' module
  (import "env" "add" (func $env_add (param i32 i32) (result i32)))
  (import "env" "mul" (func $env_mul (param i32 i32) (result i32)))

  ;; Imports from 'wasi_snapshot_preview1' module
  (import "wasi_snapshot_preview1" "fd_write" (func $fd_write (param i32 i32 i32 i32) (result i32)))

  ;; Import a simple void function
  (import "env" "log" (func $log (param i32)))

  ;; Add a mutable global to trigger wrapper
  (global $counter (mut i32) (i32.const 0))

  ;; Local function 1: simple calculation
  (func $local_add (param i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.add
  )

  ;; Local function 2: multiply
  (func $local_mul (param i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.mul
  )

  ;; Export: call both import and local functions
  (func (export "mixed_calls") (param i32 i32) (result i32)
    ;; Increment counter
    global.get $counter
    i32.const 1
    i32.add
    global.set $counter

    ;; Call imported function
    local.get 0
    local.get 1
    call $env_add  ;; Import: host provides add

    ;; Call local function with result
    i32.const 2
    call $local_mul  ;; Local: compute result * 2
  )

  ;; Export: test multiple imports
  (func (export "use_multiple_imports") (param i32 i32) (result i32)
    (local i32)
    ;; Call first import
    local.get 0
    local.get 1
    call $env_add

    ;; Log the result
    local.tee 2  ;; Save to temp local for logging

    ;; Call log (void import)
    call $log

    ;; Get temp value back
    local.get 2
  )

  ;; Export: use WASI import
  (func (export "use_wasi") (result i32)
    i32.const 1
    i32.const 0
    i32.const 1
    i32.const 20
    call $fd_write
  )

  ;; Export: call all three types of imports
  (func (export "call_all_imports") (param i32 i32) (result i32)
    (local i32)
    ;; Call env.add
    local.get 0
    local.get 1
    call $env_add
    local.set 2

    ;; Call env.log (void)
    local.get 2
    call $log

    ;; Get second result from env.mul
    local.get 0
    local.get 1
    call $env_mul
    local.get 2
    i32.add

    ;; Call WASI and add its result
    i32.const 1
    i32.const 0
    i32.const 1
    i32.const 24
    call $fd_write
    i32.add
  )

  ;; Export: call local functions only
  (func (export "call_local_only") (param i32 i32) (result i32)
    local.get 0
    local.get 1
    call $local_add
    i32.const 3
    call $local_mul
  )

  ;; Export: mixed local and import
  (func (export "local_then_import") (param i32 i32) (result i32)
    ;; First use local function
    local.get 0
    local.get 1
    call $local_add  ;; Result: param0 + param1

    ;; Then use import
    i32.const 10
    call $env_mul  ;; Multiply result by 10
  )

  ;; Export: get counter
  (func (export "get_counter") (result i32)
    global.get $counter
  )
)
