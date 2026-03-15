;; Regression test for issue #19: indirect calls missing host parameter for WASI functions
;;
;; This module has:
;; - A function import: env.log
;; - A writer function that calls the import directly (needs_host = true)
;; - A dispatcher function that uses call_indirect to reach writer (but has no direct import call)
;;
;; The bug: dispatcher gets no host parameter even though it dispatches to writer which needs it.

(module
  (type $log_fn (func (param i32)))
  (type $dispatch_fn (func (param i32 i32)))

  (import "env" "log" (func $log (type $log_fn)))

  (table 1 funcref)
  (elem (i32.const 0) $writer)

  ;; writer calls the import directly — needs host parameter
  (func $writer (type $log_fn)
    local.get 0
    call $log
  )

  ;; dispatcher uses call_indirect to reach writer
  ;; dispatcher has no direct import call itself, but before the fix
  ;; it would not get a host parameter, causing codegen to fail
  (func $dispatcher (type $dispatch_fn)
    local.get 0
    local.get 1
    call_indirect (type $log_fn)
  )

  (export "writer" (func $writer))
  (export "dispatcher" (func $dispatcher))
)
