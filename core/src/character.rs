pub enum Character<'gc> {
    Graphic(Box<crate::graphic::Graphic<'gc>>),
    MovieClip(Box<crate::movie_clip::MovieClip<'gc>>),
    Bitmap(crate::backend::render::BitmapHandle),
    Button(Box<crate::button::Button<'gc>>),
    Font(Box<crate::font::Font>),
    MorphShape(Box<crate::morph_shape::MorphShape<'gc>>),
    Text(Box<crate::text::Text<'gc>>),
    Sound(crate::backend::audio::SoundHandle),
}
