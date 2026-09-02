use crate::avm1::{PropertyMap as Avm1PropertyMap, PropertyMap};
use crate::avm2::{
    Class as Avm2Class, ClassWeak as Avm2ClassWeak, Domain as Avm2Domain,
    TranslationUnitWeak as Avm2TranslationUnitWeak,
};
use crate::backend::audio::SoundHandle;
use crate::character::Character;
use crate::memory_report::LibraryMemoryUsage;

use crate::display_object::{Bitmap, DisplayObjectWeak, Graphic, MorphShape, Text};
use crate::font::{Font, FontDescriptor, FontLike, FontQuery, FontType};
use crate::prelude::*;
use crate::string::AvmString;
use crate::tag_utils::SwfMovie;
use gc_arena::collect::Trace;
use gc_arena::{Collect, Finalization, Gc, GcWeak, Mutation};
use ruffle_render::backend::RenderBackend;
use ruffle_render::bitmap::BitmapHandle;
use ruffle_render::utils::remove_invalid_jpeg_data;
use ruffle_wstr::{WStr, WString};

use crate::backend::ui::{FontDefinition, UiBackend};
use crate::font::DefaultFont;
use fnv::{FnvHashMap, FnvHashSet};
use std::collections::HashMap;
use std::sync::{Arc, Weak};
use weak_table::PtrWeakKeyHashMap;

/// Identity key for an AVM2 class.
///
/// A [`Avm2ClassWeak`] keeps its own allocation alive even after the class it
/// points at is collected, so while an entry exists its address cannot be
/// reused by a different class and pointer identity stays sound.
fn class_key(class: Avm2Class<'_>) -> usize {
    class.as_ptr() as usize
}

/// One `SymbolClass` mapping: which character of which movie a class builds.
///
/// Both halves are weak. The class must not be held strongly, because a class
/// owns its translation unit, which owns a strong `Arc<SwfMovie>` of the movie
/// it was loaded from - a strong class here would pin the entire SWF, and with
/// it this entry's own weak movie reference, so the entry could never expire.
#[derive(Collect)]
#[collect(no_drop)]
struct ClassSymbol<'gc> {
    class: Avm2ClassWeak<'gc>,

    #[collect(require_static)]
    movie: Weak<SwfMovie>,

    #[collect(require_static)]
    symbol: CharacterId,
}

/// The mappings between class objects and library characters defined by
/// `SymbolClass`.
#[derive(Collect, Default)]
#[collect(no_drop)]
pub struct Avm2ClassRegistry<'gc> {
    /// The character each AVM2 class is expected to instantiate, keyed by class
    /// identity.
    ///
    /// Dead entries are cleared out by [`Self::remove_dead_classes`]; until
    /// then their weak references keep the addresses they were keyed on unique,
    /// so a stale entry can never be mistaken for a newer class.
    class_map: FnvHashMap<usize, ClassSymbol<'gc>>,
}

impl<'gc> Avm2ClassRegistry<'gc> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Retrieve the library symbol for a given AVM2 class object.
    ///
    /// A value of `None` indicates that this AVM2 class is not associated with
    /// a library symbol.
    pub fn class_symbol(&self, class_def: Avm2Class<'gc>) -> Option<(Arc<SwfMovie>, CharacterId)> {
        let entry = self.class_map.get(&class_key(class_def))?;
        Some((entry.movie.upgrade()?, entry.symbol))
    }

    /// Forgets the entries of classes that are dead: already collected, or
    /// not reached by the marking phase this finalization concludes.
    ///
    /// Neither check upgrades the class, so asking the question never keeps
    /// a class alive for another collection cycle.
    fn remove_dead_classes(&mut self, fc: &Finalization<'gc>) {
        self.class_map.retain(|_, entry| {
            !entry.class.is_dropped() && !entry.class.is_dead(fc) && entry.movie.strong_count() > 0
        });
    }

    /// Associate an AVM2 class definition with a given library symbol.
    pub fn set_class_symbol(
        &mut self,
        class_def: Avm2Class<'gc>,
        movie: Arc<SwfMovie>,
        symbol: CharacterId,
    ) {
        let key = class_key(class_def);
        if let Some(old) = self
            .class_map
            .get(&key)
            .and_then(|entry| Some((entry.movie.upgrade()?, entry.symbol)))
        {
            if Arc::ptr_eq(&movie, &old.0) && symbol != old.1 {
                // Flash player actually allows using the same class in multiple SymbolClass
                // entries in the same swf, with *different* symbol ids. Whichever one
                // is processed first will *win*, and the second one will be ignored.
                // We still log a warning, since we wouldn't expect this to happen outside
                // of deliberately crafted SWFs.
                tracing::warn!(
                    "Tried to overwrite class {:?} id={:?} with symbol id={:?} from same movie",
                    class_def,
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
        self.class_map.insert(
            key,
            ClassSymbol {
                class: class_def.downgrade(),
                movie: Arc::downgrade(&movie),
                symbol,
            },
        );
    }
}

/// Why a movie's library is exempt from being released.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Pin {
    /// Released like any other library once nothing needs it any more.
    None,

    /// The movie the player was started with. Everything else is loaded from
    /// it, directly or indirectly, and it can never be unloaded.
    Root,

    /// A movie fetched by an `ImportAssets` tag. Its characters are copied
    /// into the importing movie's library, which then asks for their
    /// definitions by movie rather than through any object of its own, so
    /// nothing traceable would keep this library alive.
    ImportedAssets,
}

/// Symbol library for a single given SWF.
///
/// # Lifetime
///
/// A library is not an ordinary garbage-collected value, and it is not an
/// ordinary root either. It is stored by the [`Library`] root, but the
/// collector is only told about its *anchors* - the things outside the
/// library that would need it - and not about its *contents*:
///
/// * the display object the movie was loaded into (`content_root`);
/// * the translation units of the movie's ABC code (`translation_units`),
///   which every class, method and script from the movie keeps alive;
/// * the definition data each character shares with its instances, which
///   is checked directly at the end of marking.
///
/// When marking has finished, [`Library::resolve_releasable_libraries`] asks
/// whether any anchor was reached from the root. If one was, the contents are
/// resurrected and marking resumes from them; if none was, the library is
/// dropped and its contents are swept with everything else.
///
/// This is what lets a loaded SWF go away once it is genuinely finished
/// with, while still keeping it for exactly as long as Flash would: a class
/// taken out of its `ApplicationDomain` keeps its symbols instantiable, a
/// clip moved out of the loaded content keeps its timeline working, a domain
/// that is still referenced keeps its definitions. The library's own strong
/// references to its movie and its characters - which are what made it
/// immortal before, since the map it lives in is weakly keyed on that very
/// movie - no longer count for anything, because they are never traced.
///
/// Libraries that nothing traceable could ever anchor are pinned instead;
/// see [`Pin`].
pub struct MovieLibrary<'gc> {
    swf: Arc<SwfMovie>,
    characters: HashMap<CharacterId, Character<'gc>>,
    export_characters: Avm1PropertyMap<'gc, CharacterId>,
    imported_assets: HashMap<AvmString<'gc>, CharacterId>,
    jpeg_tables: Option<Vec<u8>>,
    fonts: FontMap<'gc>,
    avm2_domain: Option<Avm2Domain<'gc>>,

    /// The display object this movie was loaded into, for movies that were
    /// loaded by a `Loader` or `loadMovie`. An anchor; held weakly so that
    /// the library itself never keeps its content alive.
    content_root: Option<DisplayObjectWeak<'gc>>,

    /// The translation units loaded from this movie's `DoAbc` tags. Anchors;
    /// held weakly so that the library never keeps the movie's code alive.
    ///
    /// Every class, method and script from the movie holds its translation
    /// unit strongly, so if any of them is reachable - from an
    /// `ApplicationDomain` that still lists its definitions, from an instance,
    /// from a closure or a timer - the unit is marked, and the library has to
    /// stay because that code can still ask for the movie's characters.
    translation_units: Vec<Avm2TranslationUnitWeak<'gc>>,

    /// Whether this library is exempt from being released. See [`Pin`].
    pin: Pin,

    /// Set by the finalization pass of the current collection cycle once it
    /// has decided to keep this library; cleared again when the cycle's dead
    /// libraries are dropped.
    kept: bool,

    /// Bytes of non-garbage-collected memory this library keeps alive: the
    /// movie's own SWF data plus the still-compressed source of every bitmap it
    /// has registered.
    ///
    /// The collector paces itself against the memory it can see, and it can
    /// only see the size of the `Gc` allocations themselves. A movie's
    /// characters are small objects pointing at very large buffers, so without
    /// telling the collector about those buffers a session can hold hundreds of
    /// megabytes of dead content while the collector believes it has almost
    /// nothing to do. See [`Library::release_unreachable_movies`].
    external_bytes: usize,
}

unsafe impl<'gc> Collect<'gc> for MovieLibrary<'gc> {
    /// Traces the whole library. Only used for pinned libraries, and by the
    /// resurrection pass; see [`MovieLibraries`].
    fn trace<C: Trace<'gc>>(&self, cc: &mut C) {
        self.trace_anchors(cc);
        self.trace_contents(cc);
    }
}

/// A tracer that resurrects instead of marks: every strong pointer it is
/// handed is brought back for the current collection cycle, along with -
/// once marking resumes - everything reachable from it.
struct Resurrector<'a, 'gc>(&'a Finalization<'gc>);

impl<'gc> Trace<'gc> for Resurrector<'_, 'gc> {
    fn trace_gc(&mut self, gc: Gc<'gc, ()>) {
        Gc::resurrect(self.0, gc);
    }

    fn trace_gc_weak(&mut self, _weak: GcWeak<'gc, ()>) {
        // Weak pointers were already traced from the root, and resurrecting
        // through them would defeat their purpose.
    }
}

impl<'gc> MovieLibrary<'gc> {
    pub fn new(swf: Arc<SwfMovie>) -> Self {
        Self {
            external_bytes: swf.uncompressed_len().max(0) as usize,
            swf,
            characters: HashMap::new(),
            imported_assets: HashMap::new(),
            export_characters: Avm1PropertyMap::new(),
            jpeg_tables: None,
            fonts: Default::default(),
            avm2_domain: None,
            content_root: None,
            translation_units: Vec::new(),
            pin: Pin::None,
            kept: false,
        }
    }

    /// Traces the weak handles on the things outside this library that would
    /// need it. These are traced from the root for every library, pinned or
    /// not: a weak pointer that is not traced cannot tell that its target is
    /// about to be collected.
    fn trace_anchors<C: Trace<'gc>>(&self, cc: &mut C) {
        cc.trace(&self.content_root);
        cc.trace(&self.translation_units);
    }

    /// Traces everything this library owns: its characters, fonts, export
    /// names and domain. Only traced from the root for pinned libraries; for
    /// every other library this is what the finalization pass resurrects
    /// once it knows the library is still needed.
    fn trace_contents<C: Trace<'gc>>(&self, cc: &mut C) {
        cc.trace(&self.characters);
        cc.trace(&self.export_characters);
        cc.trace(&self.imported_assets);
        cc.trace(&self.fonts);
        cc.trace(&self.avm2_domain);
    }

    /// Whether anything outside this library still reaches something that
    /// needs it. Only meaningful during finalization, once marking has
    /// finished and before anything is swept.
    fn is_needed(&self, fc: &Finalization<'gc>) -> bool {
        if self.content_root.is_some_and(|root| !root.is_dead(fc)) {
            return true;
        }
        if self.translation_units.iter().any(|unit| !unit.is_dead(fc)) {
            return true;
        }
        self.characters
            .values()
            .any(|character| character.has_reachable_instances(fc))
    }

    fn is_pinned(&self) -> bool {
        self.pin != Pin::None
    }

    /// Marks this library as fetched by `ImportAssets`, which keeps it for
    /// the rest of the session. See [`Pin::ImportedAssets`].
    pub fn pin_for_imported_assets(&mut self) {
        self.pin = Pin::ImportedAssets;
    }

    /// Records a translation unit loaded from this movie's ABC code. See
    /// [`Self::translation_units`].
    pub fn register_translation_unit(&mut self, unit: Avm2TranslationUnitWeak<'gc>) {
        self.translation_units.push(unit);
    }

    /// Forgets translation units that have been collected.
    fn remove_dead_translation_units(&mut self, fc: &Finalization<'gc>) {
        self.translation_units.retain(|unit| !unit.is_dead(fc));
    }

    /// Non-GC bytes this library is keeping resident.
    fn external_bytes(&self) -> usize {
        self.external_bytes
    }

    /// Records the display object a loaded movie was placed into.
    ///
    /// Only call this for movies loaded by `Loader`/`loadMovie`; the root movie
    /// deliberately has no content root, so that its library is never swept.
    pub fn set_content_root(&mut self, root: DisplayObjectWeak<'gc>) {
        self.content_root = Some(root);
    }

    /// Whether this library was loaded for content that is still reachable.
    ///
    /// Reported by [`crate::memory_report`]. A library whose content is gone
    /// but which is still listed is either still needed by something else -
    /// a class held out of its `ApplicationDomain`, say - or has not been
    /// through a collection yet.
    pub fn has_live_content(&self, mc: &Mutation<'gc>) -> bool {
        self.content_root
            .is_some_and(|root| root.upgrade(mc).is_some())
    }

    /// Totals up everything this library is currently keeping resident.
    ///
    /// Used by [`crate::memory_report`] to attribute retained memory to the
    /// movie that owns it.
    pub fn memory_usage(&self) -> LibraryMemoryUsage {
        let mut usage = LibraryMemoryUsage {
            characters: self.characters.len(),
            has_domain: self.avm2_domain.is_some(),
            ..Default::default()
        };

        for character in self.characters.values() {
            // Does this character hold a strong `Arc` back to the very movie
            // this library is weakly keyed on? Counting these is what
            // distinguishes "somebody still needs this movie" from "this
            // library is the only thing keeping its own key alive".
            let character_movie = match character {
                Character::EditText(o) => Some(o.movie()),
                Character::Graphic(o) => Some(o.movie()),
                Character::MovieClip(o) => Some(o.movie()),
                Character::Avm1Button(o) => Some(o.movie()),
                Character::Avm2Button(o) => Some(o.movie()),
                Character::MorphShape(o) => Some(o.movie()),
                Character::Text(o) => Some(o.movie()),
                Character::Video(o) => Some(o.movie()),
                _ => None,
            };
            if let Some(m) = character_movie
                && Arc::ptr_eq(&m, &self.swf)
            {
                usage.self_refs += 1;
            }

            match character {
                Character::Bitmap(bitmap) => {
                    let compressed = bitmap.compressed();
                    let size = compressed.size();
                    usage.bitmaps += 1;
                    if bitmap.is_uploaded() {
                        usage.uploaded_bitmaps += 1;
                    }
                    usage.bitmap_source_bytes += compressed.source_bytes();
                    // Four bytes per pixel once decoded to RGBA, which is what
                    // both the decoded copy and the GPU texture cost.
                    usage.bitmap_decoded_bytes += size.width as usize * size.height as usize * 4;
                }
                Character::Sound(_) => usage.sounds += 1,
                Character::Font(_) => usage.fonts += 1,
                _ => {}
            }
        }

        usage
    }

    /// Registers a character; returns `true` if successful, or `false` if a character with
    /// the given ID already exists.
    pub fn register_character(&mut self, id: CharacterId, character: Character<'gc>) -> bool {
        use std::collections::hash_map::Entry;
        match self.characters.entry(id) {
            Entry::Vacant(e) => {
                if let Character::Font(font) = character {
                    self.fonts.register(font);
                }
                if let Character::Bitmap(bitmap) = character {
                    self.external_bytes += bitmap.compressed().source_bytes();
                }
                e.insert(character);
                true
            }
            Entry::Occupied(_) => {
                tracing::error!("Character ID collision: Tried to register ID {} twice", id);
                false
            }
        }
    }

    /// Registers an export name for a given character ID.
    /// This character will then be instantiable from AVM1.
    pub fn register_export(&mut self, id: CharacterId, export_name: AvmString<'gc>) {
        let character_exists = self.contains_character(id);
        debug_assert!(character_exists);
        if !character_exists {
            tracing::error!(
                "Tried to register export '{export_name}' for a non-existent character {id}"
            );
            return;
        }

        self.export_characters.insert(export_name, id, false);
    }

    pub fn register_font_name(&mut self, character_id: u16, font_name: &str) {
        let Some(Character::Font(font)) = self.character_by_id(character_id) else {
            return;
        };

        let descriptor = FontDescriptor::from_parts(
            font_name,
            font.descriptor().bold(),
            font.descriptor().italic(),
        );
        self.fonts.register_with_descriptor(font, &descriptor);
    }

    pub fn characters(&self) -> &HashMap<CharacterId, Character<'gc>> {
        &self.characters
    }

    pub fn export_characters(&self) -> &PropertyMap<'gc, CharacterId> {
        &self.export_characters
    }

    pub fn contains_character(&self, id: CharacterId) -> bool {
        self.characters.contains_key(&id)
    }

    pub fn character_by_id(&self, id: CharacterId) -> Option<Character<'gc>> {
        self.characters.get(&id).copied()
    }

    pub fn character_by_export_name(&self, name: &WStr) -> Option<(CharacterId, Character<'gc>)> {
        if let Some(id) = self.export_characters.get(name, false)
            && let Some(character) = self.characters.get(id)
        {
            return Some((*id, *character));
        }
        None
    }

    pub fn character_id_by_import_name(&self, name: &WStr) -> Option<CharacterId> {
        self.imported_assets.get(name).copied()
    }

    pub fn register_import(&mut self, name: AvmString<'gc>, id: CharacterId) {
        self.imported_assets.insert(name, id);
    }

    /// Instantiates the library item with the given character ID into a display object.
    /// The object must then be post-instantiated before being used.
    pub fn instantiate_by_id(
        &self,
        id: CharacterId,
        mc: &Mutation<'gc>,
    ) -> Option<DisplayObject<'gc>> {
        if let Some(&character) = self.characters.get(&id) {
            self.instantiate_display_object(id, character, mc)
        } else {
            tracing::error!("Tried to instantiate a non-registered character ID {id}");
            None
        }
    }

    /// Instantiates the library item with the given export name into a display object.
    /// The object must then be post-instantiated before being used.
    pub fn instantiate_by_export_name(
        &self,
        export_name: &WStr,
        mc: &Mutation<'gc>,
    ) -> Option<DisplayObject<'gc>> {
        if let Some((id, character)) = self.character_by_export_name(export_name) {
            self.instantiate_display_object(id, character, mc)
        } else {
            tracing::error!("Tried to instantiate a non-registered character {export_name}");
            None
        }
    }

    /// Instantiates the given character into a display object.
    /// The object must then be post-instantiated before being used.
    fn instantiate_display_object(
        &self,
        id: CharacterId,
        character: Character<'gc>,
        mc: &Mutation<'gc>,
    ) -> Option<DisplayObject<'gc>> {
        match character {
            Character::Bitmap(bitmap) => {
                let avm2_class = bitmap.avm2_class();
                let bitmap = bitmap.compressed().decode().unwrap();
                let bitmap = Bitmap::new(mc, id, bitmap, self.swf.clone());
                bitmap.set_avm2_bitmapdata_class(mc, avm2_class);
                Some(bitmap.instantiate(mc).into())
            }
            Character::EditText(edit_text) => Some(edit_text.instantiate(mc).into()),
            Character::Graphic(graphic) => Some(graphic.instantiate(mc).into()),
            Character::MorphShape(morph_shape) => Some(morph_shape.instantiate(mc).into()),
            Character::MovieClip(movie_clip) => Some(movie_clip.instantiate(mc).into()),
            Character::Avm1Button(button) => Some(button.instantiate(mc).into()),
            Character::Avm2Button(button) => Some(button.instantiate(mc).into()),
            Character::Text(text) => Some(text.instantiate(mc).into()),
            Character::Video(video) => Some(video.instantiate(mc).into()),
            _ => {
                // Cannot instantiate non-display object
                None
            }
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
}

impl ruffle_render::bitmap::BitmapSource for MovieLibrarySource<'_, '_> {
    fn bitmap_size(&self, id: u16) -> Option<ruffle_render::bitmap::BitmapSize> {
        if let Some(Character::Bitmap(bitmap)) = self.library.characters.get(&id) {
            Some(bitmap.compressed().size())
        } else {
            None
        }
    }

    fn bitmap_handle(&self, id: u16, backend: &mut dyn RenderBackend) -> Option<BitmapHandle> {
        let Some(Character::Bitmap(bitmap)) = self.library.characters.get(&id) else {
            return None;
        };

        match bitmap.bitmap_handle(backend) {
            Ok(handle) => Some(handle),
            Err(e) => {
                tracing::error!("Failed to register bitmap character {id}: {e}");
                None
            }
        }
    }
}

/// One library per movie.
///
/// Weakly keyed on the movie, but note that this on its own never releases
/// anything: a library and nearly every character in it hold strong
/// `Arc<SwfMovie>` clones of the very movie they are filed under, so the key
/// cannot expire while the entry exists. Entries are released by the
/// finalization pass instead; see [`MovieLibrary`].
struct MovieLibraries<'gc>(PtrWeakKeyHashMap<Weak<SwfMovie>, MovieLibrary<'gc>>);

unsafe impl<'gc> Collect<'gc> for MovieLibraries<'gc> {
    #[inline]
    fn trace<C: Trace<'gc>>(&self, cc: &mut C) {
        for (_, library) in self.0.iter() {
            if library.is_pinned() {
                cc.trace(library);
            } else {
                // The contents are deliberately left untraced. They are
                // resurrected at the end of marking if the library turns
                // out to be needed, and dropped with the library otherwise.
                library.trace_anchors(cc);
            }
        }
    }
}

impl<'gc> MovieLibraries<'gc> {
    fn new() -> Self {
        Self(PtrWeakKeyHashMap::new())
    }

    fn get(&self, key: &Arc<SwfMovie>) -> Option<&MovieLibrary<'gc>> {
        self.0.get(key)
    }

    fn get_or_insert_mut(&mut self, movie: Arc<SwfMovie>) -> &mut MovieLibrary<'gc> {
        self.0
            .entry(movie.clone())
            .or_insert_with(|| MovieLibrary::new(movie))
    }

    fn known_movies(&self) -> impl Iterator<Item = Arc<SwfMovie>> {
        self.0.keys()
    }

    /// Keeps every releasable library that something outside it still
    /// reaches, by resurrecting its contents.
    ///
    /// Returns whether anything was resurrected. If so, marking has to be
    /// resumed - the resurrected contents may reach the anchors of other
    /// libraries - and this asked again once it has finished.
    fn resurrect_needed(&mut self, fc: &Finalization<'gc>) -> bool {
        let mut resurrected = false;
        for (_, library) in self.0.iter_mut() {
            if library.is_pinned() || library.kept {
                continue;
            }
            if library.is_needed(fc) {
                library.kept = true;
                library.trace_contents(&mut Resurrector(fc));
                resurrected = true;
            }
        }
        resurrected
    }

    /// Drops every releasable library that the finalization pass did not
    /// keep, and clears the marks of the ones it did for the next cycle.
    ///
    /// Returns the number of libraries dropped.
    fn drop_unneeded(&mut self, fc: &Finalization<'gc>) -> usize {
        // Not `retain`: `weak_table`'s implementation walks bucket indices
        // and, after removing an entry, shifts the following entries back
        // into the slot it has already passed, so it can skip entries. A
        // skipped library here would survive the sweep with its contents
        // untraced. Decide first, then remove by key.
        let mut dead = Vec::new();
        for (movie, library) in self.0.iter_mut() {
            if library.is_pinned() || std::mem::take(&mut library.kept) {
                library.remove_dead_translation_units(fc);
            } else {
                dead.push(movie);
            }
        }
        let dropped = dead.len();
        for movie in dead {
            self.0.remove(&movie);
        }
        dropped
    }

    fn set_root(&mut self, movie: Arc<SwfMovie>) {
        for (_, library) in self.0.iter_mut() {
            if library.pin == Pin::Root {
                library.pin = Pin::None;
            }
        }
        self.get_or_insert_mut(movie).pin = Pin::Root;
    }
}

/// Symbol library for multiple movies.
#[derive(Collect)]
#[collect(no_drop)]
pub struct Library<'gc> {
    /// All the movie libraries.
    movie_libraries: MovieLibraries<'gc>,

    /// A cache of seen device fonts.
    // TODO: Descriptors shouldn't be stored in fonts. Fonts should be a list that we iterate and ask "do you match". A font can have zero or many names.
    device_fonts: FontMap<'gc>,

    /// "Global" embedded fonts, registered through AVM2 `Font.registerFont`.
    /// These should be checked before any Movie-specific library's own fonts.
    global_fonts: FontMap<'gc>,

    /// A set of which fonts we've asked from the backend already, to help with negative caching.
    /// If we've asked for a specific font, record it here and don't ask again.
    font_lookup_cache: FnvHashSet<FontQuery>,

    /// Cached font sort queries.
    font_sort_cache: FnvHashMap<FontQuery, Vec<Font<'gc>>>,

    /// The implementation names of each default font.
    default_font_names: FnvHashMap<DefaultFont, Vec<String>>,

    /// The cached list of implementations per default font.
    default_font_cache: FnvHashMap<(DefaultFont, bool, bool), Vec<Font<'gc>>>,

    /// A list of the symbols associated with specific AVM2 constructor
    /// prototypes.
    avm2_class_registry: Avm2ClassRegistry<'gc>,

    /// The non-GC memory currently reported to the collector, so that only the
    /// change since the last sweep has to be reported.
    #[collect(require_static)]
    reported_external_bytes: usize,
}

impl<'gc> Library<'gc> {
    pub fn empty() -> Self {
        Self {
            movie_libraries: MovieLibraries::new(),
            device_fonts: Default::default(),
            global_fonts: Default::default(),
            font_lookup_cache: Default::default(),
            font_sort_cache: Default::default(),
            default_font_names: Default::default(),
            default_font_cache: Default::default(),
            avm2_class_registry: Default::default(),
            reported_external_bytes: 0,
        }
    }

    pub fn library_for_movie(&self, movie: Arc<SwfMovie>) -> Option<&MovieLibrary<'gc>> {
        self.movie_libraries.get(&movie)
    }

    pub fn library_for_movie_mut(&mut self, movie: Arc<SwfMovie>) -> &mut MovieLibrary<'gc> {
        self.movie_libraries.get_or_insert_mut(movie)
    }

    pub fn known_movies(&self) -> impl Iterator<Item = Arc<SwfMovie>> {
        self.movie_libraries.known_movies()
    }

    /// Marks `movie` as the movie the player was started with, whose library
    /// is never released. See [`Pin::Root`].
    pub fn set_root_movie(&mut self, movie: Arc<SwfMovie>) {
        self.movie_libraries.set_root(movie);
    }

    /// The finalization step of a collection cycle: decides which loaded
    /// movies' libraries live and which die, now that marking has finished.
    ///
    /// The collector was only told about each library's anchors, not its
    /// contents (see [`MovieLibrary`]). Any library with a live anchor has
    /// its contents resurrected; in that case this returns `true`, and the
    /// caller has to resume marking and call this again, because the
    /// resurrected contents may reach the anchors of other libraries. Once
    /// nothing more is resurrected, every remaining unpinned library is
    /// dropped, its objects are left for the sweep, and this returns
    /// `false`.
    ///
    /// Must be called between the end of marking and the start of sweeping,
    /// every cycle: an untraced library whose contents were neither
    /// resurrected nor dropped would be left pointing at swept objects.
    pub fn resolve_releasable_libraries(&mut self, fc: &Finalization<'gc>) -> bool {
        if self.movie_libraries.resurrect_needed(fc) {
            return true;
        }

        let dropped = self.movie_libraries.drop_unneeded(fc);
        if dropped > 0 {
            tracing::debug!("Released {dropped} unreachable movie librar(y/ies)");
        }
        self.avm2_class_registry.remove_dead_classes(fc);
        self.report_external_allocation(fc);
        false
    }

    /// Keeps the collector's idea of how much memory is in play in step with
    /// the SWF data and bitmap sources the movie libraries actually hold.
    ///
    /// Without this the collector only counts its own allocations, which for a
    /// movie library is a rounding error next to the buffers those allocations
    /// point at, and it ends up pacing itself far too slowly to keep up with
    /// content that loads and drops megabytes of assets at a time.
    pub fn report_external_allocation(&mut self, mc: &Mutation<'gc>) {
        let total: usize = self
            .movie_libraries
            .0
            .iter()
            .map(|(_, library)| library.external_bytes())
            .sum();

        let metrics = mc.metrics();
        if total > self.reported_external_bytes {
            metrics.mark_external_allocation(total - self.reported_external_bytes);
        } else {
            metrics.mark_external_deallocation(self.reported_external_bytes - total);
        }
        self.reported_external_bytes = total;
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
        // First try to find any exactly matching fonts.
        for name in self.default_font_names.entry(name).or_default().clone() {
            let query = FontQuery::new(FontType::Device, name, is_bold, is_italic);
            if let Some(font) = self.get_or_load_exact_device_font(&query, ui, renderer, gc_context)
            {
                result.push(font);
                break; // TODO: Return multiple fonts when it's needed.
            }
        }

        // Nothing found, try a compatible font.
        if result.is_empty() {
            for name in self.default_font_names.entry(name).or_default().clone() {
                let query = FontQuery::new(FontType::Device, name, is_bold, is_italic);
                if let Some(font) = self.device_fonts.find(&query) {
                    result.push(font);
                    break; // TODO: Return multiple fonts when it's needed.
                }
            }
        }

        self.default_font_cache
            .insert((name, is_bold, is_italic), result.clone());
        result
    }

    /// Returns the device font exactly matching the requested options.
    fn get_or_load_exact_device_font(
        &mut self,
        query: &FontQuery,
        ui: &dyn UiBackend,
        renderer: &mut dyn RenderBackend,
        gc_context: &Mutation<'gc>,
    ) -> Option<Font<'gc>> {
        // If we have the exact matching font already, use that
        // TODO: We should instead ask each font if it matches a given name. Partial matches are allowed, and fonts may have any amount of names.
        if let Some(font) = self.device_fonts.get(query) {
            return Some(font);
        }

        // We don't have this font already. Did we ask for it before?
        let new_request = self.font_lookup_cache.insert(query.clone());
        if new_request {
            // First time asking for this font, see if our backend can provide anything relevant
            ui.load_device_font(query, &mut |definition| {
                self.register_device_font(gc_context, renderer, definition)
            });

            // Check again. A backend may or may not have provided some new fonts,
            // and they may or may not be relevant to the one we're asking for.
            if let Some(font) = self.device_fonts.get(query) {
                return Some(font);
            }

            let name = &query.name;
            let is_bold = query.is_bold;
            let is_italic = query.is_italic;
            warn!("Unknown device font \"{name}\" (bold: {is_bold}, italic: {is_italic})");
        }

        None
    }

    /// Returns the device font compatible with the requested options.
    pub fn get_or_load_device_font(
        &mut self,
        name: &str,
        is_bold: bool,
        is_italic: bool,
        ui: &dyn UiBackend,
        renderer: &mut dyn RenderBackend,
        gc_context: &Mutation<'gc>,
    ) -> Option<Font<'gc>> {
        let query = FontQuery::new(FontType::Device, name.to_owned(), is_bold, is_italic);

        // Try to find an exactly matching font.
        if let Some(font) = self.get_or_load_exact_device_font(&query, ui, renderer, gc_context) {
            return Some(font);
        }

        // Fallback: Try to find an existing font to re-use instead of giving up.
        self.device_fonts.find(&query)
    }

    fn sort_device_fonts(
        &mut self,
        query: &FontQuery,
        ui: &dyn UiBackend,
        renderer: &mut dyn RenderBackend,
        gc_context: &Mutation<'gc>,
    ) -> Vec<Font<'gc>> {
        // First, ask the backend to sort the fonts for us.
        let fonts = ui.sort_device_fonts(query, &mut |definition| {
            self.register_device_font(gc_context, renderer, definition)
        });

        let fonts: Vec<Font<'gc>> = fonts
            .iter()
            .filter_map(|font_query| self.device_fonts.get(font_query))
            .collect();

        if !fonts.is_empty() {
            return fonts;
        }

        // When the backend failed (or doesn't support sorting fonts), fall back
        // to loading one font only without sorting.
        let font = self.get_or_load_device_font(
            &query.name,
            query.is_bold,
            query.is_italic,
            ui,
            renderer,
            gc_context,
        );
        font.map(|font| vec![font]).unwrap_or_default()
    }

    pub fn get_or_sort_device_fonts(
        &mut self,
        name: &str,
        is_bold: bool,
        is_italic: bool,
        ui: &dyn UiBackend,
        renderer: &mut dyn RenderBackend,
        gc_context: &Mutation<'gc>,
    ) -> Vec<Font<'gc>> {
        // TODO We should be able to return a &Vec here, but (1) the borrow
        //   checker is too strict and doesn't allow if branching, and
        //   (2) there's no way to insert a value and get a reference to
        //   it at the same time.
        let query = FontQuery::new(FontType::Device, name.to_owned(), is_bold, is_italic);
        if let Some(fonts) = self.font_sort_cache.get(&query) {
            return fonts.clone();
        }

        let fonts = self.sort_device_fonts(&query, ui, renderer, gc_context);
        self.font_sort_cache.insert(query, fonts.clone());
        fonts
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
                let is_bold = font.descriptor().bold();
                let is_italic = font.descriptor().italic();
                tracing::debug!(
                    "Loaded new device font \"{name}\" (bold: {is_bold}, italic: {is_italic}) from swf tag"
                );
                self.device_fonts.register(font);
            }
            FontDefinition::FontFile {
                name,
                is_bold,
                is_italic,
                data,
                index,
            } => {
                let descriptor = FontDescriptor::from_parts(&name, is_bold, is_italic);
                if let Ok(font) =
                    Font::from_font_file(gc_context, descriptor, data, index, FontType::Device)
                {
                    let name = font.descriptor().name().to_owned();
                    tracing::debug!(
                        "Loaded new device font \"{name}\" (bold: {is_bold}, italic: {is_italic}) from file"
                    );
                    self.device_fonts.register(font);
                } else {
                    warn!("Failed to load device font from file");
                }
            }
            FontDefinition::ExternalRenderer {
                name,
                is_bold,
                is_italic,
                font_renderer,
            } => {
                let descriptor = FontDescriptor::from_parts(&name, is_bold, is_italic);
                let font = Font::from_renderer(gc_context, descriptor, font_renderer);
                tracing::debug!(
                    "Loaded new externally rendered font \"{name}\" (bold: {is_bold}, italic: {is_italic})"
                );
                self.device_fonts.register(font);
            }
        }
        self.default_font_cache.clear();
    }

    /// Find a font by its name and parameters.
    pub fn get_embedded_font_by_name(
        &self,
        name: &str,
        font_type: FontType,
        is_bold: bool,
        is_italic: bool,
        movie: Option<Arc<SwfMovie>>,
    ) -> Option<Font<'gc>> {
        let query = FontQuery::new(font_type, name.to_owned(), is_bold, is_italic);
        if let Some(font) = self.global_fonts.find(&query) {
            return Some(font);
        }
        if let Some(movie) = movie
            && let Some(library) = self.library_for_movie(movie)
        {
            if let Some((_, font)) = library.character_by_export_name(&WString::from_utf8(name)) {
                // Exporting a font seems to override font lookup completely.
                return if let Character::Font(font) = font {
                    Some(font)
                } else {
                    None
                };
            }
            if let Some(font) = library.fonts.find(&query) {
                return Some(font);
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
struct FontMap<'gc>(FnvHashMap<FontQuery, Font<'gc>>);

impl<'gc> FontMap<'gc> {
    pub fn register(&mut self, font: Font<'gc>) {
        self.register_with_descriptor(font, font.descriptor());
    }

    pub fn register_with_descriptor(&mut self, font: Font<'gc>, descriptor: &FontDescriptor) {
        self.0
            .entry(FontQuery::from_descriptor(font.font_type(), descriptor))
            .or_insert(font);
    }

    pub fn get(&self, font_query: &FontQuery) -> Option<Font<'gc>> {
        self.0.get(font_query).copied()
    }

    pub fn find(&self, font_query: &FontQuery) -> Option<Font<'gc>> {
        // The order here is specific, and tested in `tests/swfs/fonts/embed_matching/fallback_preferences`

        // Exact match
        if let Some(font) = self.get(font_query) {
            return Some(font);
        }

        let is_italic = font_query.is_italic;
        let is_bold = font_query.is_bold;

        let mut fallback_query = font_query.clone();
        if is_italic ^ is_bold {
            // If one is set (but not both), then try upgrading to bold italic...
            fallback_query.is_bold = true;
            fallback_query.is_italic = true;
            if let Some(font) = self.get(&fallback_query) {
                return Some(font);
            }

            // and then downgrading to regular
            fallback_query.is_bold = false;
            fallback_query.is_italic = false;
            if let Some(font) = self.get(&fallback_query) {
                return Some(font);
            }

            // and then finally whichever one we don't have set
            fallback_query.is_bold = !is_bold;
            fallback_query.is_italic = !is_italic;
            if let Some(font) = self.get(&fallback_query) {
                return Some(font);
            }
        } else {
            // We don't have an exact match and we were either looking for regular or bold-italic

            if is_italic && is_bold {
                // Do we have regular? (unless we already looked for it)
                fallback_query.is_bold = false;
                fallback_query.is_italic = false;
                if let Some(font) = self.get(&fallback_query) {
                    return Some(font);
                }
            }

            // Do we have bold?
            fallback_query.is_bold = true;
            fallback_query.is_italic = false;
            if let Some(font) = self.get(&fallback_query) {
                return Some(font);
            }

            // Do we have italic?
            fallback_query.is_bold = false;
            fallback_query.is_italic = true;
            if let Some(font) = self.get(&fallback_query) {
                return Some(font);
            }

            if !is_bold && !is_italic {
                // Do we have bold italic? (unless we already looked for it)
                fallback_query.is_bold = true;
                fallback_query.is_italic = true;
                if let Some(font) = self.get(&fallback_query) {
                    return Some(font);
                }
            }
        }

        None
    }

    pub fn all(&self) -> Vec<Font<'gc>> {
        self.0.values().copied().collect()
    }
}
