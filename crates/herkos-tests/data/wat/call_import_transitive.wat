;; Regression test for transitive host parameter in direct calls
;;
;; This module has:
;; - A function import: env.log
;; - A writer function that directly calls the import (needs_host = true)
;; - A caller function that directly calls writer (but has no direct import call)
;;
;; The bug: caller function gets no host parameter even though it transitively
;; needs it (because it calls writer which calls the import).

(module
  (import "env" "log" (func $log (param i32)))

  ;; writer directly calls the import — needs_host = true
  (func $writer (param i32)
    local.get 0
    call $log
  )

  ;; caller directly calls writer — has no direct import call itself,
  ;; but before the fix it would not get a host parameter, causing codegen to fail
  ;; because writer's generated signature requires host
  (func $caller (param i32)
    local.get 0
    call $writer
  )

  (export "writer" (func $writer))
  (export "caller" (func $caller))
)
