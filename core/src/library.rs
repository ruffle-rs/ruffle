use crate::backend::audio::SoundHandle;
use crate::character::Character;
use crate::display_object::DisplayObject;
use crate::font::Font;
use crate::prelude::*;
use gc_arena::{GcCell, MutationContext};
use std::collections::HashMap;
use swf::CharacterId;

pub struct Library<'gc> {
    characters: HashMap<CharacterId, Character<'gc>>,
    jpeg_tables: Option<Vec<u8>>,
    device_font: Box<Font>,
}

impl<'gc> Library<'gc> {
    pub fn new(device_font: Box<Font>) -> Self {
        Library {
            characters: HashMap::new(),
            jpeg_tables: None,
            device_font,
        }
    }

    pub fn register_character(&mut self, id: CharacterId, character: Character<'gc>) {
        // TODO(Herschel): What is the behavior if id already exists?
        if !self.contains_character(id) {
            self.characters.insert(id, character);
        } else {
            log::error!("Character ID collision: Tried to register ID {} twice", id);
        }
    }

    pub fn contains_character(&self, id: CharacterId) -> bool {
        self.characters.contains_key(&id)
    }

    pub fn get_character(&self, id: CharacterId) -> Option<&Character<'gc>> {
        self.characters.get(&id)
    }

    pub fn get_character_mut(&mut self, id: CharacterId) -> Option<&mut Character<'gc>> {
        self.characters.get_mut(&id)
    }

    pub fn instantiate_display_object(
        &self,
        id: CharacterId,
        gc_context: MutationContext<'gc, '_>,
    ) -> Result<DisplayNode<'gc>, Box<dyn std::error::Error>> {
        let obj: Box<dyn DisplayObject<'gc>> = match self.characters.get(&id) {
            Some(Character::EditText(edit_text)) => edit_text.clone(),
            Some(Character::Graphic(graphic)) => graphic.clone(),
            Some(Character::MorphShape(morph_shape)) => morph_shape.clone(),
            Some(Character::MovieClip(movie_clip)) => movie_clip.clone(),
            Some(Character::Button(button)) => button.clone(),
            Some(Character::Text(text)) => text.clone(),
            Some(_) => return Err("Not a DisplayObject".into()),
            None => {
                log::error!("Tried to instantiate non-registered character ID {}", id);
                return Err("Character id doesn't exist".into());
            }
        };
        let result = GcCell::allocate(gc_context, obj);
        result
            .write(gc_context)
            .post_instantiation(gc_context, result);
        Ok(result)
    }

    pub fn get_font(&self, id: CharacterId) -> Option<&Font> {
        if let Some(&Character::Font(ref font)) = self.characters.get(&id) {
            Some(font)
        } else {
            None
        }
    }

    pub fn get_sound(&self, id: CharacterId) -> Option<SoundHandle> {
        if let Some(Character::Sound(sound)) = self.characters.get(&id) {
            Some(*sound)
        } else {
            None
        }
    }

    pub fn set_jpeg_tables(&mut self, data: Vec<u8>) {
        let data = crate::backend::render::remove_invalid_jpeg_data(&data[..]).to_vec();
        self.jpeg_tables = Some(data);
    }

    pub fn jpeg_tables(&self) -> Option<&[u8]> {
        self.jpeg_tables.as_ref().map(|data| &data[..])
    }

    /// Returns the device font for use when a font is unavailable.
    pub fn device_font(&self) -> &Font {
        &*self.device_font
    }
}

unsafe impl<'gc> gc_arena::Collect for Library<'gc> {
    #[inline]
    fn trace(&self, cc: gc_arena::CollectionContext) {
        for character in self.characters.values() {
            character.trace(cc);
        }
    }
}
