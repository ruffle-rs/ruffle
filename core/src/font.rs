use crate::backend::render::{RenderBackend, ShapeHandle};

type Error = Box<dyn std::error::Error>;

pub struct Font {
    glyphs: Vec<ShapeHandle>,

    /// The scaling applied to the font height to render at the proper size.
    /// This depends on the DefineFont tag version.
    scale: f32,
}

impl Font {
    pub fn from_swf_tag(renderer: &mut dyn RenderBackend, tag: &swf::Font) -> Result<Font, Error> {
        let mut glyphs = vec![];
        for glyph in &tag.glyphs {
            let shape_handle = renderer.register_glyph_shape(glyph);
            glyphs.push(shape_handle);
        }
        Ok(Font {
            glyphs,

            /// DefineFont3 stores coordinates at 20x the scale of DefineFont1/2.
            /// (SWF19 p.164)
            scale: if tag.version >= 3 { 20480.0 } else { 1024.0 },
        })
    }

    pub fn get_glyph(&self, i: usize) -> Option<ShapeHandle> {
        self.glyphs.get(i).cloned()
    }

    #[inline]
    pub fn scale(&self) -> f32 {
        self.scale
    }
}
