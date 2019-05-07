pub enum Character {
    Graphic(Box<crate::graphic::Graphic>),
    MovieClip(Box<crate::movie_clip::MovieClip>),
    Bitmap(crate::backend::render::BitmapHandle),
    Button(Box<crate::button::Button>),
    Font(Box<crate::font::Font>),
    Text(Box<crate::text::Text>),
    Sound(crate::backend::audio::SoundHandle),
}
