//! Wasm module containers — `Module` and `LibraryModule`.
//!
//! A `Module` owns its own linear memory (like a POSIX process).
//! A `LibraryModule` borrows the caller's memory (like a shared library).
//!
//! Both contain module-specific globals (`G`, transpiler-generated) and an
//! indirect call table. The transpiler generates `impl` blocks with the
//! concrete exported/internal functions.

use crate::memory::IsolatedMemory;
use crate::table::Table;

/// A module that defines its own memory (§4.1).
///
/// - `G`: transpiler-generated globals struct (one typed field per Wasm global)
/// - `MAX_PAGES`: maximum linear memory size (Wasm pages, 64 KiB each)
/// - `TABLE_SIZE`: maximum indirect call table entries
///
/// The transpiler generates an `impl` block on this struct with the
/// module's exported and internal functions.
pub struct Module<G, const MAX_PAGES: usize, const TABLE_SIZE: usize> {
    /// Owned linear memory — isolated by the Rust type system.
    pub memory: IsolatedMemory<MAX_PAGES>,
    /// Module-level global variables.
    pub globals: G,
    /// Indirect call table for `call_indirect`.
    pub table: Table<TABLE_SIZE>,
}

impl<G, const MAX_PAGES: usize, const TABLE_SIZE: usize> Module<G, MAX_PAGES, TABLE_SIZE> {
    /// Create a new module with the given initial memory size, globals, and table.
    ///
    /// The transpiler generates a wrapper that calls this with the correct
    /// initial values derived from the Wasm binary (data segments, element
    /// segments, global initializers).
    ///
    /// # Errors
    /// Returns `ConstructionError` if `initial_pages` exceeds `MAX_PAGES`.
    #[inline(never)]
    pub fn try_new(
        initial_pages: usize,
        globals: G,
        table: Table<TABLE_SIZE>,
    ) -> Result<Self, crate::ConstructionError> {
        Ok(Self {
            memory: IsolatedMemory::try_new(initial_pages)?,
            globals,
            table,
        })
    }

    /// Initialize a `Module` in-place within a caller-provided slot.
    ///
    /// Unlike `try_new`, this writes directly into `slot` without ever creating
    /// a large `Result<Self, E>` on the call stack. Use this when `MAX_PAGES`
    /// is large, to avoid stack overflow in debug builds.
    ///
    /// # Errors
    /// Returns `ConstructionError` if `initial_pages` exceeds `MAX_PAGES`.
    #[inline(never)]
    pub fn try_init(
        slot: &mut core::mem::MaybeUninit<Self>,
        initial_pages: usize,
        globals: G,
        table: Table<TABLE_SIZE>,
    ) -> Result<(), crate::ConstructionError> {
        let ptr = slot.as_mut_ptr();
        // SAFETY: ptr comes from MaybeUninit so it is valid for writes and
        // correctly aligned. We initialise all three fields before the caller
        // can call assume_init on the slot. The cast of the memory field pointer
        // to *mut MaybeUninit<IsolatedMemory<MAX_PAGES>> is valid because
        // MaybeUninit<T> has the same memory layout as T (guaranteed by the
        // standard library), and the field is currently uninitialized.
        unsafe {
            IsolatedMemory::try_init(
                &mut *(core::ptr::addr_of_mut!((*ptr).memory)
                    as *mut core::mem::MaybeUninit<IsolatedMemory<MAX_PAGES>>),
                initial_pages,
            )?;
            core::ptr::addr_of_mut!((*ptr).globals).write(globals);
            core::ptr::addr_of_mut!((*ptr).table).write(table);
        }
        Ok(())
    }
}

/// A module that does NOT define its own memory (§4.1).
///
/// Operates on borrowed memory from the caller, like a shared library
/// using the host process's address space. Rust's borrow checker enforces
/// that the library cannot retain the memory reference beyond a call.
///
/// - `G`: transpiler-generated globals struct
/// - `TABLE_SIZE`: maximum indirect call table entries
pub struct LibraryModule<G, const TABLE_SIZE: usize> {
    /// Module-level global variables.
    pub globals: G,
    /// Indirect call table for `call_indirect`.
    pub table: Table<TABLE_SIZE>,
}

impl<G, const TABLE_SIZE: usize> LibraryModule<G, TABLE_SIZE> {
    /// Create a new library module with the given globals and table.
    #[inline]
    pub fn new(globals: G, table: Table<TABLE_SIZE>) -> Self {
        Self { globals, table }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::table::FuncRef;
    use crate::WasmTrap;

    /// Example transpiler-generated globals struct.
    #[derive(Debug, Default, PartialEq)]
    struct TestGlobals {
        g0: i32,
        g1: i64,
    }

    #[test]
    fn module_new_owns_memory() {
        let module = Module::<TestGlobals, 2, 0>::try_new(
            2,
            TestGlobals { g0: 42, g1: -1 },
            Table::try_new(0).unwrap(),
        )
        .unwrap();
        assert_eq!(module.memory.page_count(), 2);
        assert_eq!(module.globals.g0, 42);
        assert_eq!(module.globals.g1, -1);
        assert_eq!(module.table.size(), 0);
    }

    #[test]
    fn module_memory_is_isolated() {
        let mut m1 = Module::<TestGlobals, 2, 0>::try_new(
            1,
            TestGlobals::default(),
            Table::try_new(0).unwrap(),
        )
        .unwrap();
        let m2 = Module::<TestGlobals, 2, 0>::try_new(
            1,
            TestGlobals::default(),
            Table::try_new(0).unwrap(),
        )
        .unwrap();

        // Write to m1's memory — m2 is unaffected.
        m1.memory.store_i32(0, 0xDEAD_BEEF_u32 as i32).unwrap();
        assert_eq!(m2.memory.load_i32(0).unwrap(), 0);
    }

    #[test]
    fn library_module_borrows_caller_memory() {
        let mut caller = Module::<TestGlobals, 2, 0>::try_new(
            1,
            TestGlobals::default(),
            Table::try_new(0).unwrap(),
        )
        .unwrap();
        let lib = LibraryModule::<TestGlobals, 0>::new(
            TestGlobals { g0: 7, g1: 0 },
            Table::try_new(0).unwrap(),
        );

        // Caller writes to its own memory.
        caller.memory.store_i32(0, 99).unwrap();

        // Library can borrow caller's memory and read what was written.
        // (In real transpiled code, this borrow happens inside generated methods.)
        let val = caller.memory.load_i32(0).unwrap();
        assert_eq!(val, 99);
        assert_eq!(lib.globals.g0, 7);
    }

    #[test]
    fn module_with_table() {
        let mut table = Table::<4>::try_new(2).unwrap();
        table
            .set(
                0,
                Some(FuncRef {
                    type_index: 0,
                    func_index: 3,
                }),
            )
            .unwrap();

        let module = Module::<(), 2, 4>::try_new(1, (), table).unwrap();
        let entry = module.table.get(0).unwrap();
        assert_eq!(entry.func_index, 3);
        assert_eq!(module.table.get(1), Err(WasmTrap::UndefinedElement));
    }

    #[test]
    fn library_module_no_globals() {
        // G = () for modules with no globals
        let lib = LibraryModule::<(), 0>::new((), Table::try_new(0).unwrap());
        assert_eq!(lib.globals, ());
    }
}
