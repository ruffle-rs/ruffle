//! Tilemalloc — size-class allocator with intrusive free lists.
//!
//! ## Why this allocator exists
//!
//! Long-running WASM workloads that sustain heavy churn of small
//! allocations — many short-lived tiles under a few hundred bytes —
//! push general-purpose allocators (dlmalloc on the
//! `wasm32-unknown-unknown` target) into per-call costs that can grow
//! with the live set. The cost surfaces as free-list traversal and
//! coalescing work that compounds on top of every alloc/free site in
//! hot paths, even when the steady-state working set is bounded.
//!
//! This allocator targets that pathology. For the configured size
//! classes (see `config.rs`):
//!
//! - **alloc**: pop the head of an intrusive free list if non-empty,
//!   otherwise carve one tile off a bump pointer into the current
//!   extent. Either way O(1), no list walk, no coalescing search.
//! - **dealloc**: push back onto the free-list head — same cost.
//! - **add_extent** (rare, amortized): when both the free list is
//!   empty and the bump range is exhausted, ask the system allocator
//!   for one extent and install it as the new bump range. No per-tile
//!   setup write at extent creation — the per-tile cost is paid lazily
//!   as the bump pointer advances on subsequent allocs.
//! - **fallback**: anything bigger than the largest class, or with
//!   alignment greater than the largest class's auto-derived
//!   `tile_align`, transparently goes to the system allocator
//!   (dlmalloc on `wasm32-unknown-unknown`).
//!
//! Critically, the allocator **does not coalesce on free**. Tiles
//! freed by an object's destruction stay carved at their original
//! size, ready to be popped by the next allocation of the same shape.
//! A workload that repeatedly constructs and destructs objects of
//! similar layout settles into a steady state where later
//! constructions reuse the exact tiles earlier destructions freed
//! — **no additional `memory.grow()` calls** past the initial extent
//! fills.
//!
//! ## How to use
//!
//! Install as the binary's global allocator:
//!
//! ```ignore
//! use ruffle_core::tilemalloc::TileAllocator;
//!
//! #[global_allocator]
//! static GLOBAL: TileAllocator = TileAllocator::new();
//! ```
//!
//! Inspect at runtime via `GLOBAL.snapshot_stats()` (cheap; copies
//! a handful of `usize`s per class).
//!
//! ## How to tune
//!
//! Edit `config::TILE_CONFIG` and rebuild. At runtime, obtain a JSON
//! snapshot of per-class state via `TileAllocator::snapshot_stats`
//! (see the `stats` module for the full schema) — the web binary
//! re-exports it as a wasm-bindgen JS function. A class with steadily
//! growing `extents_added` wants a larger `extent_mb`; a class with
//! `alive_tiles == 0` across many snapshots is unused and can be
//! removed (or merged with an adjacent class).
//!
//! ## Safety
//!
//! The allocator is `Sync` via interior mutability with no locking,
//! which is sound on `wasm32-unknown-unknown` because the runtime is
//! single-threaded. On wasm-threads (when/if that target is enabled)
//! the inner state needs a `Mutex` wrapper.

pub mod config;
mod core;
pub mod stats;

pub use config::{TILE_CONFIG, TileClass};
pub use core::TileAllocator;
pub use stats::{TileAllocatorStats, TileClassStats, TileFallbackStats};

/// Shared global instance. The web binary's `#[global_allocator]`
/// wrapper delegates to this, and the wasm-bindgen JS stats exports
/// read their snapshots from here. On non-WASM targets the static is
/// harmless: it allocates no memory until first use, and first use
/// only happens if a code path explicitly hits the allocator wrapper
/// that references it.
pub static GLOBAL_TILEMALLOC: TileAllocator = TileAllocator::new();
