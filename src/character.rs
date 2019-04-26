use web_sys::HtmlImageElement;

pub enum Character {
    Graphic {
        image: HtmlImageElement,
        x_min: f32,
        y_min: f32,
    },
    MovieClip {
        num_frames: u16,
        tag_stream_start: u64,
    },
    Sound,
}
