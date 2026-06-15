//! Tilemalloc size-class configuration.
//!
//! **THIS IS THE FILE TO EDIT** when tuning the allocator for a different
//! workload. After editing, rebuild the web binary:
//!
//! ```text
//! cargo build -p ruffle_web --target wasm32-unknown-unknown
//! ```
//!
//! Each `TileClass` entry describes one size class served by the tile
//! allocator. Allocations whose `Layout::size <= class.size_bytes` AND
//! `Layout::align <= class.tile_align` are routed to that class's free
//! list. Requests larger than the biggest class — or with an alignment
//! larger than the largest class's `tile_align` — fall back to the
//! system allocator (dlmalloc on `wasm32-unknown-unknown`).
//!
//! `tile_align` is **not** a config knob — it is auto-derived as the
//! greatest power-of-two divisor of `size_bytes` (i.e. the lowest set
//! bit). For pow-2 sizes this is identical to `size_bytes` itself; for
//! non-pow-2 sizes like 24 / 48 / 96 the alignment falls to the
//! closest pow-2 divisor (8 / 16 / 32 respectively), which still
//! covers the vast majority of Rust allocations (most types have
//! `align <= 8`). Requests whose `Layout::align` exceeds a class's
//! derived `tile_align` skip that class and fall through to one with
//! sufficient alignment, or to the system allocator.
//!
//! ## How `extent_mb` works
//!
//! When a class's free list is empty, the allocator adds one **extent**:
//! it asks the system allocator for a contiguous region of approximately
//! `extent_mb` megabytes, then splits the extent into
//! `floor((extent_mb * 1 MiB) / size_bytes)` free tiles and pushes them
//! all on the free list. Subsequent allocations from that class then pop
//! from the head in O(1) until the list is empty again.
//!
//! - **Larger `extent_mb`** → fewer extents added → less amortized
//!   overhead, but more memory pre-allocated up front.
//! - **Smaller `extent_mb`** → more extents added over time, but tighter
//!   memory tracking.
//!
//! The actual byte count is `extent_mb * 1024 * 1024`, rounded down to
//! the nearest multiple of `size_bytes` when carving tiles. Tiny leftover
//! bytes at the tail of each extent (`< size_bytes` worth) are unused.
//!
//! ## Choosing the size classes
//!
//! Pick size classes that cover the buckets with the highest churn —
//! those are the ones producing the per-frame allocator pressure that
//! this allocator is designed to eliminate. Anything above the largest
//! configured class falls back to dlmalloc and pays the standard
//! fragmentation cost; we accept that for the long tail of rare large
//! allocations.

/// Build-time toggle for the per-class `size_histogram` and
/// `align_histogram` instrumentation, plus the fallback
/// `size_histogram`.
///
/// When `true`, every `class_alloc` / `class_dealloc` updates one
/// `Vec<u64>` entry (size_histogram, heap-indirect — the dominant
/// hot-path cost of the histogram instrumentation) plus one
/// `[u64; 8]` entry (align_histogram, inline), and every fallback
/// alloc/dealloc updates one `[u64; 10]` entry. When `false`, those
/// updates are dead-code-eliminated by LLVM, the histogram backing
/// storage stays empty (`Vec::new()` with `cap == 0`, inline arrays
/// stay all-zero), and the JSON output omits the histogram fields
/// entirely. The `to_table` one-liner is unaffected — it never
/// emitted histograms.
///
/// **Why build-time, not runtime**: histograms accumulate **live**
/// state (alloc-minus-dealloc per bucket). Switching at runtime would
/// leave the counters out of sync with what's actually in the pool —
/// you'd see only the deltas since the toggle, with no way to
/// reconcile against the alive tiles already in flight. Either you
/// collect from boot or you don't collect at all.
///
/// Default `true` for diagnostic-friendly builds. Flip to `false` for
/// production / performance-sensitive builds to reclaim the per-alloc
/// cost of the heap-indirect `Vec<u64>` update.
pub const ENABLE_HISTOGRAMS: bool = false;

/// One size class served by the tile allocator. See module doc for
/// detailed tuning guidance.
#[derive(Clone, Copy)]
pub struct TileClass {
    /// Maximum allocation size (in bytes) served by this class. A
    /// request for `N` bytes is rounded up to the smallest class with
    /// `size_bytes >= N`. The tile alignment is auto-derived as the
    /// greatest power-of-two divisor of `size_bytes`.
    pub size_bytes: usize,

    /// Extent size in **megabytes**. Each newly added extent allocates
    /// approximately this many MiB from the system allocator and carves
    /// it into `floor((extent_mb * 1 MiB) / size_bytes)` tiles. Must be
    /// at least 1.
    pub extent_mb: usize,
}

/// **EDIT HERE** to change the tilemalloc size classes.
///
/// The default below is an opinionated starting point covering the
/// 8-256 byte range at sub-power-of-two granularity, with 8 MiB of
/// extent memory reserved per class. Anything outside that range falls
/// back to the system allocator. Inspect per-class activity at runtime
/// via `TileAllocator::snapshot_stats` (re-exported as a wasm-bindgen
/// JS function by the web binary) and tune for your workload.
///
/// Constraints:
/// - Entries MUST be in strictly ascending order of `size_bytes` (the
///   picker walks the array once and picks the first class large
///   enough).
/// - `size_bytes` MUST be at least `core::mem::size_of::<*mut ()>()`
///   (4 bytes on wasm32) — the intrusive free list reuses the first
///   bytes of a free tile to store a `next` pointer.
/// - `extent_mb` MUST be at least 1.
/// - `TILE_CONFIG.len()` MUST be `<= MAX_CLASSES` in `core.rs`
///   (currently 128).
#[rustfmt::skip]
pub const TILE_CONFIG: &[TileClass] = &[
    // step 8
    TileClass { size_bytes:    8, extent_mb: 8 },
    TileClass { size_bytes:   16, extent_mb: 8 },
    TileClass { size_bytes:   24, extent_mb: 8 },
    TileClass { size_bytes:   32, extent_mb: 8 },
    TileClass { size_bytes:   40, extent_mb: 8 },
    TileClass { size_bytes:   48, extent_mb: 8 },
    TileClass { size_bytes:   56, extent_mb: 8 },
    TileClass { size_bytes:   64, extent_mb: 8 },
    // step 16
    TileClass { size_bytes:   80, extent_mb: 8 },
    TileClass { size_bytes:   96, extent_mb: 8 },
    TileClass { size_bytes:  112, extent_mb: 8 },
    TileClass { size_bytes:  128, extent_mb: 8 },
    TileClass { size_bytes:  144, extent_mb: 8 },
    TileClass { size_bytes:  160, extent_mb: 8 },
    TileClass { size_bytes:  176, extent_mb: 8 },
    TileClass { size_bytes:  192, extent_mb: 8 },
    TileClass { size_bytes:  208, extent_mb: 8 },
    TileClass { size_bytes:  224, extent_mb: 8 },
    TileClass { size_bytes:  240, extent_mb: 8 },
    TileClass { size_bytes:  256, extent_mb: 8 },
];
