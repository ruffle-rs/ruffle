use crate::context::RenderContext;
use crate::drawing::Drawing;
use crate::prelude::*;
use ruffle_render::backend::null::NullBitmapSource;
use ruffle_render::backend::{RenderBackend, ShapeHandle};
use ruffle_render::bitmap::{Bitmap, BitmapInfo};
use ruffle_render::error::Error;
use ruffle_render::transform::Transform;

use std::cell::{Cell, OnceCell, Ref, RefCell};
use std::rc::Rc;

#[derive(Debug, Clone)]
enum SwfGlyphOrShape {
    Glyph(swf::Glyph),
    Shape {
        shape: swf::Shape,
        // Handle to registered shape, loaded lazily on first render of this glyph.
        handle: Option<ShapeHandle>,
    },
    Poisoned,
}

impl SwfGlyphOrShape {
    fn shape(&mut self) -> (&mut swf::Shape, &mut Option<ShapeHandle>) {
        if let Self::Glyph(_) = self
            && let Self::Glyph(glyph) = core::mem::replace(self, Self::Poisoned)
        {
            *self = Self::Shape {
                shape: ruffle_render::shape_utils::swf_glyph_to_shape(glyph),
                handle: None,
            };
        }

        match self {
            SwfGlyphOrShape::Shape { shape, handle } => (shape, handle),
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum GlyphRenderData {
    Shape(ShapeHandle),
    Bitmap {
        info: BitmapInfo,
        tx: Twips,
        ty: Twips,
    },
}

impl GlyphRenderData {
    pub fn from_shape(shape_handle: ShapeHandle) -> Self {
        Self::Shape(shape_handle)
    }

    pub fn from_bitmap(bitmap_info: BitmapInfo, tx: Twips, ty: Twips) -> Self {
        Self::Bitmap {
            info: bitmap_info,
            tx,
            ty,
        }
    }
}

#[derive(Debug, Clone)]
enum GlyphShape {
    Swf(Box<RefCell<SwfGlyphOrShape>>),
    Drawing(Box<Drawing>),
    Bitmap(Rc<GlyphBitmap<'static>>),
    None,
}

impl GlyphShape {
    pub fn hit_test(&self, point: Point<Twips>, local_matrix: &Matrix) -> bool {
        match self {
            GlyphShape::Swf(glyph) => {
                let mut glyph = glyph.borrow_mut();
                let (shape, _) = glyph.shape();
                shape.shape_bounds.contains(point)
                    && ruffle_render::shape_utils::shape_hit_test(shape, point, local_matrix)
            }
            GlyphShape::Drawing(drawing) => drawing.hit_test(point, local_matrix),
            GlyphShape::Bitmap(_) => {
                // TODO Implement this.
                true
            }
            GlyphShape::None => false,
        }
    }

    pub fn register(&self, renderer: &mut dyn RenderBackend) -> Option<GlyphRenderData> {
        match self {
            GlyphShape::Swf(glyph) => {
                let mut glyph = glyph.borrow_mut();
                let (shape, handle) = glyph.shape();
                handle.get_or_insert_with(|| {
                    renderer.register_shape((&*shape).into(), &NullBitmapSource)
                });
                handle.clone().map(GlyphRenderData::from_shape)
            }
            GlyphShape::Drawing(drawing) => drawing
                .register_or_replace(renderer)
                .map(GlyphRenderData::from_shape),
            GlyphShape::Bitmap(bitmap) => bitmap
                .get_bitmap_info_or_register(renderer)
                .as_ref()
                .inspect_err(|err| {
                    tracing::error!(
                        "Failed to register glyph as a bitmap: {err}, glyphs will be missing"
                    )
                })
                .ok()
                .cloned()
                .map(|info| GlyphRenderData::from_bitmap(info, bitmap.tx, bitmap.ty)),
            GlyphShape::None => None,
        }
    }
}

/// A Bitmap that can be registered to a RenderBackend.
struct GlyphBitmap<'a> {
    bitmap: Cell<Option<Bitmap<'a>>>,
    handle: OnceCell<Result<BitmapInfo, Error>>,

    /// Translation in x to be applied before rendering the glyph.
    tx: Twips,

    /// Translation in y to be applied before rendering the glyph.
    ty: Twips,
}

impl<'a> std::fmt::Debug for GlyphBitmap<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GlyphBitmap")
            .field("handle", &self.handle)
            .finish()
    }
}

impl<'a> GlyphBitmap<'a> {
    pub fn new(bitmap: Bitmap<'a>, tx: Twips, ty: Twips) -> Self {
        Self {
            bitmap: Cell::new(Some(bitmap)),
            handle: OnceCell::new(),
            tx,
            ty,
        }
    }

    pub fn get_bitmap_info_or_register(
        &self,
        renderer: &mut dyn RenderBackend,
    ) -> &Result<BitmapInfo, Error> {
        self.handle.get_or_init(|| {
            let bitmap = self
                .bitmap
                .take()
                .expect("Bitmap should be available before registering");
            let width = bitmap.width();
            let height = bitmap.height();
            let handle = renderer.register_bitmap(bitmap)?;
            Ok(BitmapInfo {
                handle,
                width,
                height,
            })
        })
    }
}

#[derive(Debug, Clone)]
pub struct Glyph {
    shape: GlyphShape,
    advance: Twips,

    // The character this glyph represents.
    character: char,
}

impl Glyph {
    /// Returns an empty glyph with zero advance.
    pub fn empty(character: char) -> Self {
        Self {
            shape: GlyphShape::None,
            advance: Twips::ZERO,
            character,
        }
    }

    pub fn whitespace(character: char, advance: Twips) -> Self {
        Self {
            shape: GlyphShape::None,
            advance,
            character,
        }
    }

    pub fn from_drawing(character: char, advance: Twips, drawing: Drawing) -> Self {
        Self {
            shape: GlyphShape::Drawing(Box::new(drawing)),
            advance,
            character,
        }
    }

    pub fn from_swf(character: char, swf_glyph: swf::Glyph) -> Self {
        Self {
            advance: Twips::new(swf_glyph.advance.into()),
            shape: GlyphShape::Swf(Box::new(RefCell::new(SwfGlyphOrShape::Glyph(swf_glyph)))),
            character,
        }
    }

    pub fn from_bitmap(
        character: char,
        bitmap: Bitmap<'static>,
        advance: Twips,
        tx: Twips,
        ty: Twips,
    ) -> Self {
        Self {
            shape: GlyphShape::Bitmap(Rc::new(GlyphBitmap::new(bitmap, tx, ty))),
            advance,
            character,
        }
    }

    pub fn glyph_render_data(&self, renderer: &mut dyn RenderBackend) -> Option<GlyphRenderData> {
        self.shape.register(renderer)
    }

    pub fn hit_test(&self, point: Point<Twips>, local_matrix: &Matrix) -> bool {
        self.shape.hit_test(point, local_matrix)
    }

    pub fn advance(&self) -> Twips {
        self.advance
    }

    pub fn character(&self) -> char {
        self.character
    }

    pub fn as_ref(&self) -> GlyphRef<'_> {
        GlyphRef::Direct(self)
    }

    pub fn rendered_at_baseline(&self) -> bool {
        match self.shape {
            GlyphShape::Swf(_) => true,
            GlyphShape::Drawing(_) => true,
            GlyphShape::Bitmap(_) => false,
            GlyphShape::None => false,
        }
    }

    pub fn renderable<'gc>(&self, context: &mut RenderContext<'_, 'gc>) -> bool {
        self.glyph_render_data(context.renderer).is_some()
    }

    pub fn render<'gc>(&self, context: &mut RenderContext<'_, 'gc>) {
        use ruffle_render::commands::CommandHandler;

        let Some(render_data) = self.glyph_render_data(context.renderer) else {
            return;
        };

        match render_data {
            GlyphRenderData::Shape(shape_handle) => {
                context
                    .commands
                    .render_shape(shape_handle, context.transform_stack.transform());
            }
            GlyphRenderData::Bitmap { info, tx, ty } => {
                context.transform_stack.push(&Transform {
                    matrix: Matrix::translate(tx, ty),
                    ..Default::default()
                });

                context.commands.render_bitmap(
                    info.handle,
                    context.transform_stack.transform(),
                    true,
                    ruffle_render::bitmap::PixelSnapping::Auto,
                );

                context.transform_stack.pop();
            }
        }
    }
}

pub enum GlyphRef<'a> {
    Direct(&'a Glyph),
    Ref(Ref<'a, Glyph>),
}

impl<'a> std::ops::Deref for GlyphRef<'a> {
    type Target = Glyph;

    fn deref(&self) -> &Self::Target {
        match self {
            GlyphRef::Direct(r) => r,
            GlyphRef::Ref(r) => r.deref(),
        }
    }
}
