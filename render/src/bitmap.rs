use crate::backend::RenderBackend;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct BitmapHandle(pub usize);

#[derive(Clone, Debug)]
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

/// Decoded bitmap data from an SWF tag.
#[derive(Clone, Debug)]
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
        let expected_len = width as usize * height as usize * format.bytes_per_pixel();
        if data.len() != expected_len {
            log::warn!(
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
        if self.format == BitmapFormat::Rgb {
            self.data = self
                .data
                .chunks_exact(3)
                .flat_map(|rgb| [rgb[0], rgb[1], rgb[2], 255])
                .collect();
            self.format = BitmapFormat::Rgba;
        }
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
}

impl From<Bitmap> for Vec<i32> {
    fn from(bitmap: Bitmap) -> Self {
        match bitmap.format {
            BitmapFormat::Rgb => bitmap
                .data
                .chunks_exact(3)
                .map(|chunk| {
                    let red = chunk[0];
                    let green = chunk[1];
                    let blue = chunk[2];
                    i32::from_le_bytes([blue, green, red, 0xFF])
                })
                .collect(),
            BitmapFormat::Rgba => bitmap
                .data
                .chunks_exact(4)
                .map(|chunk| {
                    let red = chunk[0];
                    let green = chunk[1];
                    let blue = chunk[2];
                    let alpha = chunk[3];
                    i32::from_le_bytes([blue, green, red, alpha])
                })
                .collect(),
        }
    }
}

/// The pixel format of the bitmap data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BitmapFormat {
    /// 24-bit RGB.
    Rgb,

    /// 32-bit RGBA with premultiplied alpha.
    Rgba,
}

impl BitmapFormat {
    #[inline]
    pub fn bytes_per_pixel(self) -> usize {
        match self {
            BitmapFormat::Rgb => 3,
            BitmapFormat::Rgba => 4,
        }
    }
}
