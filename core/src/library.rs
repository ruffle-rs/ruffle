use crate::avm1::function::FunctionObject;
use crate::avm1::property_map::PropertyMap as Avm1PropertyMap;
use crate::avm2::{ClassObject as Avm2ClassObject, Domain as Avm2Domain};
use crate::backend::{audio::SoundHandle, render};
use crate::character::Character;
use crate::display_object::{Bitmap, Graphic, MorphShape, TDisplayObject, Text};
use crate::font::{Font, FontDescriptor};
use crate::prelude::*;
use crate::tag_utils::SwfMovie;
use crate::vminterface::AvmType;
use gc_arena::{Collect, Gc, GcCell, MutationContext};
use std::collections::HashMap;
use std::sync::{Arc, Weak};
use swf::CharacterId;
use weak_table::{traits::WeakElement, PtrWeakKeyHashMap, WeakValueHashMap};

/// The mappings between symbol names and constructors registered
/// with `Object.registerClass`.
#[derive(Collect)]
#[collect(no_drop)]
pub struct Avm1ConstructorRegistry<'gc> {
    symbol_map: GcCell<'gc, Avm1PropertyMap<FunctionObject<'gc>>>,
    is_case_sensitive: bool,
}

impl<'gc> Avm1ConstructorRegistry<'gc> {
    pub fn new(is_case_sensitive: bool, gc_context: MutationContext<'gc, '_>) -> Self {
        Self {
            symbol_map: GcCell::allocate(gc_context, Avm1PropertyMap::new()),
            is_case_sensitive,
        }
    }

    pub fn get(&self, symbol: &str) -> Option<FunctionObject<'gc>> {
        self.symbol_map
            .read()
            .get(symbol, self.is_case_sensitive)
            .copied()
    }

    pub fn set(
        &self,
        symbol: &str,
        constructor: Option<FunctionObject<'gc>>,
        gc_context: MutationContext<'gc, '_>,
    ) {
        let mut map = self.symbol_map.write(gc_context);
        if let Some(ctor) = constructor {
            map.insert(symbol, ctor, self.is_case_sensitive);
        } else {
            map.remove(symbol, self.is_case_sensitive);
        };
    }
}

#[derive(Clone)]
struct MovieSymbol(Arc<SwfMovie>, CharacterId);

#[derive(Clone)]
struct WeakMovieSymbol(Weak<SwfMovie>, CharacterId);

impl WeakElement for WeakMovieSymbol {
    type Strong = MovieSymbol;

    fn new(view: &Self::Strong) -> Self {
        Self(Arc::downgrade(&view.0), view.1)
    }

    fn view(&self) -> Option<Self::Strong> {
        if let Some(strong) = self.0.upgrade() {
            Some(MovieSymbol(strong, self.1))
        } else {
            None
        }
    }
}

/// The mappings between class objects and library characters defined by
/// `SymbolClass`.
pub struct Avm2ClassRegistry<'gc> {
    /// A list of AVM2 class objects and the character IDs they are expected to
    /// instantiate.
    class_map: WeakValueHashMap<Avm2ClassObject<'gc>, WeakMovieSymbol>,
}

unsafe impl Collect for Avm2ClassRegistry<'_> {
    fn trace(&self, cc: gc_arena::CollectionContext) {
        for (k, _) in self.class_map.iter() {
            k.trace(cc);
        }
    }
}

impl Default for Avm2ClassRegistry<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'gc> Avm2ClassRegistry<'gc> {
    pub fn new() -> Self {
        Self {
            class_map: WeakValueHashMap::new(),
        }
    }

    /// Retrieve the library symbol for a given AVM2 class object.
    ///
    /// A value of `None` indicates that this AVM2 class is not associated with
    /// a library symbol.
    pub fn class_symbol(
        &self,
        class_object: Avm2ClassObject<'gc>,
    ) -> Option<(Arc<SwfMovie>, CharacterId)> {
        match self.class_map.get(&class_object) {
            Some(MovieSymbol(movie, symbol)) => Some((movie, symbol)),
            None => None,
        }
    }

    /// Associate an AVM2 class object with a given library symbol.
    pub fn set_class_symbol(
        &mut self,
        class_object: Avm2ClassObject<'gc>,
        movie: Arc<SwfMovie>,
        symbol: CharacterId,
    ) {
        self.class_map
            .insert(class_object, MovieSymbol(movie, symbol));
    }
}

/// Symbol library for a single given SWF.
#[derive(Collect)]
#[collect(no_drop)]
pub struct MovieLibrary<'gc> {
    characters: HashMap<CharacterId, Character<'gc>>,
    export_characters: Avm1PropertyMap<Character<'gc>>,
    jpeg_tables: Option<Vec<u8>>,
    fonts: HashMap<FontDescriptor, Font<'gc>>,
    avm_type: AvmType,
    avm2_domain: Option<Avm2Domain<'gc>>,

    /// Shared reference to the constructor registry used for this movie.
    /// Should be `None` if this is an AVM2 movie.
    avm1_constructor_registry: Option<Gc<'gc, Avm1ConstructorRegistry<'gc>>>,
}

impl<'gc> MovieLibrary<'gc> {
    pub fn new(avm_type: AvmType) -> Self {
        MovieLibrary {
            characters: HashMap::new(),
            export_characters: Avm1PropertyMap::new(),
            jpeg_tables: None,
            fonts: HashMap::new(),
            avm_type,
            avm2_domain: None,
            avm1_constructor_registry: None,
        }
    }

    pub fn register_character(&mut self, id: CharacterId, character: Character<'gc>) {
        // TODO(Herschel): What is the behavior if id already exists?
        if !self.contains_character(id) {
            if let Character::Font(font) = character.clone() {
                self.fonts.insert(font.descriptor().clone(), font);
            }

            self.characters.insert(id, character);
        } else {
            log::error!("Character ID collision: Tried to register ID {} twice", id);
        }
    }

    /// Registers an export name for a given character ID.
    /// This character will then be instantiable from AVM1.
    pub fn register_export(
        &mut self,
        id: CharacterId,
        export_name: &str,
    ) -> Option<&Character<'gc>> {
        if let Some(character) = self.characters.get(&id) {
            self.export_characters
                .insert(export_name, character.clone(), false);
            Some(character)
        } else {
            log::warn!(
                "Can't register export {}: Character ID {} doesn't exist",
                export_name,
                id,
            );
            None
        }
    }

    pub fn contains_character(&self, id: CharacterId) -> bool {
        self.characters.contains_key(&id)
    }

    pub fn character_by_id(&self, id: CharacterId) -> Option<&Character<'gc>> {
        self.characters.get(&id)
    }

    pub fn character_by_export_name(&self, name: &str) -> Option<&Character<'gc>> {
        self.export_characters.get(name, false)
    }

    pub fn avm1_constructor_registry(&self) -> Option<Gc<'gc, Avm1ConstructorRegistry<'gc>>> {
        self.avm1_constructor_registry
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
        if let Some(character) = self.export_characters.get(export_name, false) {
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
            Character::Avm1Button(button) => Ok(button.instantiate(gc_context)),
            Character::Avm2Button(button) => Ok(button.instantiate(gc_context)),
            Character::Text(text) => Ok(text.instantiate(gc_context)),
            Character::Video(video) => Ok(video.instantiate(gc_context)),
            _ => Err("Not a DisplayObject".into()),
        }
    }

    pub fn get_bitmap(&self, id: CharacterId) -> Option<Bitmap<'gc>> {
        if let Some(&Character::Bitmap(bitmap)) = self.characters.get(&id) {
            Some(bitmap)
        } else {
            None
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

    /// Returns the `Graphic` with the given character ID.
    /// Returns `None` if the ID does not exist or is not a `Graphic`.
    pub fn get_graphic(&self, id: CharacterId) -> Option<Graphic<'gc>> {
        if let Some(&Character::Graphic(graphic)) = self.characters.get(&id) {
            Some(graphic)
        } else {
            None
        }
    }

    /// Returns the `MorphShape` with the given character ID.
    /// Returns `None` if the ID does not exist or is not a `MorphShape`.
    pub fn get_morph_shape(&self, id: CharacterId) -> Option<MorphShape<'gc>> {
        if let Some(&Character::MorphShape(morph_shape)) = self.characters.get(&id) {
            Some(morph_shape)
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

    /// Returns the `Text` with the given character ID.
    /// Returns `None` if the ID does not exist or is not a `Text`.
    pub fn get_text(&self, id: CharacterId) -> Option<Text<'gc>> {
        if let Some(&Character::Text(text)) = self.characters.get(&id) {
            Some(text)
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
            Some(render::remove_invalid_jpeg_data(&data[..]).to_vec())
        }
    }

    pub fn jpeg_tables(&self) -> Option<&[u8]> {
        self.jpeg_tables.as_ref().map(|data| &data[..])
    }

    /// Get the VM type of this movie.
    pub fn avm_type(&self) -> AvmType {
        self.avm_type
    }

    /// Forcibly set the AVM type of this movie.
    ///
    /// This is intended for display object types which can be created
    /// dynamically but need a placeholder movie. You should *not* attempt to
    /// change the AVM type of an actual SWF.
    pub fn force_avm_type(&mut self, new_type: AvmType) {
        self.avm_type = new_type;
    }

    pub fn set_avm2_domain(&mut self, avm2_domain: Avm2Domain<'gc>) {
        self.avm2_domain = Some(avm2_domain);
    }

    /// Get the AVM2 domain this movie runs under.
    ///
    /// Note that the presence of an AVM2 domain does *not* indicate that this
    /// movie provides AVM2 code. For example, a movie may have been loaded by
    /// AVM2 code into a particular domain, even though it turned out to be
    /// an AVM1 movie, and thus this domain is unused.
    pub fn avm2_domain(&self) -> Avm2Domain<'gc> {
        self.avm2_domain.unwrap()
    }
}

impl<'gc> render::BitmapSource for MovieLibrary<'gc> {
    fn bitmap(&self, id: u16) -> Option<render::BitmapInfo> {
        self.get_bitmap(id).and_then(|bitmap| {
            Some(render::BitmapInfo {
                handle: bitmap.bitmap_handle()?,
                width: bitmap.width(),
                height: bitmap.height(),
            })
        })
    }
}

/// Symbol library for multiple movies.
pub struct Library<'gc> {
    /// All the movie libraries.
    movie_libraries: PtrWeakKeyHashMap<Weak<SwfMovie>, MovieLibrary<'gc>>,

    /// The embedded device font.
    device_font: Option<Font<'gc>>,

    constructor_registry_case_insensitive: Gc<'gc, Avm1ConstructorRegistry<'gc>>,
    constructor_registry_case_sensitive: Gc<'gc, Avm1ConstructorRegistry<'gc>>,

    /// A list of the symbols associated with specific AVM2 constructor
    /// prototypes.
    avm2_class_registry: Avm2ClassRegistry<'gc>,
}

unsafe impl<'gc> gc_arena::Collect for Library<'gc> {
    #[inline]
    fn trace(&self, cc: gc_arena::CollectionContext) {
        for (_, val) in self.movie_libraries.iter() {
            val.trace(cc);
        }
        self.device_font.trace(cc);
        self.constructor_registry_case_insensitive.trace(cc);
        self.constructor_registry_case_sensitive.trace(cc);
        self.avm2_class_registry.trace(cc);
    }
}

impl<'gc> Library<'gc> {
    pub fn empty(gc_context: MutationContext<'gc, '_>) -> Self {
        Self {
            movie_libraries: PtrWeakKeyHashMap::new(),
            device_font: None,
            constructor_registry_case_insensitive: Gc::allocate(
                gc_context,
                Avm1ConstructorRegistry::new(false, gc_context),
            ),
            constructor_registry_case_sensitive: Gc::allocate(
                gc_context,
                Avm1ConstructorRegistry::new(true, gc_context),
            ),
            avm2_class_registry: Default::default(),
        }
    }

    pub fn library_for_movie(&self, movie: Arc<SwfMovie>) -> Option<&MovieLibrary<'gc>> {
        self.movie_libraries.get(&movie)
    }

    pub fn library_for_movie_mut(&mut self, movie: Arc<SwfMovie>) -> &mut MovieLibrary<'gc> {
        if !self.movie_libraries.contains_key(&movie) {
            let avm_type = movie.avm_type();
            let mut movie_library = MovieLibrary::new(avm_type);
            if avm_type == AvmType::Avm1 {
                movie_library.avm1_constructor_registry =
                    Some(self.get_avm1_constructor_registry(movie.version()));
            }

            self.movie_libraries.insert(movie.clone(), movie_library);
        };

        self.movie_libraries.get_mut(&movie).unwrap()
    }

    /// Returns the device font for use when a font is unavailable.
    pub fn device_font(&self) -> Option<Font<'gc>> {
        self.device_font
    }

    /// Sets the device font.
    pub fn set_device_font(&mut self, font: Option<Font<'gc>>) {
        self.device_font = font;
    }

    /// Gets the constructor registry to use for the given SWF version.
    /// Because SWFs v6 and v7+ use different case-sensitivity rules, Flash
    /// keeps two separate registries, one case-sensitive, the other not.
    fn get_avm1_constructor_registry(
        &mut self,
        swf_version: u8,
    ) -> Gc<'gc, Avm1ConstructorRegistry<'gc>> {
        if swf_version < 7 {
            self.constructor_registry_case_insensitive
        } else {
            self.constructor_registry_case_sensitive
        }
    }

    /// Get the AVM2 class registry.
    pub fn avm2_class_registry(&self) -> &Avm2ClassRegistry<'gc> {
        &self.avm2_class_registry
    }

    /// Mutate the AVM2 class registry.
    pub fn avm2_class_registry_mut(&mut self) -> &mut Avm2ClassRegistry<'gc> {
        &mut self.avm2_class_registry
    }
}
