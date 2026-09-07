use crate::context::RenderContext;
use crate::drawing::Drawing;
use crate::font::FontAtlasGlyph;
use crate::prelude::*;
use ruffle_render::backend::null::NullBitmapSource;
use ruffle_render::backend::{RenderBackend, ShapeHandle};
use ruffle_render::transform::Transform;
use std::cell::{Ref, RefCell};

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
    AtlasGlyph(FontAtlasGlyph),
}

impl GlyphRenderData {
    pub fn from_shape(shape_handle: ShapeHandle) -> Self {
        Self::Shape(shape_handle)
    }

    pub fn from_atlas(atlas_glyph: FontAtlasGlyph) -> Self {
        Self::AtlasGlyph(atlas_glyph)
    }
}

#[derive(Debug, Clone)]
enum GlyphShape {
    Swf(Box<RefCell<SwfGlyphOrShape>>),
    Drawing(Box<Drawing>),
    AtlasGlyph(FontAtlasGlyph),
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
            GlyphShape::AtlasGlyph(_) => {
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
            GlyphShape::AtlasGlyph(atlas_glyph) => atlas_glyph
                .atlas_handle(renderer)
                .as_ref()
                .map(|_| GlyphRenderData::from_atlas(atlas_glyph.clone())),
            GlyphShape::None => None,
        }
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

    pub fn from_atlas(character: char, atlas_glyph: FontAtlasGlyph, advance: Twips) -> Self {
        Self {
            shape: GlyphShape::AtlasGlyph(atlas_glyph),
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
            GlyphShape::AtlasGlyph(_) => false,
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
            GlyphRenderData::AtlasGlyph(atlas_glyph) => {
                let handle = atlas_glyph.atlas_handle(context.renderer);
                let Some(handle) = handle else {
                    return;
                };

                context.transform_stack.push(&Transform {
                    matrix: Matrix::translate(atlas_glyph.tx(), atlas_glyph.ty()),
                    ..Default::default()
                });

                context.commands.render_bitmap(
                    handle,
                    context.transform_stack.transform(),
                    true,
                    ruffle_render::bitmap::PixelSnapping::Auto,
                    atlas_glyph.atlas_region(),
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
