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
    Bytes {
        width: u32,
        height: u32,
        channels: u32,
        bytes: Vec<u8>,
    },
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
