//! Core tile allocator implementation.
//!
//! Design summary (see `mod.rs` for rationale):
//! - One **free list** per size class, intrusive: each free tile's
//!   first `size_of::<*mut ()>()` bytes hold the `next` pointer.
//! - Per class we also keep a **bump range** `[bump_ptr, bump_end)`
//!   pointing into the current extent. Tiles that have never been
//!   handed out yet are carved off the front of this range on demand;
//!   only tiles that have been freed re-enter the free list. This
//!   avoids the O(`tiles_per_extent`) write loop that a pre-populated
//!   free list would pay at every `add_extent`.
//! - **alloc** =
//!     1. If the free list is non-empty, pop the head.
//!     2. Else if `bump_ptr < bump_end`, carve one tile off the front
//!        of the bump range and advance `bump_ptr` by `tile_size`.
//!     3. Else call `add_extent` and retry from (2).
//! - **dealloc** = push back onto the free list head. Always O(1).
//! - **add_extent** = ask the system allocator for one extent and set
//!   the class's bump range to span it. No per-tile setup write. Each
//!   extent is tracked so we can report `extent_count` for stats and so
//!   `trim_empty_extents` can hand fully-free extents back.
//! - **trim** (on demand only) = sweep the free lists and return every
//!   fully-empty extent to the system allocator, undoing a transient
//!   spike that stranded extents in the otherwise grow-only pool. Never
//!   runs on the alloc/dealloc hot path. See `trim_empty_extents`.
//!
//! Concurrency: the allocator is `Sync` via interior mutability
//! (`UnsafeCell`). On `wasm32-unknown-unknown` execution is
//! single-threaded so concurrent access is impossible in practice; if
//! Ruffle ever moves to wasm-threads this needs a `Mutex` wrapper.

use core::alloc::Layout;
use core::cell::UnsafeCell;
use core::ptr;
use std::alloc::{GlobalAlloc, System};

use super::config::{ENABLE_HISTOGRAMS, TILE_CONFIG, TileClass};
use super::stats::{TileAllocatorStats, TileClassStats, TileFallbackStats};

/// Maximum supported size class count. Sized to comfortably fit the
/// `TILE_CONFIG` array while staying a compile-time constant so the
/// per-class arrays can live inline in the allocator struct.
const MAX_CLASSES: usize = 128;

/// Header at the start of every free tile. The first word is reused
/// to chain tiles into the free list, so the tile size must be at
/// least `size_of::<FreeTile>()` (= 4 bytes on wasm32).
#[repr(C)]
struct FreeTile {
    next: *mut FreeTile,
}

/// Metadata for one extent (a contiguous region obtained from the
/// system allocator and split into `tiles_per_extent` tiles). Kept in
/// a `Vec` for stats reporting and as the anchor `trim_empty_extents`
/// uses to hand fully-free extents back to the system allocator.
struct ExtentRecord {
    ptr: *mut u8,
    total_bytes: usize,
    /// Transient scratch, meaningful **only** during a single
    /// `trim_empty_extents` sweep: how many of this extent's tiles are
    /// currently on the free list, and the head of those same tiles
    /// re-chained into a per-extent bucket. Reset at the start of every
    /// sweep; never read on the alloc/dealloc hot path.
    free_count: usize,
    bucket: *mut FreeTile,
}

/// Per–size-class runtime state.
struct ClassState {
    /// Tile size in bytes. Mirrors `TileClass::size_bytes`.
    tile_size: usize,
    /// Effective alignment guaranteed for every tile of this class.
    /// Auto-derived as the greatest power-of-two divisor of
    /// `tile_size` (= lowest set bit), so for pow-2 sizes it equals
    /// `tile_size`, and for sizes like 24 / 48 / 96 it lands on
    /// 8 / 16 / 32 respectively. Used by `pick_class` to gate
    /// over-aligned requests and by `add_extent` to align the extent.
    tile_align: usize,
    /// Tiles carved per extent. Derived from `TileClass::extent_mb`
    /// as `floor((extent_mb * 1 MiB) / size_bytes)`.
    tiles_per_extent: usize,
    /// Head of the intrusive free list. `null` when empty. Populated
    /// exclusively by `class_dealloc`: fresh tiles from a newly-added
    /// extent are handed out via `bump_ptr` (see below), not chained
    /// onto the free list.
    free_head: *mut FreeTile,
    /// Lazy carve cursor into the current extent. `class_alloc` hands
    /// out tiles by stepping this pointer forward in `tile_size`-sized
    /// chunks, avoiding the O(`tiles_per_extent`) write loop that an
    /// eager free-list build would pay at every `add_extent`. Both
    /// `bump_ptr` and `bump_end` are `null` until the first
    /// `add_extent` call.
    bump_ptr: *mut u8,
    /// One past the last byte of the current extent. The invariant
    /// `(bump_end - extent_start) % tile_size == 0` is preserved by
    /// `add_extent`, so the check `bump_ptr < bump_end` is sufficient
    /// to guarantee at least one full tile fits before the carve.
    bump_end: *mut u8,
    /// Extent tracking (for stats and `trim_empty_extents` reclamation).
    extents: Vec<ExtentRecord>,
    /// Cumulative counters.
    alloc_total: usize,
    dealloc_total: usize,
    /// Cumulative count of extents added to this class since boot.
    /// `extents.len()` equals `extents_added - extents_released`; before
    /// any `trim_empty_extents` reclaims an extent the latter is 0 and it
    /// equals `extents_added`.
    extents_added: usize,
    /// Cumulative count of extents handed back to the system allocator by
    /// `trim_empty_extents` since boot. The current extent count is
    /// `extents_added - extents_released`.
    extents_released: usize,
    /// **Live** byte count of internal fragmentation currently held by
    /// this class: sum of `(tile_size - layout.size())` over every tile
    /// currently alive in the pool. Incremented in `class_alloc`,
    /// decremented in `class_dealloc` using the `layout.size()` that
    /// Rust's `GlobalAlloc` contract guarantees is passed back at
    /// dealloc — so the value reflects the present state of the pool
    /// rather than a lifetime total. Pair with `requested_bytes_total`
    /// for the live waste-as-percent.
    wasted_bytes_total: u64,
    /// **Live** byte count actually requested by callers and currently
    /// held by tiles in this class: sum of `layout.size()` over the
    /// alive tiles. Same alloc/dealloc symmetry as `wasted_bytes_total`.
    requested_bytes_total: u64,
    /// Peak `alive_tiles` ever observed for this class. Updated in
    /// `class_alloc`. Tells you how much capacity the class actually
    /// needed under load — useful to size `extent_mb` for the next
    /// run (if `peak ≤ tiles_per_extent`, one extent suffices).
    peak_alive_tiles: usize,
    /// Minimum request size (in bytes) routed to this class
    /// (inclusive). Equals the previous class's `size_bytes + 1` (from
    /// `TileClass`), or `1` for the first class. The histogram covers
    /// `size_min..=tile_size`. Exposed as `TileClassStats::size_min`.
    size_min: usize,
    /// **Live** histogram: one counter per `layout.size()` value in
    /// `size_min..=tile_size`. Index `i` corresponds to
    /// `requested_size = size_min + i`. Length = `tile_size -
    /// size_min + 1`. Incremented in `class_alloc`, decremented in
    /// `class_dealloc` — so each bucket counts the tiles of that
    /// requested size **currently alive** in the pool, not all the
    /// allocs that ever happened. Snapshot diffs over time reveal
    /// growth / shrinkage by requested size.
    size_histogram: Vec<u64>,
    /// Cumulative in-place realloc fast-path hits served by this class:
    /// the times `realloc` was called and both the old and new
    /// `layout.size()` landed here, so we returned the same pointer
    /// without copy. Pairs with `cross_class_realloc_total` (global) to
    /// gauge how often the fast path actually kicks in.
    inplace_realloc_total: u64,
    /// Cumulative cross-class realloc **arrivals from a smaller
    /// origin**: realloc calls where `new_size > old_size` and
    /// `new_layout` landed in this class. The classic "Vec::push
    /// triggered a grow that overflowed into this larger class"
    /// pattern.
    ///
    /// **Edge asymmetry**: for the **smallest** class this counter
    /// is always 0 (no smaller class exists from which a grow can
    /// arrive).
    inbound_realloc_total_grow: u64,
    /// Cumulative cross-class realloc **arrivals from a larger
    /// origin**: realloc calls where `new_size < old_size` and
    /// `new_layout` landed in this class. The buffer shrank down
    /// into us from a larger pool class or from the fallback. Mirror
    /// of `inbound_realloc_total_grow`.
    inbound_realloc_total_shrink: u64,
    /// Cumulative cross-class realloc **departures toward a larger
    /// destination**: realloc calls where `new_size > old_size` and
    /// the old buffer lived in this class. The "this class was
    /// outgrown" counter.
    ///
    /// **Edge asymmetry**: for the **largest** pool class, departures
    /// of this kind always land in the fallback (no larger pool
    /// class to receive them).
    outbound_realloc_total_grow: u64,
    /// Cumulative cross-class realloc **departures toward a smaller
    /// destination**: realloc calls where `new_size < old_size` and
    /// the old buffer lived in this class. The "this class lost a
    /// buffer to a smaller one" counter.
    ///
    /// **Edge asymmetry**: for the **smallest** class this counter
    /// is always 0 (no smaller class to shrink into).
    outbound_realloc_total_shrink: u64,
    /// **Live** histogram of `layout.align()` for tiles currently alive
    /// in this class. Buckets are `1, 2, 4, 8, 16, 32, 64, 128+` (top
    /// saturates). Same alloc/dealloc symmetry as `size_histogram`. The
    /// sum of the buckets equals `alive_tiles`. Together with
    /// `size_histogram` it lets you separate "alloc with size N that
    /// could fit in a smaller class if it weren't for align" from
    /// "alloc with size N that only fits here on size alone".
    align_histogram: [u64; ALIGN_BUCKET_COUNT],
}

impl ClassState {
    /// `prev_size` is the previous class's `size_bytes`, or 0 for the
    /// first class. Used to set `size_min = prev_size + 1` and size
    /// the per-class request histogram.
    fn new(cls: &TileClass, prev_size: usize) -> Self {
        // Greatest power-of-two divisor of size_bytes (= lowest set bit).
        // For pow-2 sizes returns size_bytes itself; for 24 → 8, 48 → 16,
        // 96 → 32, etc.
        let tile_align = cls.size_bytes & cls.size_bytes.wrapping_neg();
        let tiles_per_extent = (cls.extent_mb * 1024 * 1024) / cls.size_bytes;
        let size_min = prev_size + 1;
        // Length = tile_size - size_min + 1 = tile_size - prev_size.
        let bucket_count = cls.size_bytes - prev_size;
        Self {
            tile_size: cls.size_bytes,
            tile_align,
            tiles_per_extent,
            free_head: ptr::null_mut(),
            bump_ptr: ptr::null_mut(),
            bump_end: ptr::null_mut(),
            extents: Vec::new(),
            alloc_total: 0,
            dealloc_total: 0,
            extents_added: 0,
            extents_released: 0,
            wasted_bytes_total: 0,
            requested_bytes_total: 0,
            peak_alive_tiles: 0,
            size_min,
            // When `ENABLE_HISTOGRAMS == false` we keep the field for
            // API stability but never allocate the backing storage —
            // `Vec::new()` has `cap == 0`, no heap touch — and the
            // hot-path updates in `class_alloc` / `class_dealloc` are
            // dead-code-eliminated by LLVM. JSON serialization in
            // `stats.rs` likewise omits the field entirely.
            size_histogram: if ENABLE_HISTOGRAMS {
                vec![0u64; bucket_count]
            } else {
                Vec::new()
            },
            inplace_realloc_total: 0,
            inbound_realloc_total_grow: 0,
            inbound_realloc_total_shrink: 0,
            outbound_realloc_total_grow: 0,
            outbound_realloc_total_shrink: 0,
            align_histogram: [0u64; ALIGN_BUCKET_COUNT],
        }
    }

    /// Snapshot this class's counters into a stats struct.
    fn snapshot(&self) -> TileClassStats {
        let alive = self.alloc_total.saturating_sub(self.dealloc_total);
        let capacity = self.extents.len() * self.tiles_per_extent;
        let free = capacity.saturating_sub(alive);
        TileClassStats {
            size_min: self.size_min,
            size_max: self.tile_size,
            free_tiles: free,
            alive_tiles: alive,
            extent_bytes: self.extents.iter().map(|s| s.total_bytes).sum(),
            extent_count: self.extents.len(),
            extents_added: self.extents_added,
            extents_released: self.extents_released,
            alloc_total: self.alloc_total,
            dealloc_total: self.dealloc_total,
            peak_alive_tiles: self.peak_alive_tiles,
            wasted_bytes_total: self.wasted_bytes_total,
            requested_bytes_total: self.requested_bytes_total,
            inplace_realloc_total: self.inplace_realloc_total,
            inbound_realloc_total_grow: self.inbound_realloc_total_grow,
            inbound_realloc_total_shrink: self.inbound_realloc_total_shrink,
            outbound_realloc_total_grow: self.outbound_realloc_total_grow,
            outbound_realloc_total_shrink: self.outbound_realloc_total_shrink,
            size_histogram: self.size_histogram.clone(),
            align_histogram: self.align_histogram,
        }
    }
}

/// Number of buckets in the fallback size histogram. See
/// `fallback_bucket_for_size` for the boundary mapping.
const FALLBACK_BUCKET_COUNT: usize = 10;

/// Number of buckets in the per-class request-align histogram.
/// Buckets correspond to align values `1, 2, 4, 8, 16, 32, 64, 128+`
/// (top bucket saturates for any `align ≥ 128`). Sized to cover the
/// realistic spectrum in Rust on wasm32: cache-line `#[repr(align(64))]`
/// is conceivable, and even `#[repr(align(128))]` is occasionally used
/// for anti-false-sharing. Above 128 the distinction stops being useful
/// for tuning — those allocs all land in the largest pow-2 size classes.
pub(crate) const ALIGN_BUCKET_COUNT: usize = 8;

/// Map a power-of-two `align` to its histogram bucket index.
///
/// Buckets:
/// - 0 → `align == 1`
/// - 1 → `align == 2`
/// - 2 → `align == 4`
/// - 3 → `align == 8`
/// - 4 → `align == 16`
/// - 5 → `align == 32`
/// - 6 → `align == 64`
/// - 7 → `align ≥ 128` (saturating; very rare in Rust user code)
///
/// Rust guarantees `align` is a power of two via the `Layout`
/// invariants, so `trailing_zeros()` gives the exact log2.
fn align_bucket_for(align: usize) -> usize {
    let bucket = align.trailing_zeros() as usize;
    if bucket >= ALIGN_BUCKET_COUNT {
        ALIGN_BUCKET_COUNT - 1
    } else {
        bucket
    }
}

/// Map a fallback allocation size to its histogram bucket index.
///
/// Bucket boundaries (inclusive upper bound):
/// - 0: ≤ 512
/// - 1: ≤ 1 024
/// - 2: ≤ 2 048
/// - 3: ≤ 4 096
/// - 4: ≤ 8 192
/// - 5: ≤ 16 384
/// - 6: ≤ 32 768
/// - 7: ≤ 65 536
/// - 8: ≤ 131 072
/// - 9: >  131 072
///
/// Designed to spotlight clusters of medium-large allocations that
/// currently bypass the tile pool — candidates for new size classes.
fn fallback_bucket_for_size(size: usize) -> usize {
    if size <= 512 {
        0
    } else if size <= 1024 {
        1
    } else if size <= 2048 {
        2
    } else if size <= 4096 {
        3
    } else if size <= 8192 {
        4
    } else if size <= 16384 {
        5
    } else if size <= 32768 {
        6
    } else if size <= 65536 {
        7
    } else if size <= 131072 {
        8
    } else {
        9
    }
}

/// Inner state of the tile allocator. All mutation goes through here,
/// reached from the outer wrapper via `UnsafeCell`.
struct TileInner {
    classes: [Option<ClassState>; MAX_CLASSES],
    class_count: usize,

    /// Counters for allocations that bypassed the tile pools (too big
    /// or over-aligned). `bytes_alive` is approximate — we track size
    /// only at request time, so an inflight `realloc` may briefly
    /// double-count.
    ///
    /// `fallback_bytes_alloc` and `fallback_bytes_dealloc` are `u64`
    /// rather than `usize` because they are **cumulative-over-lifetime**:
    /// every fallback alloc and dealloc adds its `layout.size()` to
    /// them, forever. On `wasm32-unknown-unknown`, `usize` is 32 bits
    /// and would wrap past 4 GiB of accumulated byte traffic — under a
    /// sustained fallback churn that's only minutes of runtime. When
    /// one of the two wraps before the other, `bytes_alive =
    /// saturating_sub(alloc, dealloc)` collapses to 0 (or shoots to a
    /// bogus high value) and snaps back when the second wraps too,
    /// producing periodic ~minute-scale oscillations in the snapshot.
    /// `u64` gives ~18 EiB of headroom — for all practical purposes,
    /// the cumulative bytes counters cannot wrap.
    fallback_alloc_total: usize,
    fallback_dealloc_total: usize,
    fallback_bytes_alloc: u64,
    fallback_bytes_dealloc: u64,

    /// **Live** bucketed histogram of fallback allocations currently
    /// outstanding (size-keyed via `fallback_bucket_for_size`).
    /// Incremented in the fallback alloc path, decremented in the
    /// fallback dealloc path. Helps decide whether to add larger size
    /// classes above the current ceiling: a sustained large count in a
    /// specific bucket signals a hot fallback path worth absorbing.
    fallback_size_histogram: [u64; FALLBACK_BUCKET_COUNT],

    /// Cumulative count of realloc calls that could not be served by
    /// the in-place fast path and required alloc+copy+dealloc. Includes
    /// tile-to-tile cross-class and any transition involving fallback.
    /// Compare against the sum of per-class `inplace_realloc_total` to
    /// gauge fast-path effectiveness on the current workload.
    cross_class_realloc_total: u64,
}

impl TileInner {
    const fn empty() -> Self {
        const NONE: Option<ClassState> = None;
        Self {
            classes: [NONE; MAX_CLASSES],
            class_count: 0,
            fallback_alloc_total: 0,
            fallback_dealloc_total: 0,
            fallback_bytes_alloc: 0,
            fallback_bytes_dealloc: 0,
            fallback_size_histogram: [0u64; FALLBACK_BUCKET_COUNT],
            cross_class_realloc_total: 0,
        }
    }

    /// One-time init from `TILE_CONFIG`. Called lazily on first
    /// allocation so we don't need a `const fn` for `Vec::new`.
    ///
    /// **Re-entrancy**: `ClassState::new` calls `vec![0u64; N]` to
    /// allocate its per-class size histogram. That allocation routes
    /// back through the global allocator — i.e. back through us. If
    /// we leave `class_count == 0` while building the classes, the
    /// re-entrant alloc loops back into this function and stack-
    /// overflows. We therefore set `class_count = TILE_CONFIG.len()`
    /// **before** touching any class slot. Re-entrant allocs landing
    /// in here during the bootstrap loop see "initialized", and the
    /// `pick_class` path finds the requested class's `classes[i] ==
    /// None` for slots not yet populated — falling cleanly through to
    /// the system allocator. Those bootstrap allocs (the histogram
    /// buffers themselves) are therefore served by dlmalloc, after
    /// which every subsequent allocation routes through the tile
    /// pools as normal.
    fn ensure_initialized(&mut self) {
        if self.class_count != 0 {
            return;
        }
        assert!(
            TILE_CONFIG.len() <= MAX_CLASSES,
            "TILE_CONFIG has more entries than MAX_CLASSES",
        );
        // Validate ordering, minimum size, and extent sizing at init.
        // Note: size_bytes is NOT required to be a power of two; the
        // effective tile alignment is auto-derived (see ClassState::new).
        let min_tile_size = core::mem::size_of::<FreeTile>();
        let mut last_size = 0usize;
        for cls in TILE_CONFIG.iter() {
            assert!(
                cls.size_bytes > last_size,
                "TILE_CONFIG must be in strictly ascending size order",
            );
            assert!(
                cls.size_bytes >= min_tile_size,
                "TILE_CONFIG size_bytes must be at least size_of::<*mut ()>",
            );
            assert!(
                cls.extent_mb >= 1,
                "TILE_CONFIG extent_mb must be at least 1",
            );
            let tiles = (cls.extent_mb * 1024 * 1024) / cls.size_bytes;
            assert!(
                tiles >= 1,
                "TILE_CONFIG extent_mb / size_bytes must yield at least 1 tile",
            );
            last_size = cls.size_bytes;
        }
        // Mark as initialized BEFORE building any class — see the
        // re-entrancy note in the doc comment above.
        self.class_count = TILE_CONFIG.len();
        let mut prev_size = 0usize;
        for (i, cls) in TILE_CONFIG.iter().enumerate() {
            self.classes[i] = Some(ClassState::new(cls, prev_size));
            prev_size = cls.size_bytes;
        }
    }

    /// Pick the smallest class that satisfies both `size_bytes >= size`
    /// and `tile_align >= align`. Returns `None` if no class fits —
    /// caller falls back to system allocator.
    ///
    /// With non-pow-2 size classes, `tile_align` can be smaller than
    /// `size_bytes` (e.g. class 24 has tile_align=8). A request with
    /// `align == 16` therefore skips class 24 and lands in class 32 or
    /// 48 instead, even though class 24 would have been big enough on
    /// size alone.
    fn pick_class(&self, layout: Layout) -> Option<usize> {
        let required_size = layout.size();
        let required_align = layout.align();
        for i in 0..self.class_count {
            if let Some(c) = &self.classes[i]
                && c.tile_size >= required_size
                && c.tile_align >= required_align
            {
                return Some(i);
            }
        }
        None
    }

    /// Add a new extent to a class. Allocates a fresh region from the
    /// system allocator and installs it as the class's new bump range.
    /// The previous bump range, if any, was already drained (the only
    /// caller is `class_alloc`, which calls this only when
    /// `bump_ptr >= bump_end`). The free list is left untouched: tiles
    /// enter it exclusively through `class_dealloc`. Returns `false`
    /// on OOM.
    ///
    /// The bump-pointer carve performs no per-tile setup write at
    /// extent creation. The alternative — chaining every tile into
    /// the free list eagerly — would cost `tiles_per_extent` stores
    /// per call (potentially hundreds of thousands for small-tile
    /// classes), all touching fresh wasm linear-memory pages, and
    /// would surface as a visible stall at the first alloc per class.
    /// Carving lazily amortizes that per-tile touch across the
    /// subsequent N allocs.
    ///
    /// Safety: `class_idx` must be a valid initialized class.
    unsafe fn add_extent(&mut self, class_idx: usize) -> bool {
        let class = self.classes[class_idx]
            .as_mut()
            .expect("class must be initialized");
        let tile_size = class.tile_size;
        let tile_align = class.tile_align;
        let tiles_per_extent = class.tiles_per_extent;
        let extent_total = tile_size * tiles_per_extent;

        // Extent is aligned to tile_align (the greatest power-of-2
        // divisor of tile_size). Since every tile sits at an offset
        // that is a multiple of tile_size, and tile_size is itself
        // a multiple of tile_align, every tile ends up naturally
        // aligned to tile_align as well.
        let layout = match Layout::from_size_align(extent_total, tile_align) {
            Ok(l) => l,
            Err(_) => return false,
        };
        let extent_ptr = unsafe { System.alloc(layout) };
        if extent_ptr.is_null() {
            return false;
        }

        // Install the bump range. By construction
        // `extent_total = tile_size * tiles_per_extent` is an exact
        // multiple of `tile_size`, so `bump_ptr < bump_end` is the only
        // check `class_alloc` needs to confirm at least one full tile
        // fits before the carve.
        class.bump_ptr = extent_ptr;
        class.bump_end = unsafe { extent_ptr.add(extent_total) };
        class.extents.push(ExtentRecord {
            ptr: extent_ptr,
            total_bytes: extent_total,
            free_count: 0,
            bucket: ptr::null_mut(),
        });
        class.extents_added += 1;
        true
    }

    /// Tile-pool alloc fast path. Returns `null` if the class is OOM
    /// or `pick_class` rejected the request — caller falls back.
    ///
    /// `layout` is the original request from the caller; we use
    /// `layout.size()` to track internal fragmentation (waste) per
    /// class and bucket the per-class request-size histogram, and
    /// `layout.align()` to bucket the per-class align histogram.
    /// `layout.size() <= tile_size` and `layout.align() <= tile_align`
    /// are guaranteed by `pick_class`.
    ///
    /// Tile-resolution policy (see module doc):
    /// 1. Pop the free list if non-empty (recycles a recently-freed
    ///    tile — likely still hot in cache).
    /// 2. Else carve one tile off the front of `[bump_ptr, bump_end)`
    ///    (the unused tail of the current extent).
    /// 3. Else call `add_extent` to install a new bump range and retry.
    unsafe fn class_alloc(&mut self, class_idx: usize, layout: Layout) -> *mut u8 {
        // Resolve a tile. The loop runs at most twice: at most one
        // bump-range exhaustion can happen before `add_extent`
        // installs a fresh, non-empty range.
        let tile: *mut u8 = loop {
            let class = self.classes[class_idx]
                .as_mut()
                .expect("class must be initialized");
            // Fast path 1: free-list pop (recycled tile).
            if !class.free_head.is_null() {
                let head = class.free_head;
                // SAFETY: head is non-null and was previously a valid
                // tile address returned from this same class.
                class.free_head = unsafe { (*head).next };
                break head as *mut u8;
            }
            // Fast path 2: bump-pointer carve from the current extent.
            // By construction the bump range spans an integer number
            // of tiles, so `bump_ptr < bump_end` is sufficient to know
            // a full tile fits.
            if class.bump_ptr < class.bump_end {
                let carved = class.bump_ptr;
                // SAFETY: `add` stays within the extent — see invariant
                // documented on `bump_end`.
                class.bump_ptr = unsafe { class.bump_ptr.add(class.tile_size) };
                break carved;
            }
            // Slow path: grow. `add_extent` either installs a fresh
            // bump range (looping back into the carve branch above) or
            // reports OOM, in which case the caller falls back.
            if !unsafe { self.add_extent(class_idx) } {
                return ptr::null_mut();
            }
        };

        // Common accounting.
        let class = self.classes[class_idx]
            .as_mut()
            .expect("class must be initialized");
        class.alloc_total += 1;
        let requested_size = layout.size();
        // Internal-fragmentation tracking. `tile_size >= requested_size`
        // is guaranteed by `pick_class`; the difference is the byte
        // count rounded up by routing into this class.
        let waste = class.tile_size.saturating_sub(requested_size) as u64;
        class.wasted_bytes_total = class.wasted_bytes_total.saturating_add(waste);
        class.requested_bytes_total = class
            .requested_bytes_total
            .saturating_add(requested_size as u64);
        // High-water mark of alive tiles for capacity sizing diagnostics.
        let alive_now = class.alloc_total.saturating_sub(class.dealloc_total);
        if alive_now > class.peak_alive_tiles {
            class.peak_alive_tiles = alive_now;
        }
        // Per-class request-size and align histograms. Gated by
        // `ENABLE_HISTOGRAMS` so the whole block — the bucket compute,
        // the heap-indirect `size_histogram` update, and the inline
        // `align_histogram` update — is dead-code-eliminated when the
        // const is `false`. The heap-indirect Vec update is the
        // dominant hot-path cost of the histogram instrumentation;
        // gating it keeps `class_alloc` lean in production builds.
        if ENABLE_HISTOGRAMS {
            // When an over-aligned request skips a smaller class on
            // `tile_align` grounds, `requested_size` can be <
            // `size_min`; we clamp to bucket 0 (= `size_min`) —
            // slightly imprecise but bounded.
            let bucket_idx = requested_size.saturating_sub(class.size_min);
            if bucket_idx < class.size_histogram.len() {
                class.size_histogram[bucket_idx] =
                    class.size_histogram[bucket_idx].saturating_add(1);
            }
            let align_bucket = align_bucket_for(layout.align());
            class.align_histogram[align_bucket] =
                class.align_histogram[align_bucket].saturating_add(1);
        }
        tile
    }

    /// Push a tile back onto its class's free list. Safety: `ptr` must
    /// have come from `class_alloc(class_idx, layout)` with the same
    /// `layout` and not yet been freed.
    ///
    /// The Rust `GlobalAlloc` contract guarantees the layout passed to
    /// `dealloc` matches the one used at `alloc`, so we recover the
    /// original size and align for free. We use both to decrement the
    /// per-class waste / requested / size-histogram / align-histogram
    /// counters so they reflect the live state of the pool (not
    /// lifetime cumulative).
    unsafe fn class_dealloc(&mut self, class_idx: usize, ptr: *mut u8, layout: Layout) {
        let class = self.classes[class_idx]
            .as_mut()
            .expect("class must be initialized");
        let tile = ptr as *mut FreeTile;
        unsafe { (*tile).next = class.free_head };
        class.free_head = tile;
        class.dealloc_total += 1;
        let requested_size = layout.size();
        // Mirror the alloc-side accounting so the counters track the
        // currently-live state of the pool.
        let waste = class.tile_size.saturating_sub(requested_size) as u64;
        class.wasted_bytes_total = class.wasted_bytes_total.saturating_sub(waste);
        class.requested_bytes_total = class
            .requested_bytes_total
            .saturating_sub(requested_size as u64);
        // Histograms — gated symmetrically with `class_alloc`; see the
        // doc-comment there.
        if ENABLE_HISTOGRAMS {
            let bucket_idx = requested_size.saturating_sub(class.size_min);
            if bucket_idx < class.size_histogram.len() {
                class.size_histogram[bucket_idx] =
                    class.size_histogram[bucket_idx].saturating_sub(1);
            }
            let align_bucket = align_bucket_for(layout.align());
            class.align_histogram[align_bucket] =
                class.align_histogram[align_bucket].saturating_sub(1);
        }
    }

    /// Release fully-empty extents back to the system allocator, returning the
    /// total bytes reclaimed.
    ///
    /// The pool is otherwise grow-only: a transient spike — e.g. millions of
    /// short-lived objects that all land in one size class faster than the GC
    /// can reclaim them — permanently inflates `extents`. On WASM, where linear
    /// memory never shrinks, that memory then stays reserved by the pool and
    /// starves later allocations (including the fallback path other code relies
    /// on), even though it is entirely free. This hands the empty extents back
    /// to dlmalloc, where any size class *or* the fallback can reuse them.
    ///
    /// An extent is releasable when *every* one of its tiles is on the free
    /// list (all were carved and then freed). The current bump extent is never
    /// released; its un-carved tail keeps its `free_count` below
    /// `tiles_per_extent` anyway, but the `is_current` guard makes that
    /// explicit (and covers the fully-carved-then-drained edge).
    ///
    /// Cost: O(free tiles · log extents), paid only when called — typically on
    /// memory pressure or after a GC — never on the alloc/dealloc hot path.
    ///
    /// **Allocation-free** by construction: this runs *as* the global
    /// allocator, so it must not route back through `self`. It uses only
    /// in-place scratch — an in-place `sort_unstable` (which never allocates)
    /// and the per-extent `free_count`/`bucket` fields on the existing Vec.
    ///
    /// Safety / soundness: single-threaded on wasm32 (see the `Sync` impl).
    unsafe fn trim_empty_extents(&mut self) -> usize {
        let mut released_total = 0usize;
        for ci in 0..self.class_count {
            let Some(class) = self.classes[ci].as_mut() else {
                continue;
            };
            if class.extents.is_empty() || class.free_head.is_null() {
                continue;
            }

            // Sort extents by base address so a free tile's owning extent is a
            // binary search. `sort_unstable` is in place and never allocates.
            class.extents.sort_unstable_by_key(|e| e.ptr as usize);
            for e in class.extents.iter_mut() {
                e.free_count = 0;
                e.bucket = ptr::null_mut();
            }

            // Pass 1: drain the free list, re-chaining each tile into its
            // owning extent's bucket and counting per extent. Each bucket stays
            // entirely within one extent, so a released extent's chain never
            // dangles into a retained one.
            let mut orphans: *mut FreeTile = ptr::null_mut();
            let mut tile = class.free_head;
            while !tile.is_null() {
                let next = unsafe { (*tile).next };
                let addr = tile as usize;
                let pp = class.extents.partition_point(|e| (e.ptr as usize) <= addr);
                if pp > 0 && addr < class.extents[pp - 1].ptr as usize + class.extents[pp - 1].total_bytes {
                    let e = &mut class.extents[pp - 1];
                    unsafe { (*tile).next = e.bucket };
                    e.bucket = tile;
                    e.free_count += 1;
                } else {
                    // A free tile mapping to no extent should be impossible;
                    // park it on the orphan chain rather than leak it.
                    unsafe { (*tile).next = orphans };
                    orphans = tile;
                }
                tile = next;
            }

            // Pass 2: release fully-free extents, re-chain the rest. Start the
            // rebuilt free list from any orphans so none are lost.
            let tpe = class.tiles_per_extent;
            let tile_align = class.tile_align;
            let bump_end = class.bump_end;
            let mut new_head: *mut FreeTile = orphans;
            let mut released = 0usize;
            let mut released_count = 0usize;
            class.extents.retain(|e| {
                let e_end = unsafe { e.ptr.add(e.total_bytes) };
                let is_current = !bump_end.is_null() && bump_end == e_end;
                if e.free_count == tpe && !is_current {
                    let layout =
                        unsafe { Layout::from_size_align_unchecked(e.total_bytes, tile_align) };
                    unsafe { System.dealloc(e.ptr, layout) };
                    released += e.total_bytes;
                    released_count += 1;
                    false
                } else {
                    if !e.bucket.is_null() {
                        // Prepend this extent's self-contained bucket chain.
                        let mut t = e.bucket;
                        unsafe {
                            while !(*t).next.is_null() {
                                t = (*t).next;
                            }
                            (*t).next = new_head;
                        }
                        new_head = e.bucket;
                    }
                    true
                }
            });
            class.free_head = new_head;
            class.extents_released += released_count;
            released_total += released;
        }
        released_total
    }
}

/// Public allocator. Wraps `TileInner` in an `UnsafeCell` and asserts
/// `Sync` for the wasm32 single-threaded global-allocator slot.
pub struct TileAllocator {
    inner: UnsafeCell<TileInner>,
}

// SAFETY: wasm32-unknown-unknown is single-threaded. The global
// allocator is reached from exactly one execution context at a time.
// If Ruffle ever moves to wasm-threads, wrap `inner` in a `Mutex`.
unsafe impl Sync for TileAllocator {}

impl Default for TileAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl TileAllocator {
    pub const fn new() -> Self {
        Self {
            inner: UnsafeCell::new(TileInner::empty()),
        }
    }

    /// Snapshot the allocator's counters. Cheap — copies a handful of
    /// `usize`s per class. Safe to call from any runtime diagnostic
    /// path; the web binary re-exports it as a wasm-bindgen JS
    /// function.
    pub fn snapshot_stats(&self) -> TileAllocatorStats {
        // SAFETY: see Sync impl. Single-threaded on wasm32.
        let inner = unsafe { &*self.inner.get() };
        let mut classes = Vec::with_capacity(inner.class_count);
        for i in 0..inner.class_count {
            if let Some(c) = &inner.classes[i] {
                classes.push(c.snapshot());
            }
        }
        TileAllocatorStats {
            classes,
            fallback: TileFallbackStats {
                alloc_total: inner.fallback_alloc_total,
                dealloc_total: inner.fallback_dealloc_total,
                // The cumulative `u64` counters cannot wrap in practice
                // (see `TileInner` field doc). The live difference is
                // bounded by the WASM linear memory cap (4 GiB on
                // wasm32), so the down-cast to `usize` never truncates
                // information — and on 64-bit targets it's a no-op.
                bytes_alive: inner
                    .fallback_bytes_alloc
                    .saturating_sub(inner.fallback_bytes_dealloc)
                    as usize,
                size_histogram: inner.fallback_size_histogram,
            },
            cross_class_realloc_total: inner.cross_class_realloc_total,
        }
    }

    /// Release every fully-empty pool extent back to the system allocator and
    /// return the bytes reclaimed. Use from a memory-pressure or post-GC hook
    /// to undo a transient spike that stranded extents in the (otherwise
    /// grow-only) pool. Cheap when there is nothing to release; the cost scales
    /// with the number of free tiles, so prefer calling it while idle rather
    /// than per frame. See [`TileInner::trim_empty_extents`].
    pub fn trim(&self) -> usize {
        // SAFETY: see Sync impl. Single-threaded on wasm32.
        let inner = unsafe { &mut *self.inner.get() };
        unsafe { inner.trim_empty_extents() }
    }
}

unsafe impl GlobalAlloc for TileAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: see Sync impl.
        let inner = unsafe { &mut *self.inner.get() };
        inner.ensure_initialized();

        if let Some(class_idx) = inner.pick_class(layout) {
            let p = unsafe { inner.class_alloc(class_idx, layout) };
            if !p.is_null() {
                return p;
            }
            // add_extent failed (OOM). Fall through to system allocator.
        }

        // Fallback path.
        let mut p = unsafe { System.alloc(layout) };
        if p.is_null() {
            // dlmalloc could not satisfy the request within the current WASM
            // pages. A transient pool spike may have stranded reusable memory
            // in fully-empty extents — hand them back to the system allocator
            // and retry once. (Trim is allocation-free, so this can't recurse.)
            unsafe { inner.trim_empty_extents() };
            p = unsafe { System.alloc(layout) };
        }
        if !p.is_null() {
            inner.fallback_alloc_total += 1;
            inner.fallback_bytes_alloc += layout.size() as u64;
            // Fallback size histogram — gated symmetrically with the
            // per-class histograms. The scalar counters above (`*_total`
            // and `*_bytes`) stay always-on: they're cheap and useful
            // even in production builds.
            if ENABLE_HISTOGRAMS {
                let bucket = fallback_bucket_for_size(layout.size());
                inner.fallback_size_histogram[bucket] =
                    inner.fallback_size_histogram[bucket].saturating_add(1);
            }
        }
        p
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let inner = unsafe { &mut *self.inner.get() };
        // ensure_initialized has already run during the matching alloc;
        // but be defensive in case dealloc somehow runs first.
        inner.ensure_initialized();

        if let Some(class_idx) = inner.pick_class(layout) {
            unsafe { inner.class_dealloc(class_idx, ptr, layout) };
            return;
        }

        // Fallback path — mirror the alloc side.
        inner.fallback_dealloc_total += 1;
        inner.fallback_bytes_dealloc += layout.size() as u64;
        if ENABLE_HISTOGRAMS {
            let bucket = fallback_bucket_for_size(layout.size());
            inner.fallback_size_histogram[bucket] =
                inner.fallback_size_histogram[bucket].saturating_sub(1);
        }
        unsafe { System.dealloc(ptr, layout) };
    }

    // `alloc_zeroed` uses the default implementation (alloc + write_bytes).

    /// In-place `realloc` optimization. The default `GlobalAlloc::realloc`
    /// always does `alloc(new) + memcpy + dealloc(old)`, which holds two
    /// tiles live at once and pressures the source class's free list
    /// — a driver of peak-transient overshoots that can force extra
    /// extents under sustained reallocation.
    ///
    /// When `new_size` fits in the same size class as `layout.size()`,
    /// the existing tile is already big enough: we return it
    /// unchanged. Zero alloc, zero copy, zero dealloc — and crucially
    /// no transient where two tiles are simultaneously alive in the
    /// same class. The cross-class path falls back to the standard
    /// alloc+copy+dealloc sequence.
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let inner = unsafe { &mut *self.inner.get() };
        inner.ensure_initialized();

        // Build the would-be new layout (same alignment as the old one;
        // realloc never changes alignment).
        let new_layout = match Layout::from_size_align(new_size, layout.align()) {
            Ok(l) => l,
            Err(_) => return ptr::null_mut(),
        };
        let old_class = inner.pick_class(layout);
        let new_class = inner.pick_class(new_layout);

        // Fast path: both the old and new size land in the same size
        // class, so the existing tile is still big enough. Return the
        // same pointer; the caller can write up to `new_size` bytes
        // into it.
        if let Some(old_idx) = old_class
            && old_class == new_class
        {
            if let Some(c) = inner.classes[old_idx].as_mut() {
                c.inplace_realloc_total = c.inplace_realloc_total.saturating_add(1);
            }
            return ptr;
        }

        // Slow path: different class (or fallback path). Allocate a
        // fresh tile, copy the live bytes, free the old one.
        inner.cross_class_realloc_total = inner.cross_class_realloc_total.saturating_add(1);
        // In the slow path `new_size != layout.size()` is guaranteed:
        // if they were equal and align is invariant under realloc,
        // `pick_class` would have returned the same index → fast
        // inplace path above. So this comparison cleanly partitions
        // every slow-path call into "grow" or "shrink".
        let is_grow = new_size > layout.size();
        // Per-class outbound counter: charge the source class for
        // letting go of a buffer that needed to migrate. Only when
        // `old_class` is a pool class.
        if let Some(old_idx) = old_class
            && let Some(c) = inner.classes[old_idx].as_mut()
        {
            if is_grow {
                c.outbound_realloc_total_grow = c.outbound_realloc_total_grow.saturating_add(1);
            } else {
                c.outbound_realloc_total_shrink = c.outbound_realloc_total_shrink.saturating_add(1);
            }
        }
        // Per-class inbound counter: charge the destination class for
        // receiving a cross-class realloc. Only when `new_class` is a
        // pool class — cross-class reallocs that land in the fallback
        // are captured by `cross_class_realloc_total` alone (the gap
        // vs sum-of-inbound tells you how many reallocs outgrew the
        // pool entirely).
        if let Some(new_idx) = new_class
            && let Some(c) = inner.classes[new_idx].as_mut()
        {
            if is_grow {
                c.inbound_realloc_total_grow = c.inbound_realloc_total_grow.saturating_add(1);
            } else {
                c.inbound_realloc_total_shrink = c.inbound_realloc_total_shrink.saturating_add(1);
            }
        }
        let new_ptr = unsafe { self.alloc(new_layout) };
        if !new_ptr.is_null() {
            let copy_size = core::cmp::min(layout.size(), new_size);
            unsafe { core::ptr::copy_nonoverlapping(ptr, new_ptr, copy_size) };
            unsafe { self.dealloc(ptr, layout) };
        }
        new_ptr
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::alloc::{GlobalAlloc, Layout};

    /// Fill three extents of one class, free every tile, and confirm `trim`
    /// hands back the two fully-free non-current extents (16 MiB) while keeping
    /// the current bump extent — and that its recycled tiles stay usable.
    #[test]
    fn trim_releases_fully_empty_extents() {
        let a = TileAllocator::new();
        let layout = Layout::from_size_align(256, 8).unwrap();
        // Mirror config: extent_mb = 8 for the 256-byte class.
        let tpe = (8 * 1024 * 1024) / 256;
        let extent_bytes = tpe * 256;
        let n = tpe * 2 + 10; // two full extents + a partly-carved third

        let mut ptrs = Vec::with_capacity(n);
        for _ in 0..n {
            let p = unsafe { a.alloc(layout) };
            assert!(!p.is_null());
            ptrs.push(p);
        }

        let count_256 = |s: &TileAllocatorStats| {
            s.classes.iter().find(|c| c.size_max == 256).unwrap().extent_count
        };
        assert_eq!(count_256(&a.snapshot_stats()), 3);

        for p in &ptrs {
            unsafe { a.dealloc(*p, layout) };
        }

        let released = a.trim();
        assert_eq!(released, 2 * extent_bytes);

        // Stats must reflect the reclamation, not just the byte count: one
        // extent survives, `extents_released` records the two handed back, and
        // the derived counters re-track (capacity shrank so `free_tiles` drops
        // to one extent's worth; `extents_added` stays a lifetime total).
        let snap = a.snapshot_stats();
        let cls = snap.classes.iter().find(|c| c.size_max == 256).unwrap();
        assert_eq!(cls.extent_count, 1);
        assert_eq!(cls.extents_added, 3);
        assert_eq!(cls.extents_released, 2);
        assert_eq!(cls.extent_count, cls.extents_added - cls.extents_released);
        assert_eq!(cls.alive_tiles, 0);
        assert_eq!(cls.free_tiles, tpe);

        // Recycled tiles from the surviving extent must still allocate.
        for _ in 0..10 {
            assert!(!unsafe { a.alloc(layout) }.is_null());
        }

        // Trimming again is a no-op (nothing fully free now).
        assert_eq!(a.trim(), 0);
    }
}
