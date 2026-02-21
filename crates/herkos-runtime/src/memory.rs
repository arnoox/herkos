//! WebAssembly linear memory — `IsolatedMemory<const MAX_PAGES: usize>`.
//!
//! The backing array is `[[u8; PAGE_SIZE]; MAX_PAGES]` — a 2D array that
//! is contiguous in memory. We use `as_flattened()` (stable since Rust 1.80)
//! to get a flat `&[u8]` view for the inner functions.
//!
//! Note: the spec shows `[u8; MAX_PAGES * PAGE_SIZE]` but that requires the
//! unstable `generic_const_exprs` feature. The 2D array achieves identical
//! layout on stable Rust.
//!
//! Load/store operations use the **outline pattern** (§13.3): the generic
//! wrapper delegates to a non-generic inner function so that only one copy
//! of the actual bounds-checking logic exists in the binary.

use crate::{WasmResult, WasmTrap, PAGE_SIZE};

/// Isolated linear memory for a single Wasm module.
///
/// `MAX_PAGES` is the compile-time maximum (from the Wasm module's declared
/// maximum or a CLI override). The backing array is fully pre-allocated.
pub struct IsolatedMemory<const MAX_PAGES: usize> {
    /// Backing storage — `MAX_PAGES` pages of `PAGE_SIZE` bytes each.
    /// Contiguous in memory, identical layout to `[u8; MAX_PAGES * PAGE_SIZE]`.
    pages: [[u8; PAGE_SIZE]; MAX_PAGES],
    /// Number of currently active pages. Starts at `initial_pages`,
    /// incremented by `grow`. Accesses beyond `active_pages * PAGE_SIZE`
    /// are out-of-bounds traps.
    active_pages: usize,
}

impl<const MAX_PAGES: usize> IsolatedMemory<MAX_PAGES> {
    /// Create a new `IsolatedMemory` with `initial_pages` active.
    ///
    /// # Errors
    /// Returns `ConstructionError::MemoryInitialPagesExceedsMax` if `initial_pages > MAX_PAGES`.
    #[inline(never)]
    pub fn try_new(initial_pages: usize) -> Result<Self, crate::ConstructionError> {
        if initial_pages > MAX_PAGES {
            return Err(crate::ConstructionError::MemoryInitialPagesExceedsMax {
                initial: initial_pages,
                max: MAX_PAGES,
            });
        }
        Ok(Self {
            pages: [[0u8; PAGE_SIZE]; MAX_PAGES],
            active_pages: initial_pages,
        })
    }

    /// Current number of active pages.
    #[inline(always)]
    pub fn page_count(&self) -> usize {
        self.active_pages
    }

    /// Current active size in bytes.
    #[inline(always)]
    pub fn active_size(&self) -> usize {
        self.active_pages * PAGE_SIZE
    }

    /// Wasm `memory.grow` — returns previous page count, or -1 on failure.
    /// No allocation occurs: the backing array is already sized to `MAX_PAGES`.
    pub fn grow(&mut self, delta: u32) -> i32 {
        let old = self.active_pages;
        let new = old.wrapping_add(delta as usize);
        if new > MAX_PAGES {
            return -1;
        }
        // Zero-init the new pages (Wasm spec requires it).
        for page in &mut self.pages[old..new] {
            page.fill(0);
        }
        self.active_pages = new;
        old as i32
    }

    /// Wasm `memory.size` — returns current page count.
    #[inline(always)]
    pub fn size(&self) -> i32 {
        self.active_pages as i32
    }

    /// Flat read-only view of the full backing memory.
    #[inline(always)]
    fn flat(&self) -> &[u8] {
        self.pages.as_flattened()
    }

    /// Flat mutable view of the full backing memory.
    #[inline(always)]
    fn flat_mut(&mut self) -> &mut [u8] {
        self.pages.as_flattened_mut()
    }

    // ── Bulk memory operations ────────────────────────────────────────

    /// Wasm `memory.copy` — copy `len` bytes from `src` to `dst`.
    ///
    /// Semantics match `memmove`: overlapping source and destination regions
    /// are handled correctly. Traps (`OutOfBounds`) if either region extends
    /// beyond the current active memory.
    pub fn memory_copy(&mut self, dst: u32, src: u32, len: u32) -> WasmResult<()> {
        let active = self.active_size();
        let dst = dst as usize;
        let src = src as usize;
        let len = len as usize;
        if src.checked_add(len).is_none_or(|end| end > active)
            || dst.checked_add(len).is_none_or(|end| end > active)
        {
            return Err(WasmTrap::OutOfBounds);
        }
        self.flat_mut().copy_within(src..src + len, dst);
        Ok(())
    }

    // ── Bounds-checked (safe) load/store ──────────────────────────────

    /// Load an i32 from linear memory with bounds checking.
    #[inline(always)]
    pub fn load_i32(&self, offset: usize) -> WasmResult<i32> {
        load_i32_inner(self.flat(), self.active_size(), offset)
    }

    /// Load an i64 from linear memory with bounds checking.
    #[inline(always)]
    pub fn load_i64(&self, offset: usize) -> WasmResult<i64> {
        load_i64_inner(self.flat(), self.active_size(), offset)
    }

    /// Load a u8 (i32.load8_u) from linear memory with bounds checking.
    #[inline(always)]
    pub fn load_u8(&self, offset: usize) -> WasmResult<u8> {
        load_u8_inner(self.flat(), self.active_size(), offset)
    }

    /// Load a u16 (i32.load16_u) from linear memory with bounds checking.
    #[inline(always)]
    pub fn load_u16(&self, offset: usize) -> WasmResult<u16> {
        load_u16_inner(self.flat(), self.active_size(), offset)
    }

    /// Load an f32 from linear memory with bounds checking.
    #[inline(always)]
    pub fn load_f32(&self, offset: usize) -> WasmResult<f32> {
        load_f32_inner(self.flat(), self.active_size(), offset)
    }

    /// Load an f64 from linear memory with bounds checking.
    #[inline(always)]
    pub fn load_f64(&self, offset: usize) -> WasmResult<f64> {
        load_f64_inner(self.flat(), self.active_size(), offset)
    }

    /// Store an i32 into linear memory with bounds checking.
    #[inline(always)]
    pub fn store_i32(&mut self, offset: usize, value: i32) -> WasmResult<()> {
        let active = self.active_size();
        store_i32_inner(self.flat_mut(), active, offset, value)
    }

    /// Store an i64 into linear memory with bounds checking.
    #[inline(always)]
    pub fn store_i64(&mut self, offset: usize, value: i64) -> WasmResult<()> {
        let active = self.active_size();
        store_i64_inner(self.flat_mut(), active, offset, value)
    }

    /// Store a u8 (i32.store8) into linear memory with bounds checking.
    #[inline(always)]
    pub fn store_u8(&mut self, offset: usize, value: u8) -> WasmResult<()> {
        let active = self.active_size();
        store_u8_inner(self.flat_mut(), active, offset, value)
    }

    /// Store a u16 (i32.store16) into linear memory with bounds checking.
    #[inline(always)]
    pub fn store_u16(&mut self, offset: usize, value: u16) -> WasmResult<()> {
        let active = self.active_size();
        store_u16_inner(self.flat_mut(), active, offset, value)
    }

    /// Store an f32 into linear memory with bounds checking.
    #[inline(always)]
    pub fn store_f32(&mut self, offset: usize, value: f32) -> WasmResult<()> {
        let active = self.active_size();
        store_f32_inner(self.flat_mut(), active, offset, value)
    }

    /// Store an f64 into linear memory with bounds checking.
    #[inline(always)]
    pub fn store_f64(&mut self, offset: usize, value: f64) -> WasmResult<()> {
        let active = self.active_size();
        store_f64_inner(self.flat_mut(), active, offset, value)
    }

    /// Initialize a region of memory from a byte slice (Wasm data segment).
    ///
    /// Copies `data` into linear memory starting at `offset`. Equivalent to
    /// calling `store_u8` for each byte, but avoids emitting N separate
    /// function calls in generated code.
    ///
    /// # Errors
    /// Returns `Err(WasmTrap::OutOfBounds)` if `offset + data.len()` exceeds
    /// `active_pages * PAGE_SIZE`.
    #[inline(always)]
    pub fn init_data(&mut self, offset: usize, data: &[u8]) -> WasmResult<()> {
        let active = self.active_size();
        init_data_inner(self.flat_mut(), active, offset, data)
    }

    // ── Unchecked (verified) load/store ───────────────────────────────
    //
    // These skip bounds checking entirely. The caller MUST guarantee that
    // the access is in-bounds, justified by a formal proof.

    /// Load i32 without bounds checking.
    ///
    /// # Safety
    /// Caller must guarantee `offset + 3 < active_size()`.
    #[inline(always)]
    pub unsafe fn load_i32_unchecked(&self, offset: usize) -> i32 {
        load_i32_unchecked_inner(self.flat(), offset)
    }

    /// Load i64 without bounds checking.
    ///
    /// # Safety
    /// Caller must guarantee `offset + 7 < active_size()`.
    #[inline(always)]
    pub unsafe fn load_i64_unchecked(&self, offset: usize) -> i64 {
        load_i64_unchecked_inner(self.flat(), offset)
    }

    /// Store i32 without bounds checking.
    ///
    /// # Safety
    /// Caller must guarantee `offset + 3 < active_size()`.
    #[inline(always)]
    pub unsafe fn store_i32_unchecked(&mut self, offset: usize, value: i32) {
        store_i32_unchecked_inner(self.flat_mut(), offset, value)
    }

    /// Store i64 without bounds checking.
    ///
    /// # Safety
    /// Caller must guarantee `offset + 7 < active_size()`.
    #[inline(always)]
    pub unsafe fn store_i64_unchecked(&mut self, offset: usize, value: i64) {
        store_i64_unchecked_inner(self.flat_mut(), offset, value)
    }

    /// Read-only access to the active memory region.
    #[inline(always)]
    pub fn as_slice(&self) -> &[u8] {
        &self.flat()[..self.active_size()]
    }

    /// Mutable access to the active memory region.
    #[inline(always)]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        let size = self.active_size();
        &mut self.flat_mut()[..size]
    }
}

// ── Helpers ───────────────────────────────────────────────────────────

/// Bounds-check and return the sub-slice `memory[offset..offset+N]`.
/// Returns `Err(OutOfBounds)` on overflow or out-of-range — never panics.
#[inline(always)]
fn checked_slice(
    memory: &[u8],
    active_bytes: usize,
    offset: usize,
    len: usize,
) -> WasmResult<&[u8]> {
    let end = offset.checked_add(len).ok_or(WasmTrap::OutOfBounds)?;
    if end > active_bytes {
        return Err(WasmTrap::OutOfBounds);
    }
    // SAFETY: we just verified end <= active_bytes <= memory.len().
    // get() would also work but returns Option, adding another branch.
    // This is safe because the bounds are proven above.
    memory.get(offset..end).ok_or(WasmTrap::OutOfBounds)
}

/// Mutable variant of `checked_slice`.
#[inline(always)]
fn checked_slice_mut(
    memory: &mut [u8],
    active_bytes: usize,
    offset: usize,
    len: usize,
) -> WasmResult<&mut [u8]> {
    let end = offset.checked_add(len).ok_or(WasmTrap::OutOfBounds)?;
    if end > active_bytes {
        return Err(WasmTrap::OutOfBounds);
    }
    memory.get_mut(offset..end).ok_or(WasmTrap::OutOfBounds)
}

/// Convert a slice to a fixed-size array. Returns `Err(OutOfBounds)` if
/// the length doesn't match — never panics.
#[inline(always)]
fn to_array<const N: usize>(slice: &[u8]) -> WasmResult<[u8; N]> {
    slice.try_into().map_err(|_| WasmTrap::OutOfBounds)
}

// ── Non-generic inner functions (outline pattern, §13.3) ─────────────
//
// ONE copy of each function in the binary, regardless of how many
// `MAX_PAGES` instantiations exist. The generic wrappers above compile
// to a single call instruction each.
//
// No unwrap(), no indexing, no panic paths.

#[inline(never)]
fn load_i32_inner(memory: &[u8], active_bytes: usize, offset: usize) -> WasmResult<i32> {
    let s = checked_slice(memory, active_bytes, offset, 4)?;
    Ok(i32::from_le_bytes(to_array(s)?))
}

#[inline(never)]
fn load_i64_inner(memory: &[u8], active_bytes: usize, offset: usize) -> WasmResult<i64> {
    let s = checked_slice(memory, active_bytes, offset, 8)?;
    Ok(i64::from_le_bytes(to_array(s)?))
}

#[inline(never)]
fn load_u8_inner(memory: &[u8], active_bytes: usize, offset: usize) -> WasmResult<u8> {
    let s = checked_slice(memory, active_bytes, offset, 1)?;
    Ok(s[0])
}

#[inline(never)]
fn load_u16_inner(memory: &[u8], active_bytes: usize, offset: usize) -> WasmResult<u16> {
    let s = checked_slice(memory, active_bytes, offset, 2)?;
    Ok(u16::from_le_bytes(to_array(s)?))
}

#[inline(never)]
fn load_f32_inner(memory: &[u8], active_bytes: usize, offset: usize) -> WasmResult<f32> {
    let s = checked_slice(memory, active_bytes, offset, 4)?;
    Ok(f32::from_le_bytes(to_array(s)?))
}

#[inline(never)]
fn load_f64_inner(memory: &[u8], active_bytes: usize, offset: usize) -> WasmResult<f64> {
    let s = checked_slice(memory, active_bytes, offset, 8)?;
    Ok(f64::from_le_bytes(to_array(s)?))
}

#[inline(never)]
fn store_i32_inner(
    memory: &mut [u8],
    active_bytes: usize,
    offset: usize,
    value: i32,
) -> WasmResult<()> {
    let s = checked_slice_mut(memory, active_bytes, offset, 4)?;
    s.copy_from_slice(&value.to_le_bytes());
    Ok(())
}

#[inline(never)]
fn store_i64_inner(
    memory: &mut [u8],
    active_bytes: usize,
    offset: usize,
    value: i64,
) -> WasmResult<()> {
    let s = checked_slice_mut(memory, active_bytes, offset, 8)?;
    s.copy_from_slice(&value.to_le_bytes());
    Ok(())
}

#[inline(never)]
fn store_u8_inner(
    memory: &mut [u8],
    active_bytes: usize,
    offset: usize,
    value: u8,
) -> WasmResult<()> {
    let s = checked_slice_mut(memory, active_bytes, offset, 1)?;
    s[0] = value;
    Ok(())
}

#[inline(never)]
fn store_u16_inner(
    memory: &mut [u8],
    active_bytes: usize,
    offset: usize,
    value: u16,
) -> WasmResult<()> {
    let s = checked_slice_mut(memory, active_bytes, offset, 2)?;
    s.copy_from_slice(&value.to_le_bytes());
    Ok(())
}

#[inline(never)]
fn store_f32_inner(
    memory: &mut [u8],
    active_bytes: usize,
    offset: usize,
    value: f32,
) -> WasmResult<()> {
    let s = checked_slice_mut(memory, active_bytes, offset, 4)?;
    s.copy_from_slice(&value.to_le_bytes());
    Ok(())
}

#[inline(never)]
fn store_f64_inner(
    memory: &mut [u8],
    active_bytes: usize,
    offset: usize,
    value: f64,
) -> WasmResult<()> {
    let s = checked_slice_mut(memory, active_bytes, offset, 8)?;
    s.copy_from_slice(&value.to_le_bytes());
    Ok(())
}

#[inline(never)]
fn init_data_inner(
    memory: &mut [u8],
    active_bytes: usize,
    offset: usize,
    data: &[u8],
) -> WasmResult<()> {
    let dst = checked_slice_mut(memory, active_bytes, offset, data.len())?;
    dst.copy_from_slice(data);
    Ok(())
}

// ── Unchecked inner functions ─────────────────────────────────────────
//
// SAFETY: the caller (verified backend) guarantees the offset is in-bounds,
// justified by a formal proof. These use get_unchecked
// (no bounds check) and read_unaligned (no alignment requirement, matching
// Wasm's unaligned memory semantics).

#[inline(never)]
unsafe fn load_i32_unchecked_inner(memory: &[u8], offset: usize) -> i32 {
    let ptr = memory.as_ptr().add(offset) as *const i32;
    i32::from_le(ptr.read_unaligned())
}

#[inline(never)]
unsafe fn load_i64_unchecked_inner(memory: &[u8], offset: usize) -> i64 {
    let ptr = memory.as_ptr().add(offset) as *const i64;
    i64::from_le(ptr.read_unaligned())
}

#[inline(never)]
unsafe fn store_i32_unchecked_inner(memory: &mut [u8], offset: usize, value: i32) {
    let ptr = memory.as_mut_ptr().add(offset) as *mut i32;
    ptr.write_unaligned(value.to_le());
}

#[inline(never)]
unsafe fn store_i64_unchecked_inner(memory: &mut [u8], offset: usize, value: i64) {
    let ptr = memory.as_mut_ptr().add(offset) as *mut i64;
    ptr.write_unaligned(value.to_le());
}

#[cfg(test)]
mod tests {
    use super::*;

    // Use MAX_PAGES=1 for tests — 1 page = 64 KiB, fits on stack in test.
    type Mem = IsolatedMemory<1>;

    #[test]
    fn new_initializes_to_zero() {
        let mem = Mem::try_new(1).unwrap();
        assert_eq!(mem.page_count(), 1);
        assert_eq!(mem.active_size(), PAGE_SIZE);
        assert!(mem.as_slice().iter().all(|&b| b == 0));
    }

    #[test]
    fn try_new_fails_if_initial_exceeds_max() {
        let result = Mem::try_new(2);
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(crate::ConstructionError::MemoryInitialPagesExceedsMax { initial: 2, max: 1 })
        ));
    }

    // ── grow ──

    #[test]
    fn grow_success() {
        let mut mem = IsolatedMemory::<4>::try_new(1).unwrap();
        assert_eq!(mem.grow(2), 1); // old page count
        assert_eq!(mem.page_count(), 3);
    }

    #[test]
    fn grow_to_max() {
        let mut mem = IsolatedMemory::<4>::try_new(1).unwrap();
        assert_eq!(mem.grow(3), 1);
        assert_eq!(mem.page_count(), 4);
    }

    #[test]
    fn grow_beyond_max_fails() {
        let mut mem = IsolatedMemory::<4>::try_new(1).unwrap();
        assert_eq!(mem.grow(4), -1); // would be 5 pages > 4
        assert_eq!(mem.page_count(), 1); // unchanged
    }

    #[test]
    fn grow_zero_is_noop() {
        let mut mem = Mem::try_new(1).unwrap();
        assert_eq!(mem.grow(0), 1);
        assert_eq!(mem.page_count(), 1);
    }

    #[test]
    fn grow_zeroes_new_pages() {
        let mut mem = IsolatedMemory::<2>::try_new(1).unwrap();
        assert_eq!(mem.grow(1), 1);
        // Verify new page is zero via flat view
        let flat = mem.flat();
        let new_start = PAGE_SIZE;
        let new_end = 2 * PAGE_SIZE;
        assert!(flat[new_start..new_end].iter().all(|&b| b == 0));
    }

    #[test]
    fn size_returns_page_count() {
        let mem = IsolatedMemory::<4>::try_new(2).unwrap();
        assert_eq!(mem.size(), 2);
    }

    // ── load/store i32 ──

    #[test]
    fn store_load_i32_roundtrip() {
        let mut mem = Mem::try_new(1).unwrap();
        mem.store_i32(100, 0x12345678).unwrap();
        assert_eq!(mem.load_i32(100), Ok(0x12345678));
    }

    #[test]
    fn load_i32_out_of_bounds() {
        let mem = Mem::try_new(1).unwrap();
        // Last valid offset for i32: PAGE_SIZE - 4
        assert!(mem.load_i32(PAGE_SIZE - 4).is_ok());
        assert_eq!(mem.load_i32(PAGE_SIZE - 3), Err(WasmTrap::OutOfBounds));
        assert_eq!(mem.load_i32(PAGE_SIZE), Err(WasmTrap::OutOfBounds));
    }

    #[test]
    fn store_i32_out_of_bounds() {
        let mut mem = Mem::try_new(1).unwrap();
        assert!(mem.store_i32(PAGE_SIZE - 4, 42).is_ok());
        assert_eq!(mem.store_i32(PAGE_SIZE - 3, 42), Err(WasmTrap::OutOfBounds));
    }

    #[test]
    fn load_i32_offset_overflow() {
        let mem = Mem::try_new(1).unwrap();
        assert_eq!(mem.load_i32(usize::MAX), Err(WasmTrap::OutOfBounds));
    }

    // ── load/store i64 ──

    #[test]
    fn store_load_i64_roundtrip() {
        let mut mem = Mem::try_new(1).unwrap();
        mem.store_i64(200, 0x0102030405060708i64).unwrap();
        assert_eq!(mem.load_i64(200), Ok(0x0102030405060708i64));
    }

    #[test]
    fn load_i64_out_of_bounds() {
        let mem = Mem::try_new(1).unwrap();
        assert!(mem.load_i64(PAGE_SIZE - 8).is_ok());
        assert_eq!(mem.load_i64(PAGE_SIZE - 7), Err(WasmTrap::OutOfBounds));
    }

    // ── load/store u8 ──

    #[test]
    fn store_load_u8_roundtrip() {
        let mut mem = Mem::try_new(1).unwrap();
        mem.store_u8(0, 0xFF).unwrap();
        assert_eq!(mem.load_u8(0), Ok(0xFF));
    }

    #[test]
    fn load_u8_out_of_bounds() {
        let mem = Mem::try_new(1).unwrap();
        assert!(mem.load_u8(PAGE_SIZE - 1).is_ok());
        assert_eq!(mem.load_u8(PAGE_SIZE), Err(WasmTrap::OutOfBounds));
    }

    // ── load/store u16 ──

    #[test]
    fn store_load_u16_roundtrip() {
        let mut mem = Mem::try_new(1).unwrap();
        mem.store_u16(50, 0xBEEF).unwrap();
        assert_eq!(mem.load_u16(50), Ok(0xBEEF));
    }

    // ── load/store f32 ──

    #[test]
    fn store_load_f32_roundtrip() {
        let mut mem = Mem::try_new(1).unwrap();
        mem.store_f32(300, core::f32::consts::PI).unwrap();
        assert_eq!(mem.load_f32(300), Ok(core::f32::consts::PI));
    }

    // ── load/store f64 ──

    #[test]
    fn store_load_f64_roundtrip() {
        let mut mem = Mem::try_new(1).unwrap();
        mem.store_f64(400, core::f64::consts::E).unwrap();
        assert_eq!(mem.load_f64(400), Ok(core::f64::consts::E));
    }

    // ── unchecked variants ──

    #[test]
    fn unchecked_i32_roundtrip() {
        let mut mem = Mem::try_new(1).unwrap();
        unsafe {
            mem.store_i32_unchecked(100, 42);
            assert_eq!(mem.load_i32_unchecked(100), 42);
        }
    }

    #[test]
    fn unchecked_i64_roundtrip() {
        let mut mem = Mem::try_new(1).unwrap();
        unsafe {
            mem.store_i64_unchecked(200, -1i64);
            assert_eq!(mem.load_i64_unchecked(200), -1i64);
        }
    }

    // ── active_pages boundary ──

    #[test]
    fn access_beyond_active_pages_traps() {
        // MAX_PAGES=2 but only 1 page active
        let mem = IsolatedMemory::<2>::try_new(1).unwrap();
        // Within active region: OK
        assert!(mem.load_i32(0).is_ok());
        // Beyond active_pages but within backing array: still OOB
        assert_eq!(mem.load_i32(PAGE_SIZE), Err(WasmTrap::OutOfBounds));
    }

    #[test]
    fn grow_then_access_new_region() {
        let mut mem = IsolatedMemory::<2>::try_new(1).unwrap();
        assert_eq!(mem.load_i32(PAGE_SIZE), Err(WasmTrap::OutOfBounds));
        mem.grow(1);
        // Now page 2 is active — access succeeds
        assert!(mem.load_i32(PAGE_SIZE).is_ok());
        mem.store_i32(PAGE_SIZE, 99).unwrap();
        assert_eq!(mem.load_i32(PAGE_SIZE), Ok(99));
    }

    // ── init_data ──

    #[test]
    fn init_data_writes_bytes() {
        let mut mem = Mem::try_new(1).unwrap();
        mem.init_data(10, &[1u8, 2, 3, 4]).unwrap();
        assert_eq!(mem.load_u8(10).unwrap(), 1);
        assert_eq!(mem.load_u8(11).unwrap(), 2);
        assert_eq!(mem.load_u8(12).unwrap(), 3);
        assert_eq!(mem.load_u8(13).unwrap(), 4);
    }

    #[test]
    fn init_data_empty_slice_is_noop() {
        let mut mem = Mem::try_new(1).unwrap();
        assert!(mem.init_data(0, &[]).is_ok());
    }

    #[test]
    fn init_data_out_of_bounds() {
        let mut mem = Mem::try_new(1).unwrap();
        let data = [0u8; 10];
        assert_eq!(
            mem.init_data(PAGE_SIZE - 5, &data),
            Err(WasmTrap::OutOfBounds)
        );
    }

    #[test]
    fn init_data_at_boundary() {
        let mut mem = Mem::try_new(1).unwrap();
        let data = [42u8; 4];
        assert!(mem.init_data(PAGE_SIZE - 4, &data).is_ok());
        assert_eq!(mem.load_u8(PAGE_SIZE - 1).unwrap(), 42);
    }

    #[test]
    fn init_data_overwrites_existing() {
        let mut mem = Mem::try_new(1).unwrap();
        mem.store_u8(5, 0xFF).unwrap();
        mem.init_data(5, &[0xABu8]).unwrap();
        assert_eq!(mem.load_u8(5).unwrap(), 0xAB);
    }

    // ── little-endian encoding ──

    #[test]
    fn i32_is_little_endian() {
        let mut mem = Mem::try_new(1).unwrap();
        mem.store_i32(0, 0x04030201).unwrap();
        assert_eq!(mem.load_u8(0), Ok(0x01));
        assert_eq!(mem.load_u8(1), Ok(0x02));
        assert_eq!(mem.load_u8(2), Ok(0x03));
        assert_eq!(mem.load_u8(3), Ok(0x04));
    }
}

// ── Kani Formal Verification Proofs ──────────────────────────────────────
//
// These proof harnesses exhaustively verify core invariants of IsolatedMemory
// using Kani's bounded model checker. Run with: cargo kani -p herkos-runtime
//
// The proofs establish that:
// - All load/store operations either succeed or return Err (never panic)
// - grow respects MAX_PAGES and zero-initializes new pages
// - Store/load roundtrips preserve values
// - Offset overflow is handled correctly
// - active_pages never exceeds MAX_PAGES

#[cfg(kani)]
mod proofs {
    use super::*;

    /// Proof: load_i32 never panics, only returns Ok or Err(OutOfBounds).
    /// Verifies bounds checking correctness for all possible offsets.
    #[kani::proof]
    #[kani::unwind(1)]
    fn load_i32_never_panics() {
        let mem = IsolatedMemory::<4>::new(1); // 1 page active = 64 KiB
        let offset: usize = kani::any();

        // Should return Ok or Err(OutOfBounds), never panic
        let result = mem.load_i32(offset);

        // If successful, offset must be in valid range
        if result.is_ok() {
            kani::assert(
                offset.checked_add(4).is_some(),
                "successful load must not overflow",
            );
            kani::assert(
                offset + 4 <= mem.active_size(),
                "successful load must be within active region",
            );
        }
    }

    /// Proof: load_i64 never panics for any offset.
    #[kani::proof]
    #[kani::unwind(1)]
    fn load_i64_never_panics() {
        let mem = IsolatedMemory::<4>::new(2);
        let offset: usize = kani::any();
        let _ = mem.load_i64(offset);
        // Just checking it doesn't panic - Kani verifies this exhaustively
    }

    /// Proof: load_u8 never panics for any offset.
    #[kani::proof]
    #[kani::unwind(1)]
    fn load_u8_never_panics() {
        let mem = IsolatedMemory::<2>::try_new(1).unwrap();
        let offset: usize = kani::any();
        let _ = mem.load_u8(offset);
    }

    /// Proof: load_u16 never panics for any offset.
    #[kani::proof]
    #[kani::unwind(1)]
    fn load_u16_never_panics() {
        let mem = IsolatedMemory::<2>::try_new(1).unwrap();
        let offset: usize = kani::any();
        let _ = mem.load_u16(offset);
    }

    /// Proof: load_f32 never panics for any offset.
    #[kani::proof]
    #[kani::unwind(1)]
    fn load_f32_never_panics() {
        let mem = IsolatedMemory::<2>::try_new(1).unwrap();
        let offset: usize = kani::any();
        let _ = mem.load_f32(offset);
    }

    /// Proof: load_f64 never panics for any offset.
    #[kani::proof]
    #[kani::unwind(1)]
    fn load_f64_never_panics() {
        let mem = IsolatedMemory::<2>::try_new(1).unwrap();
        let offset: usize = kani::any();
        let _ = mem.load_f64(offset);
    }

    /// Proof: store_i32 never panics for any offset and value.
    #[kani::proof]
    #[kani::unwind(1)]
    fn store_i32_never_panics() {
        let mut mem = IsolatedMemory::<4>::new(1);
        let offset: usize = kani::any();
        let value: i32 = kani::any();
        let _ = mem.store_i32(offset, value);
    }

    /// Proof: store_i64 never panics for any offset and value.
    #[kani::proof]
    #[kani::unwind(1)]
    fn store_i64_never_panics() {
        let mut mem = IsolatedMemory::<4>::new(2);
        let offset: usize = kani::any();
        let value: i64 = kani::any();
        let _ = mem.store_i64(offset, value);
    }

    /// Proof: store_u8 never panics for any offset and value.
    #[kani::proof]
    #[kani::unwind(1)]
    fn store_u8_never_panics() {
        let mut mem = IsolatedMemory::<2>::try_new(1).unwrap();
        let offset: usize = kani::any();
        let value: u8 = kani::any();
        let _ = mem.store_u8(offset, value);
    }

    /// Proof: store_u16 never panics for any offset and value.
    #[kani::proof]
    #[kani::unwind(1)]
    fn store_u16_never_panics() {
        let mut mem = IsolatedMemory::<2>::try_new(1).unwrap();
        let offset: usize = kani::any();
        let value: u16 = kani::any();
        let _ = mem.store_u16(offset, value);
    }

    /// Proof: store_f32 never panics for any offset and value.
    #[kani::proof]
    #[kani::unwind(1)]
    fn store_f32_never_panics() {
        let mut mem = IsolatedMemory::<2>::try_new(1).unwrap();
        let offset: usize = kani::any();
        let value: f32 = kani::any();
        let _ = mem.store_f32(offset, value);
    }

    /// Proof: store_f64 never panics for any offset and value.
    #[kani::proof]
    #[kani::unwind(1)]
    fn store_f64_never_panics() {
        let mut mem = IsolatedMemory::<2>::try_new(1).unwrap();
        let offset: usize = kani::any();
        let value: f64 = kani::any();
        let _ = mem.store_f64(offset, value);
    }

    /// Proof: grow respects MAX_PAGES — active_pages never exceeds it.
    #[kani::proof]
    #[kani::unwind(5)]
    fn grow_respects_max_pages() {
        let mut mem = IsolatedMemory::<4>::new(1);
        let delta: u32 = kani::any();

        let old_pages = mem.page_count();
        let result = mem.grow(delta);

        // active_pages must never exceed MAX_PAGES
        kani::assert(
            mem.page_count() <= 4,
            "active_pages must not exceed MAX_PAGES",
        );

        // If grow succeeded, result should be old page count
        if result >= 0 {
            kani::assert(result == old_pages as i32, "grow returns old page count");
            // New page count is old + delta (if it fit)
            let new_expected = old_pages as u64 + delta as u64;
            if new_expected <= 4 {
                kani::assert(
                    mem.page_count() == new_expected as usize,
                    "grow updates active_pages correctly",
                );
            }
        } else {
            // If grow failed, active_pages unchanged
            kani::assert(
                mem.page_count() == old_pages,
                "failed grow leaves active_pages unchanged",
            );
        }
    }

    /// Proof: grow returns -1 (failure) if new size would exceed MAX_PAGES.
    #[kani::proof]
    #[kani::unwind(4)]
    fn grow_fails_beyond_max() {
        let mut mem = IsolatedMemory::<4>::new(2);
        // Try to grow by 3 pages: 2 + 3 = 5 > 4 (MAX_PAGES)
        let result = mem.grow(3);
        kani::assert(result == -1, "grow beyond MAX_PAGES returns -1");
        kani::assert(mem.page_count() == 2, "failed grow leaves pages unchanged");
    }

    /// Proof: store followed by load returns the same value (i32).
    #[kani::proof]
    #[kani::unwind(1)]
    fn store_load_roundtrip_i32() {
        let mut mem = IsolatedMemory::<1>::try_new(1).unwrap();
        let offset: usize = kani::any();
        let value: i32 = kani::any();

        // If store succeeds, load at the same offset must return the same value
        if mem.store_i32(offset, value).is_ok() {
            let loaded = mem.load_i32(offset);
            kani::assert(loaded.is_ok(), "load succeeds after successful store");
            kani::assert(loaded.unwrap() == value, "load returns the stored value");
        }
    }

    /// Proof: store followed by load returns the same value (i64).
    #[kani::proof]
    #[kani::unwind(1)]
    fn store_load_roundtrip_i64() {
        let mut mem = IsolatedMemory::<1>::try_new(1).unwrap();
        let offset: usize = kani::any();
        let value: i64 = kani::any();

        if mem.store_i64(offset, value).is_ok() {
            kani::assert(
                mem.load_i64(offset) == Ok(value),
                "i64 roundtrip preserves value",
            );
        }
    }

    /// Proof: store followed by load returns the same value (u8).
    #[kani::proof]
    #[kani::unwind(1)]
    fn store_load_roundtrip_u8() {
        let mut mem = IsolatedMemory::<1>::try_new(1).unwrap();
        let offset: usize = kani::any();
        let value: u8 = kani::any();

        if mem.store_u8(offset, value).is_ok() {
            kani::assert(
                mem.load_u8(offset) == Ok(value),
                "u8 roundtrip preserves value",
            );
        }
    }

    /// Proof: store followed by load returns the same value (u16).
    #[kani::proof]
    #[kani::unwind(1)]
    fn store_load_roundtrip_u16() {
        let mut mem = IsolatedMemory::<1>::try_new(1).unwrap();
        let offset: usize = kani::any();
        let value: u16 = kani::any();

        if mem.store_u16(offset, value).is_ok() {
            kani::assert(
                mem.load_u16(offset) == Ok(value),
                "u16 roundtrip preserves value",
            );
        }
    }

    /// Proof: grow zero-initializes new pages.
    #[kani::proof]
    #[kani::unwind(2)]
    fn grow_zeroes_new_pages() {
        let mut mem = IsolatedMemory::<2>::try_new(1).unwrap();

        let result = mem.grow(1);

        if result >= 0 {
            // After grow, the new page should be zero
            // Read a value from the newly activated page
            let value = mem.load_i32(PAGE_SIZE);
            if value.is_ok() {
                kani::assert(value.unwrap() == 0, "newly grown page is zero-initialized");
            }
        }
    }

    /// Proof: offset overflow is handled safely (no panic, returns OutOfBounds).
    #[kani::proof]
    #[kani::unwind(1)]
    fn offset_overflow_handled() {
        let mem = IsolatedMemory::<1>::try_new(1).unwrap();
        // Try to load at maximum possible offset (will overflow when adding size)
        let result = mem.load_i32(usize::MAX);
        kani::assert(
            result == Err(WasmTrap::OutOfBounds),
            "overflow offset returns OutOfBounds",
        );
    }

    /// Proof: accesses beyond active_pages (but within MAX_PAGES) are rejected.
    #[kani::proof]
    #[kani::unwind(1)]
    fn access_beyond_active_pages_rejected() {
        // MAX_PAGES=2 but only 1 active
        let mem = IsolatedMemory::<2>::try_new(1).unwrap();

        // Access in first page: should succeed
        let result1 = mem.load_i32(0);
        kani::assert(result1.is_ok(), "access within active pages succeeds");

        // Access in second page (not active yet): should fail
        let result2 = mem.load_i32(PAGE_SIZE);
        kani::assert(
            result2 == Err(WasmTrap::OutOfBounds),
            "access beyond active_pages is rejected",
        );
    }

    /// Proof: active_size always equals active_pages * PAGE_SIZE.
    #[kani::proof]
    #[kani::unwind(1)]
    fn active_size_invariant() {
        let mem = IsolatedMemory::<4>::new(2);
        kani::assert(
            mem.active_size() == mem.page_count() * PAGE_SIZE,
            "active_size = active_pages * PAGE_SIZE",
        );
    }

    /// Proof: size() returns active_pages as i32.
    #[kani::proof]
    #[kani::unwind(1)]
    fn size_returns_page_count() {
        let mem = IsolatedMemory::<4>::new(3);
        kani::assert(
            mem.size() == mem.page_count() as i32,
            "size() returns active_pages",
        );
    }

    /// Proof: successful load requires offset + type_size <= active_size.
    #[kani::proof]
    #[kani::unwind(1)]
    fn load_success_implies_valid_range() {
        let mem = IsolatedMemory::<1>::try_new(1).unwrap();
        let offset: usize = kani::any();

        let result = mem.load_i32(offset);

        if result.is_ok() {
            // Success implies: offset + 4 <= active_size and no overflow
            let end = offset.checked_add(4);
            kani::assert(end.is_some(), "successful load offset does not overflow");
            kani::assert(
                end.unwrap() <= mem.active_size(),
                "successful load is within bounds",
            );
        }
    }

    /// Proof: successful store requires offset + type_size <= active_size.
    #[kani::proof]
    #[kani::unwind(1)]
    fn store_success_implies_valid_range() {
        let mut mem = IsolatedMemory::<1>::try_new(1).unwrap();
        let offset: usize = kani::any();
        let value: i64 = kani::any();

        let result = mem.store_i64(offset, value);

        if result.is_ok() {
            let end = offset.checked_add(8);
            kani::assert(end.is_some(), "successful store offset does not overflow");
            kani::assert(
                end.unwrap() <= mem.active_size(),
                "successful store is within bounds",
            );
        }
    }

    /// Proof: as_slice returns a slice of exactly active_size bytes.
    #[kani::proof]
    #[kani::unwind(1)]
    fn as_slice_length_correct() {
        let mem = IsolatedMemory::<4>::new(2);
        let slice = mem.as_slice();
        kani::assert(
            slice.len() == mem.active_size(),
            "as_slice length equals active_size",
        );
    }

    /// Proof: as_mut_slice returns a slice of exactly active_size bytes.
    #[kani::proof]
    #[kani::unwind(1)]
    fn as_mut_slice_length_correct() {
        let mut mem = IsolatedMemory::<4>::new(2);
        let slice = mem.as_mut_slice();
        kani::assert(
            slice.len() == mem.active_size(),
            "as_mut_slice length equals active_size",
        );
    }
}
