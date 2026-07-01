use ruffle_render::backend::ShapeHandle;
use swf::{Rectangle, Twips};

/// Number of (grid, scale)-keyed transformed-shape handles to cache per
/// shape source. FIFO eviction; mirrors `TessellationCache::SIZE = 4`.
const SCALE9_CACHE_SIZE: usize = 4;

/// Identity of a 9-slice transformed shape: the grid rectangle plus the
/// world-scale (quantized) at which it was tessellated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Scale9Key {
    pub grid: Rectangle<Twips>,
    pub bounds: Rectangle<Twips>,
    pub sx_q: i32,
    pub sy_q: i32,
}

impl Scale9Key {
    /// Quantize world scales to 1/256 increments so floating-point drift
    /// across frames doesn't cause cache misses on otherwise-identical scales.
    pub fn new(grid: Rectangle<Twips>, bounds: Rectangle<Twips>, sx: f32, sy: f32) -> Self {
        Self {
            grid,
            bounds,
            sx_q: (sx * 256.0).round() as i32,
            sy_q: (sy * 256.0).round() as i32,
        }
    }
}

/// Small FIFO-evicting cache of (key, handle) pairs.
#[derive(Clone, Default, Debug)]
pub(crate) struct Scale9Cache {
    entries: Vec<(Scale9Key, ShapeHandle)>,
}

impl Scale9Cache {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Returns the cached handle for `key`, or invokes `build` to create one
    /// and insert it. If `build` returns `None` (e.g. the source shape has
    /// fills that can't be safely transformed) the cache is left untouched.
    pub fn get_or_try_insert<F>(&mut self, key: Scale9Key, build: F) -> Option<ShapeHandle>
    where
        F: FnOnce() -> Option<ShapeHandle>,
    {
        if let Some(handle) = self.entries.iter().find(|(k, _)| k == &key) {
            return Some(handle.1.clone());
        }
        let handle = build()?;
        if self.entries.len() >= SCALE9_CACHE_SIZE {
            self.entries.remove(0);
        }
        self.entries.push((key, handle.clone()));
        Some(handle)
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}
