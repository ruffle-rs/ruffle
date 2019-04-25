use web_sys::HtmlImageElement;

pub enum Character {
    Graphic {
        image: HtmlImageElement,
    },
    MovieClip {
        num_frames: u16,
        tag_stream: swf::read::Reader<std::io::Cursor<Vec<u8>>>,
    },
    Sound,
}
