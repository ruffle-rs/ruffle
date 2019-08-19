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

unsafe impl<'gc> gc_arena::Collect for Character<'gc> {
    #[inline]
    fn trace(&self, cc: gc_arena::CollectionContext) {
        match self {
            Character::Graphic(c) => c.trace(cc),
            Character::MovieClip(c) => c.trace(cc),
            Character::Bitmap(c) => c.trace(cc),
            Character::Button(c) => c.trace(cc),
            Character::Font(c) => c.trace(cc),
            Character::MorphShape(c) => c.trace(cc),
            Character::Text(c) => c.trace(cc),
            Character::Sound(c) => c.trace(cc),
        }
    }
}
