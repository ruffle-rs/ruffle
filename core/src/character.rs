use std::cell::OnceCell;

use crate::backend::audio::SoundHandle;
use crate::binary_data::BinaryData;
use crate::display_object::{
    Avm1Button, Avm2Button, BitmapClass, EditText, Graphic, MorphShape, MovieClip, Text, Video,
};
use crate::font::Font;
use gc_arena::barrier::unlock;
use gc_arena::lock::Lock;
use gc_arena::{Collect, Finalization, Gc, Mutation};
use ruffle_render::backend::RenderBackend;
use ruffle_render::bitmap::{Bitmap as RenderBitmap, BitmapHandle, BitmapSize};
use ruffle_render::error::Error as RenderError;
use swf::DefineBitsLossless;

#[derive(Copy, Clone, Collect, Debug)]
#[collect(no_drop)]
pub enum Character<'gc> {
    EditText(EditText<'gc>),
    Graphic(Graphic<'gc>),
    MovieClip(MovieClip<'gc>),
    Bitmap(Gc<'gc, BitmapCharacter<'gc>>),
    Avm1Button(Avm1Button<'gc>),
    Avm2Button(Avm2Button<'gc>),
    Font(Font<'gc>),
    MorphShape(MorphShape<'gc>),
    Text(Text<'gc>),
    Sound(#[collect(require_static)] SoundHandle),
    Video(Video<'gc>),
    BinaryData(Gc<'gc, BinaryData>),
}

impl<'gc> Character<'gc> {
    /// Whether an instance of this character - or anything else outside the
    /// library that defines it - still reaches the definition data the
    /// character shares with its instances.
    ///
    /// Only meaningful during finalization, once marking has completed. The
    /// character templates in a movie's library are deliberately not traced
    /// from the library (see [`crate::library::MovieLibrary`]), so if their
    /// shared data has been marked, it was marked through an instance that is
    /// on the display list or held by ActionScript.
    ///
    /// Bitmaps, fonts, sounds and binary data do not count: their instances
    /// copy or own what they need at creation and never look back at the
    /// library, so they do not need it kept alive.
    pub fn has_reachable_instances(&self, fc: &Finalization<'gc>) -> bool {
        match self {
            Character::EditText(o) => o.shared_data_is_reachable(fc),
            Character::Graphic(o) => o.shared_data_is_reachable(fc),
            Character::MovieClip(o) => o.shared_data_is_reachable(fc),
            Character::Avm1Button(o) => o.shared_data_is_reachable(fc),
            Character::Avm2Button(o) => o.shared_data_is_reachable(fc),
            Character::MorphShape(o) => o.shared_data_is_reachable(fc),
            Character::Text(o) => o.shared_data_is_reachable(fc),
            Character::Video(o) => o.shared_data_is_reachable(fc),
            Character::Bitmap(_)
            | Character::Font(_)
            | Character::Sound(_)
            | Character::BinaryData(_) => false,
        }
    }
}

#[derive(Collect, Debug)]
#[collect(no_drop)]
pub struct BitmapCharacter<'gc> {
    #[collect(require_static)]
    compressed: CompressedBitmap,
    /// A lazily constructed GPU handle, used when performing fills with this bitmap
    #[collect(require_static)]
    handle: OnceCell<BitmapHandle>,
    /// The bitmap class set by `SymbolClass` - this is used when we instantaite
    /// a `Bitmap` displayobject.
    avm2_class: Lock<BitmapClass<'gc>>,
}

impl<'gc> BitmapCharacter<'gc> {
    pub fn new(compressed: CompressedBitmap) -> Self {
        Self {
            compressed,
            handle: OnceCell::default(),
            avm2_class: Lock::new(BitmapClass::NoSubclass),
        }
    }

    pub fn compressed(&self) -> &CompressedBitmap {
        &self.compressed
    }

    /// Whether this bitmap has already been uploaded to the render backend.
    ///
    /// An uploaded bitmap also costs GPU memory, which is only reclaimed when
    /// the character itself is dropped.
    pub fn is_uploaded(&self) -> bool {
        self.handle.get().is_some()
    }

    pub fn avm2_class(&self) -> BitmapClass<'gc> {
        self.avm2_class.get()
    }

    pub fn set_avm2_class(this: Gc<'gc, Self>, bitmap_class: BitmapClass<'gc>, mc: &Mutation<'gc>) {
        unlock!(Gc::write(mc, this), Self, avm2_class).set(bitmap_class);
    }

    pub fn bitmap_handle(
        &self,
        backend: &mut dyn RenderBackend,
    ) -> Result<BitmapHandle, RenderError> {
        // FIXME - use `OnceCell::get_or_try_init` when stabilized.
        if let Some(handle) = self.handle.get() {
            return Ok(handle.clone());
        }
        let decoded = self.compressed.decode()?;
        let new_handle = backend.register_bitmap(decoded)?;
        // FIXME - do we ever want to release this handle, to avoid taking up GPU memory?
        self.handle.set(new_handle.clone()).unwrap();
        Ok(new_handle)
    }
}

/// Holds a bitmap from an SWF tag, plus the decoded width/height.
/// We avoid decompressing the image until it's actually needed - some pathological SWFS
/// like 'House' have thousands of highly-compressed (mostly empty) bitmaps, which can
/// take over 10GB of ram if we decompress them all during preloading.
#[derive(Clone, Debug)]
pub enum CompressedBitmap {
    Jpeg {
        data: Vec<u8>,
        alpha: Option<Vec<u8>>,
        width: u32,
        height: u32,
    },
    Lossless(DefineBitsLossless<'static>),
}

impl CompressedBitmap {
    pub fn size(&self) -> BitmapSize {
        match self {
            CompressedBitmap::Jpeg { width, height, .. } => BitmapSize {
                width: *width,
                height: *height,
            },
            CompressedBitmap::Lossless(define_bits_lossless) => BitmapSize {
                width: define_bits_lossless.width.into(),
                height: define_bits_lossless.height.into(),
            },
        }
    }
    /// Bytes of still-compressed image data held in memory for this bitmap.
    pub fn source_bytes(&self) -> usize {
        match self {
            CompressedBitmap::Jpeg { data, alpha, .. } => {
                data.len() + alpha.as_ref().map_or(0, |alpha| alpha.len())
            }
            CompressedBitmap::Lossless(define_bits_lossless) => define_bits_lossless.data.len(),
        }
    }

    pub fn decode(&self) -> Result<RenderBitmap<'static>, RenderError> {
        match self {
            CompressedBitmap::Jpeg {
                data,
                alpha,
                width: _,
                height: _,
            } => ruffle_render::utils::decode_define_bits_jpeg(data, alpha.as_deref()),
            CompressedBitmap::Lossless(define_bits_lossless) => {
                ruffle_render::utils::decode_define_bits_lossless(define_bits_lossless)
            }
        }
    }
}
