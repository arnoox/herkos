.. _module-representation:

Module Representation
=====================

This section describes how WebAssembly concepts map to Rust types. This is the core abstraction layer — everything else (transpilation, integration, performance) builds on these types.

Memory Model
------------

.. spec:: Memory Model
   :id: SPEC_MEMORY_MODEL
   :satisfies: REQ_MEM_PAGE_MODEL, REQ_MEM_COMPILE_TIME_SIZE, REQ_MEM_BOUNDS_CHECKED, REQ_MEM_GROW_NO_ALLOC
   :tags: memory

   How WebAssembly linear memory maps to ``IsolatedMemory<MAX_PAGES>`` in Rust.

Page Model
~~~~~~~~~~

WebAssembly linear memory is organized in pages of 64 KiB (65,536 bytes). A Wasm module declares an initial page count and an optional maximum page count.

.. code-block:: text

   Page size:    64 KiB (defined by the WebAssembly specification)
   Initial size: declared in the Wasm module (e.g., 16 pages = 1 MiB)
   Maximum size: declared in the Wasm module (e.g., 256 pages = 16 MiB)

Rust Representation: ``IsolatedMemory<MAX_PAGES>``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

*Implementation:* ``crates/herkos-runtime/src/memory.rs``

.. code-block:: rust

   const PAGE_SIZE: usize = 65536;

   struct IsolatedMemory<const MAX_PAGES: usize> {
       pages: [[u8; PAGE_SIZE]; MAX_PAGES],
       active_pages: usize,
   }

**Design decisions**:

.. list-table::
   :header-rows: 1
   :widths: 40 60

   * - Decision
     - Rationale
   * - ``MAX_PAGES`` const generic
     - No heap allocation, ``no_std`` compatible, enables monomorphization
   * - ``active_pages`` runtime tracking
     - Starts at initial page count, grows via ``memory.grow``, bounds-checks against this
   * - 2D array ``[[u8; PAGE_SIZE]; MAX_PAGES]``
     - Avoids unstable ``generic_const_exprs``. ``as_flattened()`` provides flat ``&[u8]`` views (stable Rust 1.80+)
   * - No maximum → CLI configurable
     - If the Wasm module declares no maximum, the transpiler picks a default (configurable via ``--max-pages``)

Memory Access API
~~~~~~~~~~~~~~~~~

All memory operations are flat — no ``MemoryView`` wrappers. One method per Wasm type, avoiding monomorphization of inner functions:

.. code-block:: rust

   impl<const MAX_PAGES: usize> IsolatedMemory<MAX_PAGES> {
       // Safe: bounds-checked against active_pages * PAGE_SIZE
       fn load_i32(&self, offset: usize) -> WasmResult<i32>;
       fn load_i64(&self, offset: usize) -> WasmResult<i64>;
       fn load_u8(&self, offset: usize) -> WasmResult<u8>;
       fn load_u16(&self, offset: usize) -> WasmResult<u16>;
       fn load_f32(&self, offset: usize) -> WasmResult<f32>;
       fn load_f64(&self, offset: usize) -> WasmResult<f64>;
       fn store_i32(&mut self, offset: usize, value: i32) -> WasmResult<()>;
       fn store_i64(&mut self, offset: usize, value: i64) -> WasmResult<()>;
       // ... and store_u8, store_u16, store_f32, store_f64
   }

Read-only guarantees are not a Wasm primitive — they are an analysis result. Static analysis can prove that certain regions (e.g., ``.rodata`` data segments) are never targeted by store instructions. This is relevant to the future verified backend (see :doc:`/FUTURE`).

``memory.grow`` Semantics
~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: rust

   impl<const MAX_PAGES: usize> IsolatedMemory<MAX_PAGES> {
       fn grow(&mut self, delta: u32) -> i32 {
           let old = self.active_pages;
           let new = old.wrapping_add(delta as usize);
           if new > MAX_PAGES {
               return -1;
           }
           for page in &mut self.pages[old..new] {
               page.fill(0);
           }
           self.active_pages = new;
           old as i32
       }
   }

No allocation occurs. New pages are zero-initialized per the Wasm spec.

Linear Memory Layout
~~~~~~~~~~~~~~~~~~~~

When C/C++ compiles to Wasm, the compiler organizes linear memory into conventional regions:

.. code-block:: text

   ┌─────────────────────────────────────────┐ MAX_PAGES * PAGE_SIZE
   │           (unused / growable)           │
   ├─────────────────────────────────────────┤ ← __stack_pointer (grows ↓)
   │           Shadow Stack                  │
   │   (local variables, large structs,      │
   │    spills, return values)               │
   ├─────────────────────────────────────────┤
   │           Heap (grows ↑)                │
   │   (malloc / C++ new)                    │
   ├─────────────────────────────────────────┤
   │           Data Segments                 │
   │   (.data, .rodata, .bss)                │
   └─────────────────────────────────────────┘ 0

Key points:

- Wasm's value stack only holds scalars (i32, i64, f32, f64). Large structs and address-taken locals live in the **shadow stack** in linear memory.
- A "pure" C function returning a large struct actually writes to its shadow stack frame via ``i32.store`` instructions — not pure with respect to memory.

Compile-Time Guarantees
~~~~~~~~~~~~~~~~~~~~~~~

- **Spatial safety**: all memory accesses bounds-checked against ``active_pages * PAGE_SIZE``
- **Temporal safety**: Rust's lifetime system prevents use-after-free
- **Isolation**: each module has its own ``IsolatedMemory`` instance — distinct types, distinct backing arrays, no cross-module access possible

Module Types
------------

.. spec:: Module Types
   :id: SPEC_MODULE_TYPES
   :satisfies: REQ_MOD_TWO_TYPES
   :tags: module

   Process-like (owns memory) and library-like (borrows memory) module representations.

*Implementation:* ``crates/herkos-runtime/src/module.rs``

.. code-block:: text

         ┌─────────────────────────────────────┐
         │         Module Taxonomy             │
         ├──────────────────┬──────────────────┤
         │                  │                  │
   ┌─────┴───-──┐    ┌──────┴──────┐   ┌───-───┴─────┐
   │  Module    │    │ Library     │   │ Pure        │
   │ (owns mem) │    │ Module      │   │ (no memory) │
   │            │    │ (borrows)   │   │             │
   └────────────┘    └─────────────┘   └─────────────┘
   Like a process    Like a shared      Pure computation
                     library

Process-like Module (owns memory)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: rust

   struct Module<G, const MAX_PAGES: usize, const TABLE_SIZE: usize> {
       memory: IsolatedMemory<MAX_PAGES>,
       globals: G,
       table: Table<TABLE_SIZE>,
   }

Library Module (borrows memory)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: rust

   struct LibraryModule<G, const TABLE_SIZE: usize> {
       globals: G,
       table: Table<TABLE_SIZE>,
       // no memory field — uses caller's memory
   }

.. list-table::
   :header-rows: 1

   * - Wasm declaration
     - Rust representation
     - Analogy
   * - Module defines memory
     - ``Module`` owns ``IsolatedMemory``
     - POSIX process
   * - Module imports memory
     - ``LibraryModule`` borrows ``&mut IsolatedMemory``
     - Shared library
   * - Module has no memory
     - ``LibraryModule`` with no memory parameter
     - Pure computation

Globals and Tables
------------------

.. spec:: Globals and Tables
   :id: SPEC_GLOBALS_TABLES
   :satisfies: REQ_MOD_GLOBALS, REQ_MOD_TABLE
   :tags: module, global, table

   Globals as typed struct fields, tables for indirect call dispatch.

*Implementation:* ``crates/herkos-runtime/src/table.rs``

Globals
~~~~~~~

.. code-block:: rust

   // Generated by the transpiler — one struct per module
   struct Globals {
       g0: i32,      // (global (mut i32) ...) — mutable, lives in struct
       // g1 is immutable → emitted as `const G1: i64 = 42;` instead
   }

Tables
~~~~~~

.. code-block:: rust

   struct Table<const MAX_SIZE: usize> {
       entries: [Option<FuncRef>; MAX_SIZE],
       active_size: usize,
   }

   struct FuncRef {
       type_index: u32,   // canonical type index for signature check
       func_index: u32,   // index into module function space → match dispatch
   }

Tables are initialized from element segments during module construction:

.. code-block:: rust

   // From: (elem (i32.const 0) $add $sub $mul)
   let mut table = Table::try_new(3);
   table.set(0, Some(FuncRef { type_index: 0, func_index: 0 })).unwrap();
   table.set(1, Some(FuncRef { type_index: 0, func_index: 1 })).unwrap();
   table.set(2, Some(FuncRef { type_index: 0, func_index: 2 })).unwrap();

Imports as Trait Bounds
-----------------------

.. spec:: Imports as Trait Bounds
   :id: SPEC_IMPORTS
   :satisfies: REQ_CAP_IMPORTS, REQ_CAP_ZERO_COST
   :tags: capability, import

   Wasm imports mapped to Rust trait bounds for capability-based security.

Capabilities are Rust **traits**, not bitflags. A Wasm module's imports become trait bounds on its functions:

.. code-block:: rust

   // Generated from: (import "env" "socket_open" (func ...))
   //                 (import "env" "socket_read" (func ...))
   trait SocketOps {
       fn socket_open(&mut self, domain: i32, sock_type: i32) -> WasmResult<i32>;
       fn socket_read(&mut self, fd: i32, buf_ptr: u32, len: u32) -> WasmResult<i32>;
   }

   // Module function that calls socket imports requires the trait:
   fn send_data<H: SocketOps + FileOps>(
       host: &mut H,
       memory: &mut IsolatedMemory<MAX_PAGES>,
       // ...
   ) -> WasmResult<i32> {
       let sock = host.socket_open(2, 1)?;
       // ...
   }

   // No imports → no host parameter, pure computation:
   fn pure_math(a: i32, b: i32) -> i32 { a.wrapping_add(b) }

.. list-table::
   :header-rows: 1

   * - Aspect
     - Bitflags (``const CAPS: u64``)
     - Traits
   * - Granularity
     - Coarse (1 bit = 1 class)
     - Fine (exact function signatures)
   * - Compile-time checking
     - Fails if bit not set
     - Fails if trait not implemented
   * - Error messages
     - Opaque bit mismatch
     - Clear: "trait ``SocketOps`` not implemented"
   * - Runtime cost
     - Zero
     - Zero (monomorphization)
   * - Extensibility
     - Limited to 64 bits
     - Unlimited
   * - Inter-module linking
     - Not supported
     - Natural via trait composition

Exports as Trait Implementations
--------------------------------

.. spec:: Exports as Trait Implementations
   :id: SPEC_EXPORTS
   :satisfies: REQ_CAP_EXPORTS, REQ_CAP_ZERO_COST
   :tags: capability, export

   Wasm exports mapped to Rust trait implementations.

.. code-block:: rust

   // Generated from: (export "transform" (func $transform))
   trait ImageLibExports {
       fn transform(&mut self, ptr: u32, len: u32) -> WasmResult<i32>;
       fn init(&mut self) -> WasmResult<()>;
   }

   impl<const MAX_PAGES: usize> ImageLibExports for ImageModule<MAX_PAGES> {
       fn transform(&mut self, ptr: u32, len: u32) -> WasmResult<i32> {
           func_transform(&mut self.memory, &mut self.globals, ptr, len)
       }
       fn init(&mut self) -> WasmResult<()> {
           func_init(&mut self.memory, &mut self.globals)
       }
   }

WASI Support
------------

WASI is a standard set of import traits shipped by ``herkos-runtime``:

.. code-block:: rust

   trait WasiFd {
       fn fd_read(&mut self, fd: i32, iovs: u32, iovs_len: u32, nread: u32) -> WasmResult<i32>;
       fn fd_write(&mut self, fd: i32, iovs: u32, iovs_len: u32, nwritten: u32) -> WasmResult<i32>;
       fn fd_close(&mut self, fd: i32) -> WasmResult<i32>;
       // ...
   }

   trait WasiClock {
       fn clock_time_get(&mut self, clock_id: i32, precision: i64, time: u32) -> WasmResult<i32>;
   }

   trait WasiRandom {
       fn random_get(&mut self, buf: u32, len: u32) -> WasmResult<i32>;
   }

The host implements whichever subset it supports:

.. code-block:: rust

   // Bare-metal: only fd_write (UART) and clock
   struct EmbeddedHost { /* ... */ }
   impl WasiFd for EmbeddedHost { /* UART-backed */ }
   impl WasiClock for EmbeddedHost { /* hardware timer */ }

   // Full POSIX: everything
   struct PosixHost { /* ... */ }
   impl WasiFd for PosixHost { /* real file ops */ }
   impl WasiClock for PosixHost { /* clock_gettime */ }
   impl WasiRandom for PosixHost { /* /dev/urandom */ }

Custom platform-specific capabilities beyond WASI are just additional traits (e.g., ``GpioOps``, ``CanBusOps``).

Isolation Guarantees
--------------------

.. spec:: Isolation Guarantees
   :id: SPEC_ISOLATION
   :satisfies: REQ_ISOLATION_COMPILE_TIME, REQ_FREEDOM_FROM_INTERFERENCE, REQ_ISOLATION_SPATIAL, REQ_ISOLATION_CAPABILITY
   :tags: isolation, safety

   Compile-time freedom from interference via Rust ownership model.

The ownership model enforces freedom from interference structurally:

.. code-block:: text

   ┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
   │   Module A      │     │   Module B      │     │   Library C     │
   │                 │     │                 │     │                 │
   │ ┌─────────────┐ │     │ ┌─────────────┐ │     │  (no memory)    │
   │ │ Memory A    │ │     │ │ Memory B    │ │     │                 │
   │ │ (owned)     │ │     │ │ (owned)     │ │     │  Borrows caller │
   │ └─────────────┘ │     │ └─────────────┘ │     │  memory for     │
   │ ┌─────────────┐ │     │ ┌─────────────┐ │     │  duration of    │
   │ │ Globals A   │ │     │ │ Globals B   │ │     │  each call      │
   │ └─────────────┘ │     │ └─────────────┘ │     │                 │
   └─────────────────┘     └─────────────────┘     └─────────────────┘
          ✗ cannot               ✗ cannot              ✓ borrows
          access B               access A               one at a time

1. **Module with its own memory**: cannot access another module's memory — each owns a distinct ``IsolatedMemory`` instance
2. **Library module**: can only access the specific memory it was handed via ``&mut`` borrow. Cannot hold the reference past the call (lifetime enforced), cannot access a different module's memory
3. **Pure module**: no memory at all — the type system provides no memory access methods

Inter-module calls lend memory for the duration of the call:

.. code-block:: rust

   let mut app = Module::<AppGlobals, 256, 0>::new(16, AppGlobals::default(), table)?;
   let mut lib = LibraryModule::<LibGlobals, 0>::new(LibGlobals::default(), table)?;

   // Caller's memory is borrowed for this call only.
   // Rust borrow checker guarantees the library cannot store the reference.
   let result = lib.call_export_transform(&mut app.memory, ptr, len)?;
