use crate::backend::render::{RenderBackend, ShapeHandle};

type Error = Box<dyn std::error::Error>;

pub struct Font {
    glyphs: Vec<ShapeHandle>,
}

impl Font {
    pub fn from_swf_tag(renderer: &mut dyn RenderBackend, tag: &swf::Font) -> Result<Font, Error> {
        let mut glyphs = vec![];
        for glyph in &tag.glyphs {
            let shape_handle = renderer.register_glyph_shape(glyph);
            glyphs.push(shape_handle);
        }
        Ok(Font { glyphs })
    }

    pub fn get_glyph(&self, i: usize) -> Option<ShapeHandle> {
        self.glyphs.get(i).cloned()
    }
}
