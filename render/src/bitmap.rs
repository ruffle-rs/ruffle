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
                expected_len,
                data.len(),
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

    pub fn to_rgb(mut self) -> Self {
        // Converts this bitmap to RGB, if it is not already.
        match self.format {
            BitmapFormat::Rgb => {} // no-op
            BitmapFormat::Rgba => unreachable!("Can't convert RGBA Bitmap to RGB"),
            BitmapFormat::Yuv420p => {
                let luma_len = (self.width * self.height) as usize;
                let chroma_len = (self.chroma_width() * self.chroma_height()) as usize;

                let y = &self.data[0..luma_len];
                let u = &self.data[luma_len..luma_len + chroma_len];
                let v = &self.data[luma_len + chroma_len..luma_len + 2 * chroma_len];

                self.data = yuv420_to_rgba(y, u, v, self.width as usize)
                    .chunks_exact(4)
                    .flat_map(|rgba| [rgba[0], rgba[1], rgba[2]])
                    .collect();
            }
            BitmapFormat::Yuva420p => unreachable!("Can't convert YUVA Bitmap to RGB"),
        }

        self.format = BitmapFormat::Rgb;

        self
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

#[inline]
fn intersection_same_coordinate_system(
    (r1_x_min, r1_y_min, r1_x_max, r1_y_max): (i32, i32, i32, i32),
    (r2_x_min, r2_y_min, r2_x_max, r2_y_max): (i32, i32, i32, i32),
) -> (i32, i32, i32, i32) {
    // To guard against 'min' being larger than 'max'.
    let r1_x_min = r1_x_min.min(r1_x_max);
    let r1_y_min = r1_y_min.min(r1_y_max);
    let r2_x_min = r2_x_min.min(r2_x_max);
    let r2_y_min = r2_y_min.min(r2_y_max);

    // First part of intersection.
    let r3_x_min = r1_x_min.max(r2_x_min);
    let r3_y_min = r1_y_min.max(r2_y_min);
    let r3_x_max = r1_x_max.min(r2_x_max);
    let r3_y_max = r1_y_max.min(r2_y_max);

    // In case of no overlap.
    let r3_x_min = r3_x_min.min(r3_x_max);
    let r3_y_min = r3_y_min.min(r3_y_max);

    (r3_x_min, r3_y_min, r3_x_max, r3_y_max)
}

#[inline]
fn translate_region(
    (r_x_min, r_y_min, r_x_max, r_y_max): (i32, i32, i32, i32),
    (trans_x, trans_y): (i32, i32),
) -> (i32, i32, i32, i32) {
    (
        r_x_min + trans_x,
        r_y_min + trans_y,
        r_x_max + trans_x,
        r_y_max + trans_y,
    )
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

    /// Clamps this PixelRegion to a theoretical overlap of another PixelRegion,
    /// referring to "overlapping pixels" (such as a copy destination vs copy source),
    /// in such a way that only pixels that are valid for both PixelRegions are valid.
    ///
    /// The other PixelRegion is also clamped to reflect the same overlap.
    ///
    /// The overlap of two regions starts at `self_point` on `self`, and `other_point` on `other`,
    /// and is at most `size` big.
    ///
    /// The overlap does not actually need to happen on the same coordinate plane,
    /// for example -1,-1 on this may be 100,100 on other, with an overlap region of 5x5.
    /// As long as both textures can fit that, that's considered an overlap.
    /// However, since -1,-1 is outside of the valid area on the first region,
    /// the overlap actually happens at 0,0 and 101,101 for a size of 4x4.
    pub fn clamp_with_intersection(
        &mut self,
        self_point: (i32, i32),
        other_point: (i32, i32),
        size: (i32, i32),
        other: &mut PixelRegion,
    ) {
        // Translate both regions to same coordinate system.

        let r1 = (
            self.x_min as i32,
            self.y_min as i32,
            self.x_max as i32,
            self.y_max as i32,
        );
        let r2 = (
            other.x_min as i32,
            other.y_min as i32,
            other.x_max as i32,
            other.y_max as i32,
        );

        let r1_trans = translate_region(r1, (-self_point.0, -self_point.1));
        let r2_trans = translate_region(r2, (-other_point.0, -other_point.1));

        // Intersection.

        let inters = intersection_same_coordinate_system(
            intersection_same_coordinate_system(r1_trans, r2_trans),
            (0, 0, size.0, size.1),
        );

        // Translate the intersection back.

        let r1_result = translate_region(inters, self_point);
        let r2_result = translate_region(inters, other_point);

        // Ensure empty results yield (0, 0, 0, 0).

        let is_empty = inters.0 == inters.2 || inters.1 == inters.3;

        let r1_result = if is_empty { (0, 0, 0, 0) } else { r1_result };
        let r2_result = if is_empty { (0, 0, 0, 0) } else { r2_result };

        // Mutate.

        self.x_min = r1_result.0 as u32;
        self.y_min = r1_result.1 as u32;
        self.x_max = r1_result.2 as u32;
        self.y_max = r1_result.3 as u32;

        other.x_min = r2_result.0 as u32;
        other.y_min = r2_result.1 as u32;
        other.x_max = r2_result.2 as u32;
        other.y_max = r2_result.3 as u32;
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

#[cfg(test)]
mod test {
    use super::PixelRegion;

    #[test]
    fn clamp_with_intersection() {
        fn test(
            mut a: PixelRegion,
            mut b: PixelRegion,
            a_point: (i32, i32),
            b_point: (i32, i32),
            size: (i32, i32),
            expected_a: PixelRegion,
            expected_b: PixelRegion,
        ) {
            a.clamp_with_intersection(a_point, b_point, size, &mut b);

            assert_eq!(expected_a, a, "a (self) region should match");
            assert_eq!(expected_b, b, "b (other) region should match");
        }

        test(
            PixelRegion::for_whole_size(10, 10),
            PixelRegion::for_whole_size(10, 10),
            (0, 0),
            (0, 0),
            (5, 5),
            PixelRegion::for_region_i32(0, 0, 5, 5),
            PixelRegion::for_region_i32(0, 0, 5, 5),
        );

        test(
            PixelRegion::for_whole_size(10, 10),
            PixelRegion::for_whole_size(150, 150),
            (-1, -1),
            (100, 100),
            (5, 5),
            PixelRegion::for_region_i32(0, 0, 4, 4),
            PixelRegion::for_region_i32(101, 101, 4, 4),
        );

        test(
            PixelRegion::for_whole_size(10, 10),
            PixelRegion::for_whole_size(150, 150),
            (-1, -1),
            (100, 100),
            (15, 15),
            PixelRegion::for_region_i32(0, 0, 10, 10),
            PixelRegion::for_region_i32(101, 101, 10, 10),
        );

        test(
            PixelRegion::for_region(10, 10, 20, 20),
            PixelRegion::for_whole_size(150, 150),
            (15, 5),
            (0, 0),
            (15, 15),
            PixelRegion::for_region_i32(15, 10, 15, 10),
            PixelRegion::for_region_i32(0, 5, 15, 10),
        );

        test(
            PixelRegion::for_whole_size(800, 600),
            PixelRegion::for_whole_size(200, 40),
            (400, 440),
            (40, 0),
            (40, 40),
            PixelRegion::for_region_i32(400, 440, 40, 40),
            PixelRegion::for_region_i32(40, 0, 40, 40),
        );

        test(
            PixelRegion::for_whole_size(240, 180),
            PixelRegion::for_whole_size(238, 164),
            (-1, 0),
            (0, 0),
            (240, 180),
            PixelRegion::for_region_i32(0, 0, 237, 164),
            PixelRegion::for_region_i32(1, 0, 237, 164),
        );

        test(
            PixelRegion::for_whole_size(10, 10),
            PixelRegion::for_whole_size(10, 10),
            (15, 0),
            (0, 15),
            (100, 100),
            PixelRegion::for_region_i32(0, 0, 0, 0),
            PixelRegion::for_region_i32(0, 0, 0, 0),
        );
    }
}
