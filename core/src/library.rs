use crate::avm1::{PropertyMap as Avm1PropertyMap, PropertyMap};
use crate::avm2::{ClassObject as Avm2ClassObject, Domain as Avm2Domain};
use crate::backend::audio::SoundHandle;
use crate::character::Character;

use crate::display_object::{Bitmap, Graphic, MorphShape, TDisplayObject, Text};
use crate::font::{Font, FontDescriptor, FontType};
use crate::prelude::*;
use crate::string::AvmString;
use crate::tag_utils::SwfMovie;
use gc_arena::{Collect, Mutation};
use ruffle_render::backend::RenderBackend;
use ruffle_render::bitmap::BitmapHandle;
use ruffle_render::utils::remove_invalid_jpeg_data;

use crate::backend::ui::{FontDefinition, UiBackend};
use crate::DefaultFont;
use fnv::{FnvHashMap, FnvHashSet};
use std::collections::HashMap;
use std::sync::{Arc, Weak};
use swf::CharacterId;
use weak_table::{traits::WeakElement, PtrWeakKeyHashMap, WeakValueHashMap};

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
    fn trace(&self, cc: &gc_arena::Collection) {
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
        if let Some(old) = self.class_map.get(&class_object) {
            if Arc::ptr_eq(&movie, &old.0) && symbol != old.1 {
                // Flash player actually allows using the same class in multiple SymbolClass
                // entires in the same swf, with *different* symbol ids. Whichever one
                // is processed first will *win*, and the second one will be ignored.
                // We still log a warning, since we wouldn't expect this to happen outside
                // of deliberately crafted SWFs.
                tracing::warn!(
                    "Tried to overwrite class {:?} id={:?} with symbol id={:?} from same movie",
                    class_object,
                    old.1,
                    symbol,
                );
            }
            // If we're trying to overwrite the class with a symbol from a *different* SwfMovie,
            // then just ignore it. This handles the case where a Loader has a class that shadows
            // a class in the main swf (possibly with a different ApplicationDomain). This will
            // result in the original class from the parent being used, even when the child swf
            // instantiates the clip on the timeline.
            return;
        }
        self.class_map
            .insert(class_object, MovieSymbol(movie, symbol));
    }
}

/// Symbol library for a single given SWF.
#[derive(Collect)]
#[collect(no_drop)]
pub struct MovieLibrary<'gc> {
    characters: HashMap<CharacterId, Character<'gc>>,
    export_characters: Avm1PropertyMap<'gc, CharacterId>,
    jpeg_tables: Option<Vec<u8>>,
    fonts: FontMap<'gc>,
    avm2_domain: Option<Avm2Domain<'gc>>,
}

impl<'gc> MovieLibrary<'gc> {
    pub fn new() -> Self {
        Self {
            characters: HashMap::new(),
            export_characters: Avm1PropertyMap::new(),
            jpeg_tables: None,
            fonts: Default::default(),
            avm2_domain: None,
        }
    }

    pub fn register_character(&mut self, id: CharacterId, character: Character<'gc>) {
        // TODO(Herschel): What is the behavior if id already exists?
        if !self.contains_character(id) {
            if let Character::Font(font) = character {
                self.fonts.register(font);
            }

            self.characters.insert(id, character);
        } else {
            tracing::error!("Character ID collision: Tried to register ID {} twice", id);
        }
    }

    /// Registers an export name for a given character ID.
    /// This character will then be instantiable from AVM1.
    pub fn register_export(&mut self, id: CharacterId, export_name: AvmString<'gc>) {
        self.export_characters.insert(export_name, id, false);
    }

    #[allow(dead_code)]
    pub fn characters(&self) -> &HashMap<CharacterId, Character<'gc>> {
        &self.characters
    }

    #[allow(dead_code)]
    pub fn export_characters(&self) -> &PropertyMap<'gc, CharacterId> {
        &self.export_characters
    }

    pub fn contains_character(&self, id: CharacterId) -> bool {
        self.characters.contains_key(&id)
    }

    pub fn character_by_id(&self, id: CharacterId) -> Option<&Character<'gc>> {
        self.characters.get(&id)
    }

    pub fn character_by_export_name(&self, name: AvmString<'gc>) -> Option<&Character<'gc>> {
        if let Some(id) = self.export_characters.get(name, false) {
            return self.characters.get(id);
        }
        None
    }

    /// Instantiates the library item with the given character ID into a display object.
    /// The object must then be post-instantiated before being used.
    pub fn instantiate_by_id(
        &self,
        id: CharacterId,
        gc_context: &Mutation<'gc>,
    ) -> Result<DisplayObject<'gc>, &'static str> {
        if let Some(character) = self.characters.get(&id) {
            self.instantiate_display_object(character, gc_context)
        } else {
            tracing::error!("Tried to instantiate non-registered character ID {}", id);
            Err("Character id doesn't exist")
        }
    }

    /// Instantiates the library item with the given export name into a display object.
    /// The object must then be post-instantiated before being used.
    pub fn instantiate_by_export_name(
        &self,
        export_name: AvmString<'gc>,
        gc_context: &Mutation<'gc>,
    ) -> Result<DisplayObject<'gc>, &'static str> {
        if let Some(character) = self.character_by_export_name(export_name) {
            self.instantiate_display_object(character, gc_context)
        } else {
            tracing::error!(
                "Tried to instantiate non-registered character {}",
                export_name
            );
            Err("Character id doesn't exist")
        }
    }

    /// Instantiates the given character into a display object.
    /// The object must then be post-instantiated before being used.
    fn instantiate_display_object(
        &self,
        character: &Character<'gc>,
        gc_context: &Mutation<'gc>,
    ) -> Result<DisplayObject<'gc>, &'static str> {
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
            _ => Err("Not a DisplayObject"),
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

    pub fn embedded_fonts(&self) -> Vec<Font<'gc>> {
        self.fonts.all()
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

    pub fn set_jpeg_tables(&mut self, data: &[u8]) {
        if self.jpeg_tables.is_some() {
            // SWF spec says there should only be one JPEGTables tag.
            // TODO: What is the behavior when there are multiples?
            tracing::warn!("SWF contains multiple JPEGTables tags");
            return;
        }
        // Some SWFs have a JPEGTables tag with 0 length; ignore these.
        // (Does this happen when there is only a single DefineBits tag?)
        self.jpeg_tables = if data.is_empty() {
            None
        } else {
            Some(remove_invalid_jpeg_data(data).to_vec())
        }
    }

    pub fn jpeg_tables(&self) -> Option<&[u8]> {
        self.jpeg_tables.as_ref().map(|data| &data[..])
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

    pub fn try_avm2_domain(&self) -> Option<Avm2Domain<'gc>> {
        self.avm2_domain
    }
}

pub struct MovieLibrarySource<'a, 'gc> {
    pub library: &'a MovieLibrary<'gc>,
    pub gc_context: &'a Mutation<'gc>,
}

impl<'a, 'gc> ruffle_render::bitmap::BitmapSource for MovieLibrarySource<'a, 'gc> {
    fn bitmap_size(&self, id: u16) -> Option<ruffle_render::bitmap::BitmapSize> {
        self.library
            .get_bitmap(id)
            .map(|bitmap| ruffle_render::bitmap::BitmapSize {
                width: bitmap.width(),
                height: bitmap.height(),
            })
    }

    fn bitmap_handle(&self, id: u16, backend: &mut dyn RenderBackend) -> Option<BitmapHandle> {
        self.library.get_bitmap(id).map(|bitmap| {
            bitmap
                .bitmap_data_wrapper()
                .bitmap_handle(self.gc_context, backend)
        })
    }
}

impl Default for MovieLibrary<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// Symbol library for multiple movies.
pub struct Library<'gc> {
    /// All the movie libraries.
    movie_libraries: PtrWeakKeyHashMap<Weak<SwfMovie>, MovieLibrary<'gc>>,

    /// A cache of seen device fonts.
    // TODO: Descriptors shouldn't be stored in fonts. Fonts should be a list that we iterate and ask "do you match". A font can have zero or many names.
    device_fonts: FontMap<'gc>,

    /// "Global" embedded fonts, registered through AVM2 `Font.registerFont`.
    /// These should be checked before any Movie-specific library's own fonts.
    global_fonts: FontMap<'gc>,

    /// A set of which fonts we've asked from the backend already, to help with negative caching.
    /// If we've asked for a specific font, record it here and don't ask again.
    font_lookup_cache: FnvHashSet<(String, bool, bool)>,

    /// The implementation names of each default font.
    default_font_names: FnvHashMap<DefaultFont, Vec<String>>,

    /// The cached list of implementations per default font.
    default_font_cache: FnvHashMap<(DefaultFont, bool, bool), Vec<Font<'gc>>>,

    /// A list of the symbols associated with specific AVM2 constructor
    /// prototypes.
    avm2_class_registry: Avm2ClassRegistry<'gc>,
}

unsafe impl<'gc> gc_arena::Collect for Library<'gc> {
    #[inline]
    fn trace(&self, cc: &gc_arena::Collection) {
        for (_, val) in self.movie_libraries.iter() {
            val.trace(cc);
        }
        for (_, val) in self.default_font_cache.iter() {
            val.trace(cc);
        }
        self.device_fonts.trace(cc);
        self.global_fonts.trace(cc);
        self.avm2_class_registry.trace(cc);
    }
}

impl<'gc> Library<'gc> {
    pub fn empty() -> Self {
        Self {
            movie_libraries: PtrWeakKeyHashMap::new(),
            device_fonts: Default::default(),
            global_fonts: Default::default(),
            font_lookup_cache: Default::default(),
            default_font_names: Default::default(),
            default_font_cache: Default::default(),
            avm2_class_registry: Default::default(),
        }
    }

    pub fn library_for_movie(&self, movie: Arc<SwfMovie>) -> Option<&MovieLibrary<'gc>> {
        self.movie_libraries.get(&movie)
    }

    pub fn library_for_movie_mut(&mut self, movie: Arc<SwfMovie>) -> &mut MovieLibrary<'gc> {
        // NOTE(Clippy): Cannot use or_default() here as PtrWeakKeyHashMap does not have such a method on its Entry API
        #[allow(clippy::unwrap_or_default)]
        self.movie_libraries
            .entry(movie)
            .or_insert_with(MovieLibrary::new)
    }

    pub fn known_movies(&self) -> Vec<Arc<SwfMovie>> {
        self.movie_libraries.keys().collect()
    }

    /// Returns the default Font implementations behind the built in names (ie `_sans`)
    pub fn default_font(
        &mut self,
        name: DefaultFont,
        is_bold: bool,
        is_italic: bool,
        ui: &dyn UiBackend,
        renderer: &mut dyn RenderBackend,
        gc_context: &Mutation<'gc>,
    ) -> Vec<Font<'gc>> {
        // Can't use entry api here as we want to use self for `load_device_font`.
        // Cache the value as this will be looked up a lot, and font lookup by name can be expensive if lots of fonts exist.
        if let Some(cache) = self.default_font_cache.get(&(name, is_bold, is_italic)) {
            return cache.clone();
        }

        let mut result = vec![];
        for name in self.default_font_names.entry(name).or_default().clone() {
            if let Some(font) =
                self.get_or_load_device_font(&name, is_bold, is_italic, ui, renderer, gc_context)
            {
                result.push(font);
            }
        }

        self.default_font_cache
            .insert((name, is_bold, is_italic), result.clone());
        result
    }

    /// Returns the device font for use when a font is unavailable.
    pub fn get_or_load_device_font(
        &mut self,
        name: &str,
        is_bold: bool,
        is_italic: bool,
        ui: &dyn UiBackend,
        renderer: &mut dyn RenderBackend,
        gc_context: &Mutation<'gc>,
    ) -> Option<Font<'gc>> {
        // If we have the exact matching font already, use that
        // TODO: We should instead ask each font if it matches a given name. Partial matches are allowed, and fonts may have any amount of names.
        if let Some(font) = self.device_fonts.get(name, is_bold, is_italic) {
            return Some(*font);
        }

        // We don't have this font already. Did we ask for it before?
        let new_request = self
            .font_lookup_cache
            .insert((name.to_string(), is_bold, is_italic));
        if new_request {
            // First time asking for this font, see if our backend can provide anything relevant
            ui.load_device_font(name, is_bold, is_italic, &mut |definition| {
                self.register_device_font(gc_context, renderer, definition)
            });
        }

        // Check again. A backend may or may not have provided some new fonts,
        // and they may or may not be relevant to the one we're asking for.
        match self.device_fonts.get(name, is_bold, is_italic) {
            None => {
                if new_request {
                    warn!("Unknown device font \"{name}\" (bold: {is_bold} italic: {is_italic})");
                }

                // The default fallback:
                // Try to find an existing font to re-use instead of giving up.
                if let Some(font) = self.device_fonts.find(name, is_bold, is_italic) {
                    return Some(font);
                }

                None
            }
            Some(font) => Some(*font),
        }
    }

    pub fn set_default_font(&mut self, font: DefaultFont, names: Vec<String>) {
        self.default_font_names.insert(font, names);
        self.default_font_cache.clear();
    }

    pub fn register_device_font(
        &mut self,
        gc_context: &Mutation<'gc>,
        renderer: &mut dyn RenderBackend,
        definition: FontDefinition<'_>,
    ) {
        match definition {
            FontDefinition::SwfTag(tag, encoding) => {
                let font =
                    Font::from_swf_tag(gc_context, renderer, tag, encoding, FontType::Device);
                let name = font.descriptor().name().to_owned();
                info!("Loaded new device font \"{name}\" from swf tag");
                self.device_fonts.register(font);
            }
        }
        self.default_font_cache.clear();
    }

    /// Find a font by it's name and parameters.
    pub fn get_embedded_font_by_name(
        &self,
        name: &str,
        is_bold: bool,
        is_italic: bool,
        movie: Option<Arc<SwfMovie>>,
    ) -> Option<Font<'gc>> {
        if let Some(font) = self.global_fonts.find(name, is_bold, is_italic) {
            return Some(font);
        }
        if let Some(movie) = movie {
            if let Some(library) = self.library_for_movie(movie) {
                if let Some(font) = library.fonts.find(name, is_bold, is_italic) {
                    return Some(font);
                }
            }
        }
        None
    }

    pub fn global_fonts(&self) -> Vec<Font<'gc>> {
        self.global_fonts.all()
    }

    pub fn register_global_font(&mut self, font: Font<'gc>) {
        self.global_fonts.register(font);
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

#[derive(Collect, Default)]
#[collect(no_drop)]
struct FontMap<'gc>(FnvHashMap<FontDescriptor, Font<'gc>>);

impl<'gc> FontMap<'gc> {
    pub fn register(&mut self, font: Font<'gc>) {
        // The first font with a given descriptor wins
        if !self.0.contains_key(font.descriptor()) {
            self.0.insert(font.descriptor().clone(), font);
        }
    }

    pub fn get(&self, name: &str, is_bold: bool, is_italic: bool) -> Option<&Font<'gc>> {
        self.0
            .get(&FontDescriptor::from_parts(name, is_bold, is_italic))
    }

    pub fn find(&self, name: &str, is_bold: bool, is_italic: bool) -> Option<Font<'gc>> {
        // The order here is specific, and tested in `tests/swfs/fonts/embed_matching/fallback_preferences`

        // Exact match
        if let Some(font) = self.get(name, is_bold, is_italic) {
            return Some(*font);
        }

        if is_italic ^ is_bold {
            // If one is set (but not both), then try upgrading to bold italic...
            if let Some(font) = self.get(name, true, true) {
                return Some(*font);
            }

            // and then downgrading to regular
            if let Some(font) = self.get(name, false, false) {
                return Some(*font);
            }

            // and then finally whichever one we don't have set
            if let Some(font) = self.get(name, !is_bold, !is_italic) {
                return Some(*font);
            }
        } else {
            // We don't have an exact match and we were either looking for regular or bold-italic

            if is_italic && is_bold {
                // Do we have regular? (unless we already looked for it)
                if let Some(font) = self.get(name, false, false) {
                    return Some(*font);
                }
            }

            // Do we have bold?
            if let Some(font) = self.get(name, true, false) {
                return Some(*font);
            }

            // Do we have italic?
            if let Some(font) = self.get(name, false, true) {
                return Some(*font);
            }

            if !is_bold && !is_italic {
                // Do we have bold italic? (unless we already looked for it)
                if let Some(font) = self.get(name, true, true) {
                    return Some(*font);
                }
            }
        }

        None
    }

    pub fn all(&self) -> Vec<Font<'gc>> {
        self.0.values().copied().collect()
    }
}
