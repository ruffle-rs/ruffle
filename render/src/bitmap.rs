use gc_arena::Collect;
use h263_rs_yuv::bt601::yuv420_to_rgba;
use std::fmt::Debug;
use std::sync::Arc;

use downcast_rs::{impl_downcast, Downcast};
use swf::{Rectangle, Twips};

use crate::backend::RenderBackend;

#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
pub struct BitmapHandle(pub Arc<dyn BitmapHandleImpl>);

pub trait BitmapHandleImpl: Downcast + Debug {}
impl_downcast!(BitmapHandleImpl);

/// Info returned by the `register_bitmap` methods.
#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
pub struct BitmapInfo {
    pub handle: BitmapHandle,
    pub width: u16,
    pub height: u16,
}

#[derive(Copy, Clone, Debug)]
pub struct BitmapSize {
    pub width: u16,
    pub height: u16,
}

/// An object that returns a bitmap given an ID.
///
/// This is used by render backends to get the bitmap used in a bitmap fill.
/// For movie libraries, this will return the bitmap with the given character ID.
pub trait BitmapSource {
    fn bitmap_size(&self, id: u16) -> Option<BitmapSize>;
    fn bitmap_handle(&self, id: u16, renderer: &mut dyn RenderBackend) -> Option<BitmapHandle>;
}

pub type RgbaBufRead<'a> = Box<dyn FnOnce(&[u8], u32) + 'a>;

pub trait SyncHandle: Downcast + Debug {
    /// Retrieves the rendered pixels from a previous `render_offscreen` call
    fn retrieve_offscreen_texture(
        self: Box<Self>,
        with_rgba: RgbaBufRead,
    ) -> Result<(), crate::error::Error>;
}
impl_downcast!(SyncHandle);

impl Clone for Box<dyn SyncHandle> {
    fn clone(&self) -> Box<dyn SyncHandle> {
        panic!("SyncHandle should have been consumed before clone() is called!")
    }
}

/// Decoded bitmap data from an SWF tag.
#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
pub struct Bitmap {
    width: u32,
    height: u32,
    format: BitmapFormat,
    data: Vec<u8>,
}

impl Bitmap {
    /// Ensures that `data` is the correct size for the given `width` and `height`.
    pub fn new(width: u32, height: u32, format: BitmapFormat, mut data: Vec<u8>) -> Self {
        // If the size is incorrect, either we screwed up or the decoder screwed up.
        let expected_len = format.length_for_size(width as usize, height as usize);
        if data.len() != expected_len {
            tracing::warn!(
                "Incorrect bitmap data size, expected {} bytes, got {}",
                data.len(),
                expected_len
            );
            // Truncate or zero pad to the expected size.
            data.resize(expected_len, 0);
        }
        Self {
            width,
            height,
            format,
            data,
        }
    }

    pub fn to_rgba(mut self) -> Self {
        // Converts this bitmap to RGBA, if it is not already.
        match self.format {
            BitmapFormat::Rgb => {
                self.data = self
                    .data
                    .chunks_exact(3)
                    .flat_map(|rgb| [rgb[0], rgb[1], rgb[2], 255])
                    .collect();
            }
            BitmapFormat::Rgba => {} // no-op
            BitmapFormat::Yuv420p => {
                let luma_len = (self.width * self.height) as usize;
                let chroma_len = (self.chroma_width() * self.chroma_height()) as usize;

                let y = &self.data[0..luma_len];
                let u = &self.data[luma_len..luma_len + chroma_len];
                let v = &self.data[luma_len + chroma_len..luma_len + 2 * chroma_len];

                self.data = yuv420_to_rgba(y, u, v, self.width as usize);
            }
            BitmapFormat::Yuva420p => {
                let luma_len = (self.width * self.height) as usize;
                let chroma_len = (self.chroma_width() * self.chroma_height()) as usize;

                let y = &self.data[0..luma_len];
                let u = &self.data[luma_len..luma_len + chroma_len];
                let v = &self.data[luma_len + chroma_len..luma_len + chroma_len + chroma_len];
                let a = &self.data[luma_len + 2 * chroma_len..2 * luma_len + 2 * chroma_len];

                let rgba = yuv420_to_rgba(y, u, v, self.width as usize);

                // RGB components need to be clamped to alpha to avoid invalid premultiplied colors
                self.data = rgba
                    .chunks_exact(4)
                    .zip(a)
                    .flat_map(|(rgba, a)| [rgba[0].min(*a), rgba[1].min(*a), rgba[2].min(*a), *a])
                    .collect()
            }
        }

        self.format = BitmapFormat::Rgba;

        self
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn chroma_width(&self) -> u32 {
        match self.format {
            BitmapFormat::Yuv420p | BitmapFormat::Yuva420p => (self.width + 1) / 2,
            _ => unreachable!("Can't get chroma width for non-YUV bitmap"),
        }
    }

    pub fn chroma_height(&self) -> u32 {
        match self.format {
            BitmapFormat::Yuv420p | BitmapFormat::Yuva420p => (self.height + 1) / 2,
            _ => unreachable!("Can't get chroma height for non-YUV bitmap"),
        }
    }

    #[inline]
    pub fn format(&self) -> BitmapFormat {
        self.format
    }

    #[inline]
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    #[inline]
    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    pub fn as_colors(&self) -> impl Iterator<Item = i32> + '_ {
        let chunks = match self.format {
            BitmapFormat::Rgb => self.data.chunks_exact(3),
            BitmapFormat::Rgba => self.data.chunks_exact(4),
            _ => unimplemented!(
                "Can't iterate over non-RGB(A) bitmaps as colors, convert with `to_rgba` first"
            ),
        };
        chunks.map(|chunk| {
            let red = chunk[0];
            let green = chunk[1];
            let blue = chunk[2];
            let alpha = chunk.get(3).copied().unwrap_or(0xFF);
            i32::from_le_bytes([blue, green, red, alpha])
        })
    }
}

/// The pixel format of the bitmap data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BitmapFormat {
    /// 24-bit RGB.
    Rgb,

    /// 32-bit RGBA with premultiplied alpha.
    Rgba,

    /// planar YUV 420
    Yuv420p,

    /// planar YUV 420, premultiplied with alpha (RGB channels are to be clamped after conversion)
    Yuva420p,
}

impl BitmapFormat {
    #[inline]
    pub fn length_for_size(self, width: usize, height: usize) -> usize {
        match self {
            BitmapFormat::Rgb => width * height * 3,
            BitmapFormat::Rgba => width * height * 4,
            BitmapFormat::Yuv420p => width * height + ((width + 1) / 2) * ((height + 1) / 2) * 2,
            BitmapFormat::Yuva420p => {
                width * height * 2 + ((width + 1) / 2) * ((height + 1) / 2) * 2
            }
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct PixelRegion {
    pub x_min: u32,
    pub y_min: u32,
    pub x_max: u32,
    pub y_max: u32,
}

impl PixelRegion {
    pub fn encompassing_twips(a: (Twips, Twips), b: (Twips, Twips)) -> Self {
        // Figure out what our two ranges are
        let (min, max) = ((a.0.min(b.0), a.1.min(b.1)), (a.0.max(b.0), a.1.max(b.1)));

        // Increase max by one pixel as we've calculated the *encompassed* max
        let max = (
            max.0 + Twips::from_pixels_i32(1),
            max.1 + Twips::from_pixels_i32(1),
        );

        // Make sure we're never going below 0
        Self {
            x_min: min.0.to_pixels().floor().max(0.0) as u32,
            y_min: min.1.to_pixels().floor().max(0.0) as u32,
            x_max: max.0.to_pixels().ceil().max(0.0) as u32,
            y_max: max.1.to_pixels().ceil().max(0.0) as u32,
        }
    }

    pub fn for_region_i32(x: i32, y: i32, width: i32, height: i32) -> Self {
        let a = (x, y);
        let b = (x.saturating_add(width), y.saturating_add(height));
        let (min, max) = ((a.0.min(b.0), a.1.min(b.1)), (a.0.max(b.0), a.1.max(b.1)));

        Self {
            x_min: min.0.max(0) as u32,
            y_min: min.1.max(0) as u32,
            x_max: max.0.max(0) as u32,
            y_max: max.1.max(0) as u32,
        }
    }

    pub fn for_region(x: u32, y: u32, width: u32, height: u32) -> Self {
        let a = (x, y);
        let b = (x.saturating_add(width), y.saturating_add(height));
        let (min, max) = ((a.0.min(b.0), a.1.min(b.1)), (a.0.max(b.0), a.1.max(b.1)));

        Self {
            x_min: min.0,
            y_min: min.1,
            x_max: max.0,
            y_max: max.1,
        }
    }

    pub fn encompassing_pixels_i32(a: (i32, i32), b: (i32, i32)) -> Self {
        Self::encompassing_pixels(
            (a.0.max(0) as u32, a.1.max(0) as u32),
            (b.0.max(0) as u32, b.1.max(0) as u32),
        )
    }

    pub fn encompassing_pixels(a: (u32, u32), b: (u32, u32)) -> Self {
        // Figure out what our two ranges are
        let (min, max) = ((a.0.min(b.0), a.1.min(b.1)), (a.0.max(b.0), a.1.max(b.1)));

        // Increase max by one pixel as we've calculated the *encompassed* max
        let max = (max.0.saturating_add(1), max.1.saturating_add(1));

        Self {
            x_min: min.0,
            y_min: min.1,
            x_max: max.0,
            y_max: max.1,
        }
    }

    pub fn for_whole_size(width: u32, height: u32) -> Self {
        Self {
            x_min: 0,
            y_min: 0,
            x_max: width,
            y_max: height,
        }
    }

    pub fn for_pixel(x: u32, y: u32) -> Self {
        Self {
            x_min: x,
            y_min: y,
            x_max: x + 1,
            y_max: y + 1,
        }
    }

    pub fn clamp(&mut self, width: u32, height: u32) {
        self.x_min = self.x_min.min(width);
        self.y_min = self.y_min.min(height);
        self.x_max = self.x_max.min(width);
        self.y_max = self.y_max.min(height);
    }

    pub fn union(&mut self, other: PixelRegion) {
        self.x_min = self.x_min.min(other.x_min);
        self.y_min = self.y_min.min(other.y_min);
        self.x_max = self.x_max.max(other.x_max);
        self.y_max = self.y_max.max(other.y_max);
    }

    pub fn encompass(&mut self, x: u32, y: u32) {
        self.x_min = self.x_min.min(x);
        self.y_min = self.y_min.min(y);
        self.x_max = self.x_max.max(x + 1);
        self.y_max = self.y_max.max(y + 1);
    }

    pub fn intersects(&self, other: PixelRegion) -> bool {
        self.x_min <= other.x_max
            && self.x_max >= other.x_min
            && self.y_min <= other.y_max
            && self.y_max >= other.y_min
    }

    pub fn width(&self) -> u32 {
        self.x_max - self.x_min
    }

    pub fn height(&self) -> u32 {
        self.y_max - self.y_min
    }
}

impl From<Rectangle<Twips>> for PixelRegion {
    fn from(value: Rectangle<Twips>) -> Self {
        Self {
            x_min: value.x_min.to_pixels().floor().max(0.0) as u32,
            y_min: value.y_min.to_pixels().floor().max(0.0) as u32,
            x_max: value.x_max.to_pixels().ceil().max(0.0) as u32,
            y_max: value.y_max.to_pixels().ceil().max(0.0) as u32,
        }
    }
}
