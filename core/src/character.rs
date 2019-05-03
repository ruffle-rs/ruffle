pub enum Character {
    Graphic {
        x_min: f32,
        y_min: f32,
        shape_handle: crate::backend::render::ShapeHandle,
    },
    MovieClip {
        num_frames: u16,
        tag_stream_start: u64,
    },
    Bitmap(crate::backend::render::BitmapHandle),
    Button(Box<swf::Button>),
    Sound,
}
