pub enum Character<'gc> {
    Graphic(Box<crate::graphic::Graphic>),
    MovieClip(Box<crate::movie_clip::MovieClip<'gc>>),
    Bitmap(crate::backend::render::BitmapHandle),
    Button(Box<crate::button::Button<'gc>>),
    Font(Box<crate::font::Font>),
    MorphShape(Box<crate::morph_shape::MorphShape>),
    Text(Box<crate::text::Text>),
    Sound(crate::backend::audio::SoundHandle),
}
