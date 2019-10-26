use crate::backend::audio::SoundHandle;
use crate::display_object::{Bitmap, Button, EditText, Graphic, MorphShape, MovieClip, Text};
use crate::font::Font;

pub enum Character<'gc> {
    EditText(Box<EditText<'gc>>),
    Graphic(Box<Graphic<'gc>>),
    MovieClip(Box<MovieClip<'gc>>),
    Bitmap(Box<Bitmap<'gc>>),
    Button(Box<Button<'gc>>),
    Font(Box<Font>),
    MorphShape(Box<MorphShape<'gc>>),
    Text(Box<Text<'gc>>),
    Sound(SoundHandle),
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
        }
    }
}
