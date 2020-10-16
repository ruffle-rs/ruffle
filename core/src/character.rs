use crate::backend::audio::SoundHandle;
use crate::display_object::{
    Bitmap, Button, EditText, Graphic, MorphShape, MovieClip, Text, Video,
};
use crate::font::Font;

#[derive(Clone)]
pub enum Character<'gc> {
    EditText(EditText<'gc>),
    Graphic(Graphic<'gc>),
    MovieClip(MovieClip<'gc>),
    Bitmap(Bitmap<'gc>),
    Button(Button<'gc>),
    Font(Font<'gc>),
    MorphShape(MorphShape<'gc>),
    Text(Text<'gc>),
    Sound(SoundHandle),
    Video(Video<'gc>),
}

unsafe impl<'gc> gc_arena::Collect for Character<'gc> {
    #[inline]
    fn trace(&self, cc: gc_arena::CollectionContext) {
        match self {
            Character::EditText(c) => c.trace(cc),
            Character::Graphic(c) => c.trace(cc),
            Character::MovieClip(c) => c.trace(cc),
            Character::Bitmap(c) => c.trace(cc),
            Character::Button(c) => c.trace(cc),
            Character::Font(c) => c.trace(cc),
            Character::MorphShape(c) => c.trace(cc),
            Character::Text(c) => c.trace(cc),
            Character::Sound(c) => c.trace(cc),
            Character::Video(c) => c.trace(cc),
        }
    }
}
