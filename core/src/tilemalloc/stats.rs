//! Runtime statistics for the tile allocator.
//!
//! Snapshots are read-only views taken at a single point in time. They
//! are produced by `TileAllocator::snapshot_stats` and rendered into
//! two transport formats — `to_json` (machine-readable, the stable
//! schema for tooling) and `to_table` (human-readable, fixed-width
//! columns for direct diff between snapshots). Both are surfaced
//! to JavaScript via wasm-bindgen exports in `web/src/lib.rs`.

use super::config::ENABLE_HISTOGRAMS;

/// Per–size-class snapshot.
#[derive(Clone, Debug, Default)]
pub struct TileClassStats {
    /// Minimum request size (in bytes) routed to this class
    /// (inclusive). Equals the previous class's `size_max + 1`, or `1`
    /// for the first class. Together with `size_max` it defines the
    /// closed interval `[size_min, size_max]` of `layout.size()` values
    /// that this class absorbs, and indexes `size_histogram` via
    /// `requested_size = size_min + i`.
    pub size_min: usize,

    /// Maximum request size (in bytes) served by this class — the tile
    /// size from the config. Allocations with
    /// `layout.size() <= size_max` (and `layout.align() <= tile_align`)
    /// are routed here.
    pub size_max: usize,

    /// Tiles currently sitting on this class's free list, immediately
    /// reusable on the next `alloc` for this class. Sum of
    /// `alive_tiles` and `free_tiles` is the total tile capacity
    /// across all extents currently held for this class.
    pub free_tiles: usize,

    /// Currently alive tiles (`alloc_total - dealloc_total`).
    pub alive_tiles: usize,

    /// Total bytes the extents of this class are currently occupying
    /// in the WASM linear memory. Equals
    /// `extent_count * size_max * tiles_per_extent`.
    pub extent_bytes: usize,

    /// Extents currently held for this class. Each extent contributes
    /// `tiles_per_extent` tile slots to capacity.
    pub extent_count: usize,

    /// Cumulative count of extents added to this class since boot
    /// (= calls to the system allocator to grow capacity). Each
    /// addition adds `tiles_per_extent` to capacity. Growth in this
    /// counter between snapshots means the working set for this class
    /// is exceeding the extent pool — consider increasing `extent_mb`
    /// in the config.
    pub extents_added: usize,

    /// Cumulative alloc calls served by this class (does NOT count
    /// allocations that fell back to the system allocator).
    pub alloc_total: usize,

    /// Cumulative dealloc calls served by this class.
    pub dealloc_total: usize,

    /// Peak `alive_tiles` ever observed for this class. Cumulative
    /// high-water mark across the lifetime of the allocator. Useful
    /// to decide whether the configured `extent_mb` is sized
    /// appropriately for the class — if `peak_alive_tiles >
    /// tiles_per_extent` (= `extent_bytes / size_max` per extent),
    /// the class needed more than one extent at some point.
    pub peak_alive_tiles: usize,

    /// **Live** internal-fragmentation bytes currently held by this
    /// class: sum of `(size_max - layout.size())` over every tile
    /// currently alive. Incremented in `class_alloc`, decremented in
    /// `class_dealloc` using `layout.size()` (which Rust's `GlobalAlloc`
    /// contract guarantees is the same value at alloc and dealloc).
    /// Reflects the present state of the pool, not a lifetime total.
    /// Pair with `requested_bytes_total` for the live waste-as-percent.
    pub wasted_bytes_total: u64,

    /// **Live** bytes actually requested by callers and currently held
    /// by tiles in this class: sum of `layout.size()` over the alive
    /// tiles. Same alloc/dealloc symmetry as `wasted_bytes_total`.
    pub requested_bytes_total: u64,

    /// Cumulative in-place realloc fast-path hits served by this
    /// class: `realloc` calls where both old and new `layout.size()`
    /// landed here, so we returned the same pointer without copy.
    /// Pair with `TileAllocatorStats::cross_class_realloc_total` to
    /// gauge how often the fast path actually kicks in.
    pub inplace_realloc_total: u64,

    /// Cross-class realloc **arrivals from a smaller origin** (grow
    /// landed here). `new_size > old_size` and `new_layout` lives in
    /// this class. Always 0 for the smallest pool class.
    pub inbound_realloc_total_grow: u64,

    /// Cross-class realloc **arrivals from a larger origin** (shrink
    /// landed here). `new_size < old_size` and `new_layout` lives in
    /// this class.
    pub inbound_realloc_total_shrink: u64,

    /// Cross-class realloc **departures toward a larger destination**
    /// (this class was outgrown). `new_size > old_size` and `layout`
    /// lived here.
    pub outbound_realloc_total_grow: u64,

    /// Cross-class realloc **departures toward a smaller destination**
    /// (this class lost a buffer to a smaller one). `new_size <
    /// old_size` and `layout` lived here. Always 0 for the smallest
    /// pool class.
    pub outbound_realloc_total_shrink: u64,

    /// **Live** histogram: one counter per `layout.size()` value in
    /// `size_min..=size_max`. Index `i` corresponds to
    /// `requested_size = size_min + i`. Length =
    /// `size_max - size_min + 1`. Each bucket counts the tiles of
    /// that requested size **currently alive** in the pool (not all
    /// allocs ever made). Diff between snapshots reveals which sizes
    /// grew or shrank over the window.
    pub size_histogram: Vec<u64>,

    /// **Live** histogram of `layout.align()` for tiles currently alive
    /// in this class. Buckets:
    /// - `[0]` → `align == 1` (e.g. `Box<u8>`, `Vec<u8>`, `String` backing)
    /// - `[1]` → `align == 2` (e.g. `Box<u16>`)
    /// - `[2]` → `align == 4` (structs with `u32` / `usize` / pointer on wasm32)
    /// - `[3]` → `align == 8` (structs containing `u64`)
    /// - `[4]` → `align == 16` (SIMD `v128`, some allocator metadata)
    /// - `[5]` → `align == 32` (rare; some `#[repr(align(32))]`)
    /// - `[6]` → `align == 64` (cache-line padding, anti-false-sharing)
    /// - `[7]` → `align ≥ 128` (saturating; very rare in Rust user code)
    ///
    /// The bucket sum equals `alive_tiles` (every alive tile counted
    /// once). Cross-reference with `size_histogram` to tell apart
    /// "this size lands here because of size alone — a smaller class
    /// with align ≥ requested would have fit" from "this size lands
    /// here because no smaller class has enough align".
    pub align_histogram: [u64; 8],
}

/// Statistics for allocations that bypassed the tile pools and went
/// straight to the system allocator (dlmalloc on wasm32). These are
/// the "long tail" — sizes larger than the biggest class, or
/// over-aligned requests.
#[derive(Clone, Debug, Default)]
pub struct TileFallbackStats {
    pub alloc_total: usize,
    pub dealloc_total: usize,

    /// Approximate live bytes in fallback path (`alloc_bytes -
    /// dealloc_bytes`). Useful to see if the long tail is itself
    /// growing.
    pub bytes_alive: usize,

    /// **Live** histogram of fallback allocations currently outstanding,
    /// bucketed log-style by `layout.size()`. Boundaries (inclusive
    /// upper): `[0]=512, [1]=1024, [2]=2048, [3]=4096, [4]=8192,
    /// [5]=16384, [6]=32768, [7]=65536, [8]=131072, [9]=∞`. Each bucket
    /// counts fallback allocations of that size class that are
    /// currently alive (the alloc/dealloc symmetry is preserved).
    /// A sustained large count in some bucket signals a hot fallback
    /// path worth absorbing by adding a tile size class above the
    /// current ceiling.
    pub size_histogram: [u64; 10],
}

/// Top-level snapshot.
#[derive(Clone, Debug, Default)]
pub struct TileAllocatorStats {
    pub classes: Vec<TileClassStats>,
    pub fallback: TileFallbackStats,

    /// Cumulative count of realloc calls that could not be served by
    /// the in-place fast path. Sum of: tile-to-tile cross-class
    /// transitions, tile-to-fallback transitions, fallback-to-tile
    /// transitions, and fallback-to-fallback transitions. Compare
    /// against the sum of `TileClassStats::inplace_realloc_total`
    /// across all classes to compute the fast-path hit ratio.
    pub cross_class_realloc_total: u64,
}

impl TileAllocatorStats {
    /// Total bytes currently held by all extents combined. Together
    /// with `fallback.bytes_alive` this is the tile allocator's net
    /// contribution to the WASM linear memory footprint.
    pub fn total_extent_bytes(&self) -> usize {
        self.classes.iter().map(|c| c.extent_bytes).sum()
    }

    /// Total live bytes across all tile pools (= sum of
    /// `alive_tiles * size_bytes`). This is the working set actually
    /// in use, distinct from `total_extent_bytes` which is the
    /// reserved capacity (live + free).
    pub fn total_tile_alive_bytes(&self) -> usize {
        self.classes
            .iter()
            .map(|c| c.alive_tiles * c.size_max)
            .sum()
    }

    /// Aggregate live waste across all classes (in bytes). This is the
    /// total internal-fragmentation overhead currently consumed by
    /// alive tiles — i.e. how many bytes the pool is rounding up
    /// "right now". Diff between snapshots reveals where rounding is
    /// growing or shrinking.
    pub fn total_wasted_bytes(&self) -> u64 {
        self.classes.iter().map(|c| c.wasted_bytes_total).sum()
    }

    /// Aggregate live requested bytes across all classes.
    pub fn total_requested_bytes(&self) -> u64 {
        self.classes.iter().map(|c| c.requested_bytes_total).sum()
    }

    /// Pretty-print the snapshot as a single line. The whole snapshot
    /// is wrapped in `[...]`; each record (one per size class plus
    /// fallback and totals) is wrapped in `{...}` and carries
    /// `key=value` pairs inline. Designed to be emitted as a String.
    ///
    /// Output shape (one line):
    ///
    /// ```text
    /// [{class=8 free=63391 alive=985185 ext=1 added=1 alloc=1123812 dealloc=138627 peak=985300 live_mb=7.52 util_pct=92.5 waste_pct=0.0}...{fallback alloc=190934 dealloc=134793 alive_mb=144.66}{totals pool_mb=192.00 tile_live_mb=80.74 live_tot_mb=225.40 waste_mb=6.8 waste_pct=8.4 realloc_inplace=12345 realloc_inbound_grow=54 realloc_inbound_shrink=35 realloc_outbound_grow=42 realloc_outbound_shrink=35 realloc_cross=678}]
    /// ```
    ///
    /// Per-class fields (in the order they appear, matching the JSON
    /// schema where applicable):
    /// - `class` = `size_max` (the tile size of the class).
    /// - `free` / `alive` = `free_tiles` / `alive_tiles`.
    /// - `ext` / `added` = `extent_count` / `extents_added`.
    /// - `alloc` / `dealloc` = `alloc_total` / `dealloc_total`.
    /// - `peak` = lifetime peak of `alive_tiles` (sizing hint for `extent_mb`).
    /// - `live_mb` = `alive_tiles * size_max / 1 MiB` (the working set
    ///   actually used).
    /// - `util_pct` = current `alive_bytes * 100 / extent_bytes`
    ///   (= how much of the reserved capacity is actually live now).
    ///   Low util_pct means over-allocated extents; consider shrinking
    ///   `extent_mb`.
    /// - `waste_pct` = live `wasted / (wasted + requested) * 100`
    ///   (= internal-fragmentation share for tiles **currently alive**
    ///   in this class). Differs between snapshots iff the alive size
    ///   distribution changes.
    ///
    /// Totals fields:
    /// - `waste_mb` = aggregate live waste in MiB across all classes.
    /// - `waste_pct` = aggregate over all classes.
    /// - `realloc_inplace` = sum of all classes' fast-path realloc hits.
    /// - `realloc_inbound_grow` / `realloc_inbound_shrink` = sums of
    ///   cross-class realloc arrivals into pool classes, split by
    ///   whether the call was a grow (`new > old`) or shrink. Both
    ///   `≤ realloc_cross`; the gap vs `realloc_cross` is the
    ///   contribution of reallocs whose destination was the fallback.
    /// - `realloc_outbound_grow` / `realloc_outbound_shrink` =
    ///   mirror pair: sums of cross-class realloc departures from
    ///   pool classes, split the same way.
    /// - `realloc_cross` = realloc calls that needed alloc+copy+dealloc.
    ///
    /// The per-class `size_histogram` and `size_min` are intentionally
    /// NOT emitted here (too verbose / derivable from `class` — find
    /// them in `to_json`). Same for the fallback `size_histogram`.
    pub fn to_table(&self) -> String {
        const MIB: f64 = 1024.0 * 1024.0;
        let mut s = String::with_capacity(180 * (self.classes.len() + 3));
        s.push('[');
        for c in &self.classes {
            let live_bytes = c.alive_tiles * c.size_max;
            let live_mib = live_bytes as f64 / MIB;
            let waste_pct = waste_percent(c.wasted_bytes_total, c.requested_bytes_total);
            let util_pct = if c.extent_bytes == 0 {
                0.0
            } else {
                live_bytes as f64 * 100.0 / c.extent_bytes as f64
            };
            s.push_str(&format!(
                "{{class={} free={} alive={} ext={} added={} alloc={} dealloc={} peak={} live_mb={:.2} util_pct={:.1} waste_pct={:.1}}}",
                c.size_max,
                c.free_tiles,
                c.alive_tiles,
                c.extent_count,
                c.extents_added,
                c.alloc_total,
                c.dealloc_total,
                c.peak_alive_tiles,
                live_mib,
                util_pct,
                waste_pct,
            ));
        }
        let tile_live = self.total_tile_alive_bytes();
        let pool = self.total_extent_bytes();
        let fb_alive = self.fallback.bytes_alive;
        let live_tot = tile_live + fb_alive;
        let total_wasted = self.total_wasted_bytes();
        let total_requested = self.total_requested_bytes();
        let waste_pct_tot = waste_percent(total_wasted, total_requested);
        let total_inplace: u64 = self.classes.iter().map(|c| c.inplace_realloc_total).sum();
        let total_inbound_grow: u64 = self
            .classes
            .iter()
            .map(|c| c.inbound_realloc_total_grow)
            .sum();
        let total_inbound_shrink: u64 = self
            .classes
            .iter()
            .map(|c| c.inbound_realloc_total_shrink)
            .sum();
        let total_outbound_grow: u64 = self
            .classes
            .iter()
            .map(|c| c.outbound_realloc_total_grow)
            .sum();
        let total_outbound_shrink: u64 = self
            .classes
            .iter()
            .map(|c| c.outbound_realloc_total_shrink)
            .sum();
        s.push_str(&format!(
            "{{fallback alloc={} dealloc={} alive_mb={:.2}}}",
            self.fallback.alloc_total,
            self.fallback.dealloc_total,
            fb_alive as f64 / MIB,
        ));
        s.push_str(&format!(
            "{{totals pool_mb={:.2} tile_live_mb={:.2} live_tot_mb={:.2} waste_mb={:.2} waste_pct={:.1} realloc_inplace={} realloc_inbound_grow={} realloc_inbound_shrink={} realloc_outbound_grow={} realloc_outbound_shrink={} realloc_cross={}}}",
            pool as f64 / MIB,
            tile_live as f64 / MIB,
            live_tot as f64 / MIB,
            total_wasted as f64 / MIB,
            waste_pct_tot,
            total_inplace,
            total_inbound_grow,
            total_inbound_shrink,
            total_outbound_grow,
            total_outbound_shrink,
            self.cross_class_realloc_total,
        ));
        s.push(']');
        s
    }

    /// Serialize the snapshot as a compact JSON string for tooling
    /// (machine-readable contract). No whitespace, no escaping needed
    /// (all values are non-negative integers). Schema:
    ///
    /// ```json
    /// {
    ///   "fallback": { "alloc_total": N, "dealloc_total": N,
    ///                 "bytes_alive": N,
    ///                 "size_histogram": [N, ...]  // gated, see note },
    ///   "totals":   { "extent_bytes": N, "class_count": N,
    ///                 "wasted_bytes_total": N, "requested_bytes_total": N,
    ///                 "inplace_realloc_total": N,
    ///                 "cross_class_realloc_total": N },
    ///   "classes":  [
    ///     { "size_min": N, "size_max": N,
    ///       "free_tiles": N, "alive_tiles": N,
    ///       "extent_bytes": N, "extent_count": N, "extents_added": N,
    ///       "alloc_total": N, "dealloc_total": N,
    ///       "peak_alive_tiles": N,
    ///       "wasted_bytes_total": N, "requested_bytes_total": N,
    ///       "inplace_realloc_total": N,
    ///       "inbound_realloc_total_grow": N,
    ///       "inbound_realloc_total_shrink": N,
    ///       "outbound_realloc_total_grow": N,
    ///       "outbound_realloc_total_shrink": N,
    ///       "size_histogram": [N, ...],            // see note below
    ///       "align_histogram": [N, N, ..., N] },   // see note below
    ///     ...
    ///   ]
    /// }
    /// ```
    ///
    /// **Histograms gating**: the per-class `size_histogram` /
    /// `align_histogram` and the fallback `size_histogram` are
    /// emitted only when the build-time const
    /// `config::ENABLE_HISTOGRAMS` is `true`. When it's `false`, those
    /// fields are omitted entirely from their parent object — not
    /// emitted as empty arrays — so the schema is conditional on the
    /// build configuration.
    ///
    /// All `*_total` and `*_histogram` values are **live** (= reflecting
    /// the current pool state, alloc-minus-dealloc), except for
    /// `alloc_total`, `dealloc_total`, `extents_added`,
    /// `inplace_realloc_total`, `inbound_realloc_total_{grow,shrink}`,
    /// `outbound_realloc_total_{grow,shrink}`,
    /// `cross_class_realloc_total`, and `peak_alive_tiles`, which are
    /// lifetime cumulative / high-water.
    ///
    /// `size_histogram[i]` for a class corresponds to
    /// `requested_size = size_min + i`; for fallback, see
    /// `TileFallbackStats::size_histogram` for the bucket boundaries.
    pub fn to_json(&self) -> String {
        let mut s = String::with_capacity(256 + self.classes.len() * 320);
        s.push('{');

        s.push_str("\"fallback\":{");
        s.push_str(&format!("\"alloc_total\":{},", self.fallback.alloc_total));
        s.push_str(&format!(
            "\"dealloc_total\":{},",
            self.fallback.dealloc_total
        ));
        s.push_str(&format!("\"bytes_alive\":{}", self.fallback.bytes_alive));
        if ENABLE_HISTOGRAMS {
            s.push_str(",\"size_histogram\":");
            append_u64_array(&mut s, &self.fallback.size_histogram);
        }
        s.push_str("},");

        let total_inplace: u64 = self.classes.iter().map(|c| c.inplace_realloc_total).sum();
        s.push_str("\"totals\":{");
        s.push_str(&format!("\"extent_bytes\":{},", self.total_extent_bytes()));
        s.push_str(&format!("\"class_count\":{},", self.classes.len()));
        s.push_str(&format!(
            "\"wasted_bytes_total\":{},",
            self.total_wasted_bytes()
        ));
        s.push_str(&format!(
            "\"requested_bytes_total\":{},",
            self.total_requested_bytes()
        ));
        s.push_str(&format!("\"inplace_realloc_total\":{},", total_inplace));
        s.push_str(&format!(
            "\"cross_class_realloc_total\":{}",
            self.cross_class_realloc_total
        ));
        s.push_str("},");

        s.push_str("\"classes\":[");
        for (i, c) in self.classes.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            s.push('{');
            s.push_str(&format!("\"size_min\":{},", c.size_min));
            s.push_str(&format!("\"size_max\":{},", c.size_max));
            s.push_str(&format!("\"free_tiles\":{},", c.free_tiles));
            s.push_str(&format!("\"alive_tiles\":{},", c.alive_tiles));
            s.push_str(&format!("\"extent_bytes\":{},", c.extent_bytes));
            s.push_str(&format!("\"extent_count\":{},", c.extent_count));
            s.push_str(&format!("\"extents_added\":{},", c.extents_added));
            s.push_str(&format!("\"alloc_total\":{},", c.alloc_total));
            s.push_str(&format!("\"dealloc_total\":{},", c.dealloc_total));
            s.push_str(&format!("\"peak_alive_tiles\":{},", c.peak_alive_tiles));
            s.push_str(&format!("\"wasted_bytes_total\":{},", c.wasted_bytes_total));
            s.push_str(&format!(
                "\"requested_bytes_total\":{},",
                c.requested_bytes_total
            ));
            s.push_str(&format!(
                "\"inplace_realloc_total\":{},",
                c.inplace_realloc_total
            ));
            s.push_str(&format!(
                "\"inbound_realloc_total_grow\":{},",
                c.inbound_realloc_total_grow
            ));
            s.push_str(&format!(
                "\"inbound_realloc_total_shrink\":{},",
                c.inbound_realloc_total_shrink
            ));
            s.push_str(&format!(
                "\"outbound_realloc_total_grow\":{},",
                c.outbound_realloc_total_grow
            ));
            s.push_str(&format!(
                "\"outbound_realloc_total_shrink\":{}",
                c.outbound_realloc_total_shrink
            ));
            // Histograms are emitted only when `config::ENABLE_HISTOGRAMS`
            // is `true` at build time. When off, `size_histogram` is
            // an empty `Vec::new()` and `align_histogram` is all-zero;
            // emitting them would waste bytes and lie about the data.
            if ENABLE_HISTOGRAMS {
                s.push_str(",\"size_histogram\":");
                append_u64_array(&mut s, &c.size_histogram);
                s.push_str(",\"align_histogram\":");
                append_u64_array(&mut s, &c.align_histogram);
            }
            s.push('}');
        }
        s.push(']');

        s.push('}');
        s
    }
}

/// Append a `[1,2,3,...]` JSON array of u64 to `s`. Compact, no
/// whitespace. Used for the per-class and fallback size histograms.
fn append_u64_array(s: &mut String, arr: &[u64]) {
    s.push('[');
    for (i, v) in arr.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&v.to_string());
    }
    s.push(']');
}

/// Compute `wasted / (wasted + requested) * 100`, returning 0.0 when
/// the denominator is zero (= no alloc routed through this class yet).
fn waste_percent(wasted: u64, requested: u64) -> f64 {
    let denom = wasted + requested;
    if denom == 0 {
        0.0
    } else {
        wasted as f64 * 100.0 / denom as f64
    }
}
