use crate::avm1::globals::SystemPrototypes;
use crate::avm1::Object;
use crate::backend::audio::SoundHandle;
use crate::character::Character;
use crate::display_object::TDisplayObject;
use crate::font::Font;
use crate::prelude::*;
use gc_arena::MutationContext;
use std::collections::HashMap;
use swf::CharacterId;

pub struct Library<'gc> {
    characters: HashMap<CharacterId, Character<'gc>>,
    jpeg_tables: Option<Vec<u8>>,
    device_font: Option<Box<Font>>,
}

impl<'gc> Library<'gc> {
    pub fn new() -> Self {
        Library {
            characters: HashMap::new(),
            jpeg_tables: None,
            device_font: None,
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

    #[allow(dead_code)]
    pub fn get_character(&self, id: CharacterId) -> Option<&Character<'gc>> {
        self.characters.get(&id)
    }

    #[allow(dead_code)]
    pub fn get_character_mut(&mut self, id: CharacterId) -> Option<&mut Character<'gc>> {
        self.characters.get_mut(&id)
    }

    pub fn instantiate_display_object(
        &self,
        id: CharacterId,
        gc_context: MutationContext<'gc, '_>,
        prototypes: &SystemPrototypes<'gc>,
    ) -> Result<DisplayObject<'gc>, Box<dyn std::error::Error>> {
        let (mut obj, proto): (DisplayObject<'gc>, Object<'gc>) = match self.characters.get(&id) {
            Some(Character::Bitmap(bitmap)) => (bitmap.instantiate(gc_context), prototypes.object),
            Some(Character::EditText(edit_text)) => {
                (edit_text.instantiate(gc_context), prototypes.object)
            }
            Some(Character::Graphic(graphic)) => {
                (graphic.instantiate(gc_context), prototypes.object)
            }
            Some(Character::MorphShape(morph_shape)) => {
                (morph_shape.instantiate(gc_context), prototypes.object)
            }
            Some(Character::MovieClip(movie_clip)) => {
                (movie_clip.instantiate(gc_context), prototypes.movie_clip)
            }
            Some(Character::Button(button)) => (button.instantiate(gc_context), prototypes.object),
            Some(Character::Text(text)) => (text.instantiate(gc_context), prototypes.object),
            Some(_) => return Err("Not a DisplayObject".into()),
            None => {
                log::error!("Tried to instantiate non-registered character ID {}", id);
                return Err("Character id doesn't exist".into());
            }
        };
        obj.post_instantiation(gc_context, obj, proto);
        Ok(obj)
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
        if self.jpeg_tables.is_some() {
            // SWF spec says there should only be one JPEGTables tag.
            // TODO: What is the behavior when there are multiples?
            log::warn!("SWF contains multiple JPEGTables tags");
            return;
        }
        // Some SWFs have a JPEGTables tag with 0 length; ignore these.
        // (Does this happen when there is only a single DefineBits tag?)
        self.jpeg_tables = if data.is_empty() {
            None
        } else {
            Some(crate::backend::render::remove_invalid_jpeg_data(&data[..]).to_vec())
        }
    }

    pub fn jpeg_tables(&self) -> Option<&[u8]> {
        self.jpeg_tables.as_ref().map(|data| &data[..])
    }

    /// Returns the device font for use when a font is unavailable.
    pub fn device_font(&self) -> Option<&Font> {
        self.device_font.as_ref().map(AsRef::as_ref)
    }

    /// Sets the device font.
    pub fn set_device_font(&mut self, font: Option<Box<Font>>) {
        self.device_font = font;
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

impl Default for Library<'_> {
    fn default() -> Self {
        Self::new()
    }
}
