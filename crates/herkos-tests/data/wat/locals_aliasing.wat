(module
  ;; ── Regression test: the digital_root aliasing bug ───────────────────────
  ;;
  ;; Computes n % 10 (unsigned) using the exact instruction sequence that lived
  ;; inside the inner loop of digital_root and exposed the LocalGet aliasing bug.
  ;;
  ;; Pattern:
  ;;   local.get 0   <- snapshot of n; stays on stack bottom until i32.sub
  ;;   local.get 0   <- n again, for division (consumed by i32.div_u)
  ;;   i32.const 10
  ;;   i32.div_u     <- n / 10; now stack = [n, n/10]
  ;;   local.tee 0   <- stores n/10 back into local 0; keeps n/10 on stack
  ;;   i32.const 10
  ;;   i32.mul       <- (n/10)*10
  ;;   i32.sub       <- n - (n/10)*10 = n%10
  ;;
  ;; With the old buggy LocalGet (pushing the local's VarId directly), the
  ;; local.tee would emit `v0 = quotient`, mutating the VarId that was already
  ;; sitting at the bottom of the value stack from the first local.get 0.
  ;; The subtraction then computed (n/10) - (n/10)*10 instead of n - (n/10)*10,
  ;; returning -9 for n=10 instead of the correct 0.
  (func (export "mod10_via_tee") (param i32) (result i32)
    local.get 0          ;; snap of n, survives to i32.sub → [n]
    local.get 0          ;; n for division                 → [n, n]
    i32.const 10
    i32.div_u            ;; n / 10                         → [n, n/10]
    local.tee 0          ;; local 0 ← n/10, keep on stack → [n, n/10]
    i32.const 10
    i32.mul              ;; (n/10) * 10                    → [n, (n/10)*10]
    i32.sub              ;; n - (n/10)*10 = n % 10         → [n%10]
    return)

  ;; ── local.get snapshot survives a local.set on the same local ───────────
  ;;
  ;; Reads `n`, then overwrites local 0 with n*3, then reads the new value.
  ;; Returns: old_n - new_n = n - 3n = -2n.
  ;;
  ;; With the bug: the bottom stack entry aliased v0, so after `local.set 0`
  ;; wrote v0=3n, the "old" read also became 3n → result was 3n - 3n = 0
  ;; instead of n - 3n = -2n (wrong for n ≠ 0).
  (func (export "preserve_across_set") (param i32) (result i32)
    local.get 0          ;; snap of n, survives to i32.sub → [n]
    local.get 0          ;; n for multiplication           → [n, n]
    i32.const 3
    i32.mul              ;; n * 3                          → [n, 3n]
    local.set 0          ;; local 0 ← 3n                  → [n]
    local.get 0          ;; new value of local 0 = 3n      → [n, 3n]
    i32.sub              ;; n - 3n = -2n
    return)

  ;; ── local.get snapshot survives a local.tee on the same local ───────────
  ;;
  ;; A prior snapshot of `a` must equal `a` after local.tee stores (a + b)
  ;; into local 0. Returns: a - (a + b) = -b.
  ;;
  ;; With the bug: after `local.tee 0` emitted `v0 = a+b`, the bottom stack
  ;; entry (first local.get 0) also became a+b, so the subtraction computed
  ;; (a+b) - (a+b) = 0 instead of a - (a+b) = -b (wrong for b ≠ 0).
  (func (export "get_snap_vs_tee") (param i32 i32) (result i32)
    local.get 0          ;; snap of a, survives to i32.sub → [a]
    local.get 0          ;; a for addition                 → [a, a]
    local.get 1          ;; b                              → [a, a, b]
    i32.add              ;; a + b                          → [a, a+b]
    local.tee 0          ;; local 0 ← a+b, keep on stack  → [a, a+b]
    i32.sub              ;; a - (a+b) = -b
    return)

  ;; ── local.get snapshot survives both a local.tee and a local.set ────────
  ;;
  ;; First tee stores (a + b) into local 0, then a further set overwrites it
  ;; with (a + b) * 2.  The original snapshot of `a` must still equal `a`.
  ;; Returns: a - (a + b)*2.
  ;;
  ;; With the bug: v0 is overwritten twice (first to a+b by tee, then to
  ;; (a+b)*2 by set).  The bottom stack entry, being an alias to v0, ends
  ;; up as (a+b)*2, so the subtraction returns 0 instead of a - (a+b)*2.
  (func (export "get_tee_then_set") (param i32 i32) (result i32)
    local.get 0          ;; snap of a, survives both writes → [a]
    local.get 0          ;; a for addition                  → [a, a]
    local.get 1          ;; b                               → [a, a, b]
    i32.add              ;; a + b                           → [a, a+b]
    local.tee 0          ;; local 0 ← a+b, keep on stack   → [a, a+b]
    i32.const 2
    i32.mul              ;; (a+b) * 2                       → [a, (a+b)*2]
    local.set 0          ;; local 0 ← (a+b)*2              → [a]
    local.get 0          ;; (a+b)*2                         → [a, (a+b)*2]
    i32.sub              ;; a - (a+b)*2
    return))
