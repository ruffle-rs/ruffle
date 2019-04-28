pub enum Character {
    Graphic {
        x_min: f32,
        y_min: f32,
        shape_handle: crate::backend::render::common::ShapeHandle,
    },
    MovieClip {
        num_frames: u16,
        tag_stream_start: u64,
    },
    Sound,
}
