use crate::backend::audio::SoundHandle;
use crate::character::Character;
use crate::display_object::TDisplayObject;
use crate::font::{Font, FontDescriptor};
use crate::prelude::*;
use crate::tag_utils::{SwfMovie, SwfSlice};
use crate::vminterface::AvmType;
use gc_arena::{Collect, MutationContext};
use std::collections::HashMap;
use std::sync::{Arc, Weak};
use swf::{CharacterId, TagCode};
use weak_table::PtrWeakKeyHashMap;

/// Boxed error alias.
type Error = Box<dyn std::error::Error>;

/// Symbol library for a single given SWF.
#[derive(Collect)]
#[collect(no_drop)]
pub struct MovieLibrary<'gc> {
    characters: HashMap<CharacterId, Character<'gc>>,
    export_characters: HashMap<String, Character<'gc>>,
    jpeg_tables: Option<Vec<u8>>,
    device_font: Option<Font<'gc>>,
    fonts: HashMap<FontDescriptor, Font<'gc>>,
    avm_type: AvmType,
}

impl<'gc> MovieLibrary<'gc> {
    pub fn new(avm_type: AvmType) -> Self {
        MovieLibrary {
            characters: HashMap::new(),
            export_characters: HashMap::new(),
            jpeg_tables: None,
            device_font: None,
            fonts: HashMap::new(),
            avm_type,
        }
    }

    pub fn register_character(&mut self, id: CharacterId, character: Character<'gc>) {
        // TODO(Herschel): What is the behavior if id already exists?
        if !self.contains_character(id) {
            if let Character::Font(font) = character.clone() {
                self.fonts.insert(font.descriptor(), font);
            }

            self.characters.insert(id, character);
        } else {
            log::error!("Character ID collision: Tried to register ID {} twice", id);
        }
    }

    /// Registers an export name for a given character ID.
    /// This character will then be instantiable from AVM1.
    pub fn register_export(&mut self, id: CharacterId, export_name: &str) {
        use std::collections::hash_map::Entry;
        if let Some(character) = self.characters.get(&id) {
            match self.export_characters.entry(export_name.to_string()) {
                Entry::Vacant(e) => {
                    e.insert(character.clone());
                }
                Entry::Occupied(_) => {
                    log::warn!(
                        "Can't register export {}: Export already exists",
                        export_name
                    );
                }
            }
        } else {
            log::warn!(
                "Can't register export {}: Character ID {} doesn't exist",
                export_name,
                id
            )
        }
    }

    pub fn contains_character(&self, id: CharacterId) -> bool {
        self.characters.contains_key(&id)
    }

    #[allow(dead_code)]
    pub fn get_character_by_id(&self, id: CharacterId) -> Option<&Character<'gc>> {
        self.characters.get(&id)
    }

    #[allow(dead_code)]
    pub fn get_character_by_export_name(&self, name: &str) -> Option<&Character<'gc>> {
        self.export_characters.get(name)
    }

    /// Instantiates the library item with the given character ID into a display object.
    /// The object must then be post-instantiated before being used.
    pub fn instantiate_by_id(
        &self,
        id: CharacterId,
        gc_context: MutationContext<'gc, '_>,
    ) -> Result<DisplayObject<'gc>, Box<dyn std::error::Error>> {
        if let Some(character) = self.characters.get(&id) {
            self.instantiate_display_object(character, gc_context)
        } else {
            log::error!("Tried to instantiate non-registered character ID {}", id);
            Err("Character id doesn't exist".into())
        }
    }

    /// Instantiates the library item with the given export name into a display object.
    /// The object must then be post-instantiated before being used.
    pub fn instantiate_by_export_name(
        &self,
        export_name: &str,
        gc_context: MutationContext<'gc, '_>,
    ) -> Result<DisplayObject<'gc>, Box<dyn std::error::Error>> {
        if let Some(character) = self.export_characters.get(export_name) {
            self.instantiate_display_object(character, gc_context)
        } else {
            log::error!(
                "Tried to instantiate non-registered character {}",
                export_name
            );
            Err("Character id doesn't exist".into())
        }
    }

    /// Instantiates the given character into a display object.
    /// The object must then be post-instantiated before being used.
    fn instantiate_display_object(
        &self,
        character: &Character<'gc>,
        gc_context: MutationContext<'gc, '_>,
    ) -> Result<DisplayObject<'gc>, Box<dyn std::error::Error>> {
        match character {
            Character::Bitmap(bitmap) => Ok(bitmap.instantiate(gc_context)),
            Character::EditText(edit_text) => Ok(edit_text.instantiate(gc_context)),
            Character::Graphic(graphic) => Ok(graphic.instantiate(gc_context)),
            Character::MorphShape(morph_shape) => Ok(morph_shape.instantiate(gc_context)),
            Character::MovieClip(movie_clip) => Ok(movie_clip.instantiate(gc_context)),
            Character::Button(button) => Ok(button.instantiate(gc_context)),
            Character::Text(text) => Ok(text.instantiate(gc_context)),
            _ => Err("Not a DisplayObject".into()),
        }
    }

    pub fn get_font(&self, id: CharacterId) -> Option<Font<'gc>> {
        if let Some(&Character::Font(font)) = self.characters.get(&id) {
            Some(font)
        } else {
            None
        }
    }

    /// Find a font by it's name and parameters.
    pub fn get_font_by_name(
        &self,
        name: &str,
        is_bold: bool,
        is_italic: bool,
    ) -> Option<Font<'gc>> {
        let descriptor = FontDescriptor::from_parts(name, is_bold, is_italic);

        self.fonts.get(&descriptor).copied()
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
    pub fn device_font(&self) -> Option<Font<'gc>> {
        self.device_font
    }

    /// Sets the device font.
    pub fn set_device_font(&mut self, font: Option<Font<'gc>>) {
        self.device_font = font;
    }

    /// Check if the current movie's VM type is compatible with running code on
    /// a particular VM. If it is not, then this yields an error.
    pub fn check_avm_type(&mut self, new_type: AvmType) -> Result<(), Error> {
        if self.avm_type != new_type {
            return Err(format!(
                "Blocked attempt to run {:?} code on an {:?} movie.",
                new_type, self.avm_type
            )
            .into());
        }

        self.avm_type = new_type;

        Ok(())
    }

    /// Get the VM type of this movie.
    pub fn avm_type(&self) -> AvmType {
        self.avm_type
    }
}

/// Symbol library for multiple movies.
pub struct Library<'gc> {
    /// All the movie libraries.
    movie_libraries: PtrWeakKeyHashMap<Weak<SwfMovie>, MovieLibrary<'gc>>,
}

unsafe impl<'gc> gc_arena::Collect for Library<'gc> {
    #[inline]
    fn trace(&self, cc: gc_arena::CollectionContext) {
        for (_, val) in self.movie_libraries.iter() {
            val.trace(cc);
        }
    }
}

impl<'gc> Library<'gc> {
    pub fn library_for_movie(&self, movie: Arc<SwfMovie>) -> Option<&MovieLibrary<'gc>> {
        self.movie_libraries.get(&movie)
    }

    pub fn library_for_movie_mut(&mut self, movie: Arc<SwfMovie>) -> &mut MovieLibrary<'gc> {
        if !self.movie_libraries.contains_key(&movie) {
            let slice = SwfSlice::from(movie.clone());
            let mut reader = slice.read_from(0);
            let vm_type = if movie.header().version > 8 {
                match reader.read_tag_code_and_length() {
                    Ok((tag_code, _tag_len))
                        if TagCode::from_u16(tag_code) == Some(TagCode::FileAttributes) =>
                    {
                        match reader.read_file_attributes() {
                            Ok(attributes) if attributes.is_action_script_3 => AvmType::Avm2,
                            Ok(_) => AvmType::Avm1,
                            Err(e) => {
                                log::error!("Got {} when reading FileAttributes", e);
                                AvmType::Avm1
                            }
                        }
                    }
                    // SWF defaults to AVM1 if FileAttributes is not the first tag.
                    _ => AvmType::Avm1,
                }
            } else {
                AvmType::Avm1
            };

            self.movie_libraries
                .insert(movie.clone(), MovieLibrary::new(vm_type));
        };

        self.movie_libraries.get_mut(&movie).unwrap()
    }
}

impl<'gc> Default for Library<'gc> {
    fn default() -> Self {
        Self {
            movie_libraries: PtrWeakKeyHashMap::new(),
        }
    }
}
