use std::cell::RefCell;

use crate::backend::audio::SoundHandle;
use crate::display_object::{
    Avm1Button, Avm2Button, BitmapClass, EditText, Graphic, MorphShape, MovieClip, Text, Video,
};
use crate::font::Font;
use crate::tag_utils::SwfSlice;
use gc_arena::{Collect, GcCell};
use ruffle_render::bitmap::{BitmapHandle, BitmapSize};
use swf::DefineBitsLossless;

#[derive(Clone, Collect, Debug)]
#[collect(no_drop)]
pub enum Character<'gc> {
    EditText(EditText<'gc>),
    Graphic(Graphic<'gc>),
    MovieClip(MovieClip<'gc>),
    Bitmap {
        #[collect(require_static)]
        compressed: CompressedBitmap,
        /// A lazily constructed GPU handle, used when performing fills with this bitmap
        #[collect(require_static)]
        handle: RefCell<Option<BitmapHandle>>,
        /// The bitmap class set by `SymbolClass` - this is used when we instantaite
        /// a `Bitmap` displayobject.
        avm2_bitmapdata_class: GcCell<'gc, BitmapClass<'gc>>,
    },
    Avm1Button(Avm1Button<'gc>),
    Avm2Button(Avm2Button<'gc>),
    Font(Font<'gc>),
    MorphShape(MorphShape<'gc>),
    Text(Text<'gc>),
    Sound(#[collect(require_static)] SoundHandle),
    Video(Video<'gc>),
    BinaryData(SwfSlice),
}

/// Holds a bitmap from an SWF tag, plus the decoded width/height.
/// We avoid decompressing the image until it's actually needed - some pathological SWFS
/// like 'House' have thousands of highly-compressed (mostly empty) bitmaps, which can
/// take over 10GB of ram if we decompress them all during preloading.
#[derive(Clone, Debug)]
pub enum CompressedBitmap {
    Jpeg {
        data: SwfSlice,
        alpha: Option<SwfSlice>,
        width: u16,
        height: u16,
    },
    // Used when stored data don't exactly correspond to source SWF data
    // (for example, when we prepend JpegTables)
    OwnedJpeg {
        data: Vec<u8>,
        alpha: Option<SwfSlice>,
        width: u16,
        height: u16,
    },
    Lossless {
        data: SwfSlice,
        // note: the data inside the tag is ignored.
        tag: DefineBitsLossless<'static>,
    },
}

impl CompressedBitmap {
    pub fn size(&self) -> BitmapSize {
        match self {
            CompressedBitmap::Jpeg { width, height, .. } => BitmapSize {
                width: *width,
                height: *height,
            },
            CompressedBitmap::OwnedJpeg { width, height, .. } => BitmapSize {
                width: *width,
                height: *height,
            },
            CompressedBitmap::Lossless { tag, .. } => BitmapSize {
                width: tag.width,
                height: tag.height,
            },
        }
    }
    pub fn decode(&self) -> Result<ruffle_render::bitmap::Bitmap, ruffle_render::error::Error> {
        match self {
            CompressedBitmap::Jpeg { data, alpha, .. } => {
                ruffle_render::utils::decode_define_bits_jpeg(data.as_ref(), alpha.as_ref().map(|s| s.as_ref()))
            }
            CompressedBitmap::OwnedJpeg { data, alpha, .. } => {
                ruffle_render::utils::decode_define_bits_jpeg(data, alpha.as_ref().map(|s| s.as_ref()))
            }
            CompressedBitmap::Lossless { data, tag } => {
                ruffle_render::utils::decode_define_bits_lossless(data.as_ref(), tag)
            }
        }
    }
}
