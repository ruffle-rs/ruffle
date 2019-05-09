use crate::backend::render::ShapeHandle;
use crate::player::{RenderContext, UpdateContext};
use crate::prelude::*;

type Error = Box<std::error::Error>;

pub struct Font {
    id: CharacterId,
    glyphs: Vec<ShapeHandle>,
}

impl Font {
    pub fn from_swf_tag(context: &mut UpdateContext, tag: &swf::Font) -> Result<Font, Error> {
        let mut glyphs = vec![];
        for glyph in &tag.glyphs {
            let shape_handle = context.renderer.register_glyph_shape(glyph);
            glyphs.push(shape_handle);
        }
        Ok(Font { id: tag.id, glyphs })
    }

    pub fn get_glyph(&self, i: usize) -> Option<ShapeHandle> {
        self.glyphs.get(i).cloned()
    }
}
