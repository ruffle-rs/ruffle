use ruffle_render::backend::ShapeHandle;
use ruffle_render::shape_utils::{Scale9, Scale9Space};
use std::cell::{OnceCell, RefCell};
use swf::{Rectangle, Twips};

/// Identifies a 9-sliced tessellation.
///
/// The scale is baked into the vertices and divided back out of the render matrix, so it is
/// compared by bit pattern: reusing a near-miss renders the object at the wrong size.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Scale9Key {
    bounds: Rectangle<Twips>,
    grid: Rectangle<Twips>,
    scale_x: u32,
    scale_y: u32,
    /// The linear part by bit pattern, the translation in twips. `None` when nothing folds,
    /// which covers both the owning object and a detached child: their geometry is the same,
    /// only the render matrix differs.
    to_grid_space: Option<([u32; 4], [i32; 2])>,
    tessellation_scale: i32,
    /// Distinguishes the interpolated frames of a morph; zero for anything that does not morph.
    ratio: u16,
}

impl Scale9Key {
    pub(crate) fn new(scale9: &Scale9, space: Scale9Space) -> Self {
        Self {
            bounds: scale9.bounds,
            grid: scale9.grid,
            scale_x: scale9.scale_x.to_bits(),
            scale_y: scale9.scale_y.to_bits(),
            to_grid_space: match space {
                Scale9Space::Child(m) => Some((
                    [m.a.to_bits(), m.b.to_bits(), m.c.to_bits(), m.d.to_bits()],
                    [m.tx.get(), m.ty.get()],
                )),
                Scale9Space::Own | Scale9Space::Detached(_) => None,
            },
            tessellation_scale: 0,
            ratio: 0,
        }
    }

    /// Unlike the rest of the key this is bucketed, since it only sets how finely curves are
    /// subdivided and not where any vertex lands.
    pub(crate) fn with_tessellation_scale(mut self, scale: f32) -> Self {
        self.tessellation_scale = (scale * 16.0) as i32;
        self
    }

    pub(crate) fn with_ratio(mut self, ratio: u16) -> Self {
        self.ratio = ratio;
        self
    }
}

type Slot = RefCell<Option<(Scale9Key, ShapeHandle)>>;

/// One-slot cache of the most recent 9-sliced tessellation of a shape.
///
/// An unsliced shape tessellates once and rescales through the render matrix, but
/// slicing bakes the scale into the vertices, so every distinct scale is a distinct
/// tessellation; without a cache a gridded object would re-tessellate every frame.
/// One slot suffices because an object renders against a single grid and scale per
/// frame, and it is allocated on first sliced render so the common ungridded object
/// pays a pointer.
#[derive(Clone, Debug, Default)]
pub(crate) struct Scale9Cache(OnceCell<Box<Slot>>);

impl Scale9Cache {
    /// The cached handle for `key`, or the one `build` makes, kept for the next lookup.
    /// `None` from `build` passes through uncached.
    pub(crate) fn get_or_register(
        &self,
        key: Scale9Key,
        build: impl FnOnce() -> Option<ShapeHandle>,
    ) -> Option<ShapeHandle> {
        let slot = self.0.get_or_init(Default::default);
        if let Some((cached, handle)) = slot.borrow().as_ref()
            && *cached == key
        {
            return Some(handle.clone());
        }
        let handle = build()?;
        *slot.borrow_mut() = Some((key, handle.clone()));
        Some(handle)
    }

    pub(crate) fn clear(&self) {
        if let Some(slot) = self.0.get() {
            *slot.borrow_mut() = None;
        }
    }
}
