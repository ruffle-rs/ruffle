use std::borrow::Cow;

use crate::backend::RawTexture;
use crate::bitmap::BitmapHandle;
use crate::pixel_bender::PixelBenderType;

#[derive(Debug, Clone, PartialEq)]
pub enum PixelBenderShaderArgument<'a> {
    ImageInput {
        index: u8,
        channels: u8,
        name: String,
        texture: Option<ImageInputTexture<'a>>,
    },
    ValueInput {
        index: u8,
        value: PixelBenderType,
    },
}

/// An image input.
///
/// This accepts both an owned BitmapHandle, and a borrowed texture
/// (used when applying a filter to a texture that we don't have
/// ownership of, and therefore cannot construct a BitmapHandle for).
#[derive(Debug, Clone)]
pub enum ImageInputTexture<'a> {
    Bitmap(BitmapHandle),
    TextureRef(&'a dyn RawTexture),
    Floats {
        width: u32,
        height: u32,
        data: FloatPixelData,
    },
}

#[derive(Debug, Clone)]
pub enum FloatPixelData {
    R(Vec<[f32; 1]>),
    Rg(Vec<[f32; 2]>),
    Rgb(Vec<[f32; 3]>),
    Rgba(Vec<[f32; 4]>),
}

impl FloatPixelData {
    pub fn channel_count(&self) -> u32 {
        match self {
            Self::R(_) => 1,
            Self::Rg(_) => 2,
            Self::Rgb(_) => 3,
            Self::Rgba(_) => 4,
        }
    }

    pub fn padded_data(&self) -> Cow<'_, [f32]> {
        match self {
            Self::R(r) => Cow::Borrowed(r.as_flattened()),
            Self::Rg(rg) => Cow::Borrowed(rg.as_flattened()),
            // We're going to be using an Rgba32Float texture, so we need to pad the bytes
            // with zeros for the alpha channel. The PixelBender code will only ever try to
            // use the first 3 channels (since it was compiled with a 3-channel input),
            // so it doesn't matter what value we choose here.
            Self::Rgb(rgb) => {
                let rgba = rgb
                    .iter()
                    .copied()
                    .flat_map(|[r, g, b]| [r, g, b, 0.0])
                    .collect();

                Cow::Owned(rgba)
            }
            Self::Rgba(rgba) => Cow::Borrowed(rgba.as_flattened()),
        }
    }
}

impl PartialEq for ImageInputTexture<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Bitmap(self_bitmap), Self::Bitmap(other_bitmap)) => self_bitmap == other_bitmap,
            (Self::TextureRef(self_texture), Self::TextureRef(other_texture)) => {
                self_texture.equals(*other_texture)
            }
            _ => false,
        }
    }
}

impl From<BitmapHandle> for ImageInputTexture<'_> {
    fn from(b: BitmapHandle) -> Self {
        ImageInputTexture::Bitmap(b)
    }
}

impl<'a> From<&'a dyn RawTexture> for ImageInputTexture<'a> {
    fn from(t: &'a dyn RawTexture) -> Self {
        ImageInputTexture::TextureRef(t)
    }
}
