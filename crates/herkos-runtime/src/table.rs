//! Wasm indirect call table — supports `call_indirect`.
//!
//! A Wasm table is a vector of nullable function references. `call_indirect`
//! looks up an entry by index, checks its type signature, and dispatches.
//!
//! In transpiled Rust, function pointers have heterogeneous types (different
//! params/results). We store them as type-erased `FuncRef` entries: each
//! entry carries a `type_index` (the Wasm type section index) and an opaque
//! function pointer. The transpiler generates a match-based dispatch that
//! casts the pointer back to the correct concrete type after verifying the
//! type index.
//!
//! The table uses a fixed-size backing array (const generic `MAX_SIZE`)
//! to stay `no_std` compatible.

use crate::{WasmResult, WasmTrap};

/// A single table entry: a typed function reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FuncRef {
    /// Index into the module's type section. Used by `call_indirect` to
    /// verify the caller's expected signature matches the callee's actual
    /// signature. A mismatch is a trap (`IndirectCallTypeMismatch`).
    pub type_index: u32,
    /// Index into the module's function index space. The transpiler
    /// generates a match/dispatch over this value to call the right
    /// concrete Rust function.
    pub func_index: u32,
}

/// Indirect call table with a compile-time maximum size.
///
/// `MAX_SIZE` is derived from the Wasm module's table declaration.
/// Entries are `Option<FuncRef>` — `None` means the slot is empty
/// (calling it traps with `UndefinedElement`).
pub struct Table<const MAX_SIZE: usize> {
    entries: [Option<FuncRef>; MAX_SIZE],
    /// Current number of initialized entries (analogous to `active_pages`).
    /// Accesses at or beyond this index trap with `TableOutOfBounds`.
    active_size: usize,
}

impl<const MAX_SIZE: usize> Table<MAX_SIZE> {
    /// Create a new table with `initial_size` slots, all empty (`None`).
    ///
    /// # Errors
    /// Returns `ConstructionError::TableInitialSizeExceedsMax` if `initial_size > MAX_SIZE`.
    pub fn try_new(initial_size: usize) -> Result<Self, crate::ConstructionError> {
        if initial_size > MAX_SIZE {
            return Err(crate::ConstructionError::TableInitialSizeExceedsMax {
                initial: initial_size,
                max: MAX_SIZE,
            });
        }
        Ok(Self {
            entries: [None; MAX_SIZE],
            active_size: initial_size,
        })
    }

    /// Current number of active table slots.
    #[inline(always)]
    pub fn size(&self) -> usize {
        self.active_size
    }

    /// Look up a table entry by index. Returns the `FuncRef` if present.
    ///
    /// - `TableOutOfBounds` if `index >= active_size`
    /// - `UndefinedElement` if the slot is `None`
    #[inline]
    pub fn get(&self, index: u32) -> WasmResult<FuncRef> {
        let idx = index as usize;
        if idx >= self.active_size {
            return Err(WasmTrap::TableOutOfBounds);
        }
        self.entries
            .get(idx)
            .copied()
            .flatten()
            .ok_or(WasmTrap::UndefinedElement)
    }

    /// Set a table entry. Used during module initialization (element segments).
    ///
    /// Returns `Err(TableOutOfBounds)` if `index >= active_size`.
    #[inline]
    pub fn set(&mut self, index: u32, entry: Option<FuncRef>) -> WasmResult<()> {
        let idx = index as usize;
        if idx >= self.active_size {
            return Err(WasmTrap::TableOutOfBounds);
        }
        match self.entries.get_mut(idx) {
            Some(slot) => {
                *slot = entry;
                Ok(())
            }
            None => Err(WasmTrap::TableOutOfBounds),
        }
    }

    /// Initialize table entries from element segment data.
    ///
    /// Writes `entries` (each as `(type_index, func_index)`) into consecutive
    /// slots starting at `base`. Replaces per-slot `set()` calls in generated
    /// constructors and propagates bounds errors via `?` instead of panicking.
    ///
    /// # Errors
    /// Returns `Err(TableOutOfBounds)` if any slot index is out of range.
    #[inline(always)]
    pub fn init_elements(&mut self, base: u32, entries: &[(u32, u32)]) -> WasmResult<()> {
        init_elements_inner(&mut self.entries, self.active_size, base, entries)
    }

    /// Grow the table by `delta` slots, filling new slots with `init`.
    /// Returns the previous size, or -1 on failure.
    pub fn grow(&mut self, delta: u32, init: Option<FuncRef>) -> i32 {
        let old = self.active_size;
        let new = old.wrapping_add(delta as usize);
        if new > MAX_SIZE {
            return -1;
        }
        for slot in &mut self.entries[old..new] {
            *slot = init;
        }
        self.active_size = new;
        old as i32
    }
}

// ── Non-generic inner function (outline pattern, §13.3) ──────────────────────

#[inline(never)]
fn init_elements_inner(
    slots: &mut [Option<FuncRef>],
    active_size: usize,
    base: u32,
    entries: &[(u32, u32)],
) -> WasmResult<()> {
    for (i, &(type_index, func_index)) in entries.iter().enumerate() {
        let idx = (base as usize)
            .checked_add(i)
            .ok_or(WasmTrap::TableOutOfBounds)?;
        if idx >= active_size {
            return Err(WasmTrap::TableOutOfBounds);
        }
        match slots.get_mut(idx) {
            Some(slot) => {
                *slot = Some(FuncRef {
                    type_index,
                    func_index,
                })
            }
            None => return Err(WasmTrap::TableOutOfBounds),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_ref(type_idx: u32, func_idx: u32) -> FuncRef {
        FuncRef {
            type_index: type_idx,
            func_index: func_idx,
        }
    }

    #[test]
    fn new_table_is_empty() {
        let table = Table::<8>::try_new(4).unwrap();
        assert_eq!(table.size(), 4);
        // All slots are None → UndefinedElement
        assert_eq!(table.get(0), Err(WasmTrap::UndefinedElement));
        assert_eq!(table.get(3), Err(WasmTrap::UndefinedElement));
    }

    #[test]
    fn get_out_of_bounds() {
        let table = Table::<8>::try_new(4).unwrap();
        assert_eq!(table.get(4), Err(WasmTrap::TableOutOfBounds));
        assert_eq!(table.get(100), Err(WasmTrap::TableOutOfBounds));
    }

    #[test]
    fn set_and_get() {
        let mut table = Table::<8>::try_new(4).unwrap();
        let fr = sample_ref(0, 5);
        table.set(2, Some(fr)).unwrap();

        let got = table.get(2).unwrap();
        assert_eq!(got.type_index, 0);
        assert_eq!(got.func_index, 5);
    }

    #[test]
    fn set_out_of_bounds() {
        let mut table = Table::<8>::try_new(4).unwrap();
        assert_eq!(
            table.set(4, Some(sample_ref(0, 0))),
            Err(WasmTrap::TableOutOfBounds)
        );
    }

    #[test]
    fn set_none_clears_entry() {
        let mut table = Table::<8>::try_new(4).unwrap();
        table.set(1, Some(sample_ref(0, 3))).unwrap();
        assert!(table.get(1).is_ok());
        table.set(1, None).unwrap();
        assert_eq!(table.get(1), Err(WasmTrap::UndefinedElement));
    }

    #[test]
    fn grow_success() {
        let mut table = Table::<8>::try_new(2).unwrap();
        let old = table.grow(3, None);
        assert_eq!(old, 2);
        assert_eq!(table.size(), 5);
        // New slots are None
        assert_eq!(table.get(4), Err(WasmTrap::UndefinedElement));
    }

    #[test]
    fn grow_with_init() {
        let mut table = Table::<8>::try_new(2).unwrap();
        let fr = sample_ref(1, 7);
        table.grow(2, Some(fr));
        let got = table.get(3).unwrap();
        assert_eq!(got.func_index, 7);
    }

    #[test]
    fn grow_beyond_max_fails() {
        let mut table = Table::<4>::try_new(2).unwrap();
        assert_eq!(table.grow(3, None), -1); // would be 5 > 4
        assert_eq!(table.size(), 2); // unchanged
    }

    // ── init_elements ──

    #[test]
    fn init_elements_writes_entries() {
        let mut table = Table::<8>::try_new(4).unwrap();
        table.init_elements(0, &[(1, 2), (3, 4)]).unwrap();
        let e0 = table.get(0).unwrap();
        assert_eq!(e0.type_index, 1);
        assert_eq!(e0.func_index, 2);
        let e1 = table.get(1).unwrap();
        assert_eq!(e1.type_index, 3);
        assert_eq!(e1.func_index, 4);
    }

    #[test]
    fn init_elements_empty_is_noop() {
        let mut table = Table::<4>::try_new(4).unwrap();
        assert!(table.init_elements(0, &[]).is_ok());
        assert_eq!(table.get(0), Err(WasmTrap::UndefinedElement));
    }

    #[test]
    fn init_elements_at_base_offset() {
        let mut table = Table::<8>::try_new(6).unwrap();
        table.init_elements(3, &[(0, 5)]).unwrap();
        assert_eq!(table.get(3).unwrap().func_index, 5);
        assert_eq!(table.get(0), Err(WasmTrap::UndefinedElement));
    }

    #[test]
    fn init_elements_out_of_bounds() {
        let mut table = Table::<4>::try_new(4).unwrap();
        // base=3, 2 entries → slots 3 and 4; slot 4 is OOB
        let result = table.init_elements(3, &[(0, 0), (0, 1)]);
        assert_eq!(result, Err(WasmTrap::TableOutOfBounds));
    }

    #[test]
    fn init_elements_exactly_fills_table() {
        let mut table = Table::<4>::try_new(4).unwrap();
        table
            .init_elements(0, &[(0, 0), (0, 1), (0, 2), (0, 3)])
            .unwrap();
        assert_eq!(table.get(3).unwrap().func_index, 3);
    }

    #[test]
    fn try_new_fails_if_initial_exceeds_max() {
        let result = Table::<4>::try_new(5);
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(crate::ConstructionError::TableInitialSizeExceedsMax { initial: 5, max: 4 })
        ));
    }
}

// ── Kani Formal Verification Proofs ──────────────────────────────────────

#[cfg(kani)]
mod proofs {
    use super::*;

    /// Proof: get never panics, only returns Ok or Err.
    #[kani::proof]
    #[kani::unwind(1)]
    fn get_never_panics() {
        let table = Table::<8>::new(4);
        let index: u32 = kani::any();
        let _ = table.get(index);
        // Kani verifies this doesn't panic for all possible indices
    }

    /// Proof: set never panics, only returns Ok or Err.
    #[kani::proof]
    #[kani::unwind(1)]
    fn set_never_panics() {
        let mut table = Table::<8>::new(4);
        let index: u32 = kani::any();
        let type_index: u32 = kani::any();
        let func_index: u32 = kani::any();
        let entry = Some(FuncRef {
            type_index,
            func_index,
        });
        let _ = table.set(index, entry);
    }

    /// Proof: grow respects MAX_SIZE — active_size never exceeds it.
    #[kani::proof]
    #[kani::unwind(5)]
    fn grow_respects_max_size() {
        let mut table = Table::<4>::new(1);
        let delta: u32 = kani::any();

        let old_size = table.size();
        let result = table.grow(delta, None);

        // active_size must never exceed MAX_SIZE
        kani::assert(table.size() <= 4, "active_size must not exceed MAX_SIZE");

        // If grow succeeded, result should be old size
        if result >= 0 {
            kani::assert(result == old_size as i32, "grow returns old size");
            let new_expected = old_size as u64 + delta as u64;
            if new_expected <= 4 {
                kani::assert(
                    table.size() == new_expected as usize,
                    "grow updates active_size correctly",
                );
            }
        } else {
            // If grow failed, active_size unchanged
            kani::assert(
                table.size() == old_size,
                "failed grow leaves active_size unchanged",
            );
        }
    }

    /// Proof: grow returns -1 if new size would exceed MAX_SIZE.
    #[kani::proof]
    #[kani::unwind(4)]
    fn grow_fails_beyond_max() {
        let mut table = Table::<4>::new(2);
        // Try to grow by 3 slots: 2 + 3 = 5 > 4 (MAX_SIZE)
        let result = table.grow(3, None);
        kani::assert(result == -1, "grow beyond MAX_SIZE returns -1");
        kani::assert(table.size() == 2, "failed grow leaves size unchanged");
    }

    /// Proof: set followed by get returns the same value.
    #[kani::proof]
    #[kani::unwind(1)]
    fn set_get_roundtrip() {
        let mut table = Table::<8>::new(4);
        let index: u32 = kani::any();
        let type_index: u32 = kani::any();
        let func_index: u32 = kani::any();

        let entry = FuncRef {
            type_index,
            func_index,
        };

        // If set succeeds, get should return the same entry
        if table.set(index, Some(entry)).is_ok() {
            let result = table.get(index);
            kani::assert(result.is_ok(), "get succeeds after successful set");
            let retrieved = result.unwrap();
            kani::assert(retrieved.type_index == type_index, "type_index preserved");
            kani::assert(retrieved.func_index == func_index, "func_index preserved");
        }
    }

    /// Proof: get out of bounds (index >= active_size) returns TableOutOfBounds.
    #[kani::proof]
    #[kani::unwind(1)]
    fn get_out_of_bounds_returns_error() {
        let table = Table::<8>::new(4);
        // Access at or beyond active_size
        let result1 = table.get(4);
        kani::assert(
            result1 == Err(WasmTrap::TableOutOfBounds),
            "get at active_size is out of bounds",
        );

        let result2 = table.get(100);
        kani::assert(
            result2 == Err(WasmTrap::TableOutOfBounds),
            "get beyond active_size is out of bounds",
        );
    }

    /// Proof: set out of bounds (index >= active_size) returns TableOutOfBounds.
    #[kani::proof]
    #[kani::unwind(1)]
    fn set_out_of_bounds_returns_error() {
        let mut table = Table::<8>::new(4);
        let entry = FuncRef {
            type_index: 0,
            func_index: 0,
        };

        let result = table.set(4, Some(entry));
        kani::assert(
            result == Err(WasmTrap::TableOutOfBounds),
            "set at active_size is out of bounds",
        );
    }

    /// Proof: get on empty slot returns UndefinedElement.
    #[kani::proof]
    #[kani::unwind(1)]
    fn get_empty_slot_returns_undefined() {
        let table = Table::<8>::new(4);
        // All slots start empty
        let result = table.get(0);
        kani::assert(
            result == Err(WasmTrap::UndefinedElement),
            "get on empty slot returns UndefinedElement",
        );
    }

    /// Proof: set None clears a slot.
    #[kani::proof]
    #[kani::unwind(1)]
    fn set_none_clears_slot() {
        let mut table = Table::<8>::new(4);
        let entry = FuncRef {
            type_index: 1,
            func_index: 5,
        };

        // Set to Some, then clear with None
        table.set(1, Some(entry)).unwrap();
        kani::assert(table.get(1).is_ok(), "slot is set");

        table.set(1, None).unwrap();
        let result = table.get(1);
        kani::assert(
            result == Err(WasmTrap::UndefinedElement),
            "set None clears the slot",
        );
    }

    /// Proof: grow initializes new slots with init value.
    #[kani::proof]
    #[kani::unwind(3)]
    fn grow_initializes_new_slots() {
        let mut table = Table::<8>::new(2);
        let init = FuncRef {
            type_index: 7,
            func_index: 42,
        };

        let result = table.grow(2, Some(init));

        if result >= 0 {
            // New slots should be initialized with init value
            let slot2 = table.get(2);

            if slot2.is_ok() {
                let val = slot2.unwrap();
                kani::assert(
                    val.type_index == 7 && val.func_index == 42,
                    "new slot initialized with init value",
                );
            }
        }
    }

    /// Proof: size() returns active_size.
    #[kani::proof]
    #[kani::unwind(1)]
    fn size_returns_active_size() {
        let table = Table::<8>::new(5);
        kani::assert(table.size() == 5, "size() returns active_size");
    }

    /// Proof: successful get requires index < active_size.
    #[kani::proof]
    #[kani::unwind(1)]
    fn get_success_implies_valid_index() {
        let mut table = Table::<8>::new(4);
        // Set a valid entry
        let entry = FuncRef {
            type_index: 0,
            func_index: 1,
        };
        table.set(0, Some(entry)).unwrap();

        let index: u32 = kani::any();
        let result = table.get(index);

        // If get succeeds with a valid entry, index must be < active_size
        if result.is_ok() {
            kani::assert(
                (index as usize) < table.size(),
                "successful get implies index < active_size",
            );
        }
    }

    /// Proof: successful set requires index < active_size.
    #[kani::proof]
    #[kani::unwind(1)]
    fn set_success_implies_valid_index() {
        let mut table = Table::<8>::new(4);
        let index: u32 = kani::any();
        let entry = FuncRef {
            type_index: 0,
            func_index: 0,
        };

        let result = table.set(index, Some(entry));

        if result.is_ok() {
            kani::assert(
                (index as usize) < table.size(),
                "successful set implies index < active_size",
            );
        }
    }
}
