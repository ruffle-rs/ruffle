use gc_arena::Collect;

use crate::avm2::{Object as Avm2Object, Value as Avm2Value};
use crate::backend::render::{BitmapHandle, RenderBackend};
use crate::bitmap::color_transform_params::ColorTransformParams;
use crate::bitmap::turbulence::Turbulence;
use bitflags::bitflags;
use downcast_rs::__std::fmt::Formatter;
use std::ops::Range;

/// An implementation of the Lehmer/Park-Miller random number generator
/// Uses the fixed parameters m = 2,147,483,647 and a = 16,807
pub struct LehmerRng {
    x: u32,
}

impl LehmerRng {
    pub fn with_seed(seed: u32) -> Self {
        Self { x: seed }
    }

    /// Generate the next value in the sequence via the following formula
    /// X_(k+1) = a * X_k mod m
    pub fn gen(&mut self) -> u32 {
        self.x = ((self.x as u64).overflowing_mul(16_807).0 % 2_147_483_647) as u32;
        self.x
    }

    pub fn gen_range(&mut self, rng: Range<u8>) -> u8 {
        rng.start + (self.gen() % ((rng.end - rng.start) as u32 + 1)) as u8
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Collect)]
#[collect(no_drop)]
pub struct Color(i32);

impl Color {
    pub fn blue(&self) -> u8 {
        (self.0 & 0xFF) as u8
    }

    pub fn green(&self) -> u8 {
        ((self.0 >> 8) & 0xFF) as u8
    }

    pub fn red(&self) -> u8 {
        ((self.0 >> 16) & 0xFF) as u8
    }

    pub fn alpha(&self) -> u8 {
        ((self.0 >> 24) & 0xFF) as u8
    }

    pub fn to_premultiplied_alpha(self, transparency: bool) -> Self {
        // This has some accuracy issues with some alpha values

        let old_alpha = if transparency { self.alpha() } else { 255 };

        let a = old_alpha as f64 / 255.0;

        let r = (self.red() as f64 * a).round() as u8;
        let g = (self.green() as f64 * a).round() as u8;
        let b = (self.blue() as f64 * a).round() as u8;

        Self::argb(old_alpha, r, g, b)
    }

    pub fn to_un_multiplied_alpha(self) -> Self {
        let a = self.alpha() as f64 / 255.0;

        let r = (self.red() as f64 / a).round() as u8;
        let g = (self.green() as f64 / a).round() as u8;
        let b = (self.blue() as f64 / a).round() as u8;

        Self::argb(self.alpha(), r, g, b)
    }

    pub fn argb(alpha: u8, red: u8, green: u8, blue: u8) -> Self {
        Self(i32::from_le_bytes([blue, green, red, alpha]))
    }

    pub fn with_alpha(&self, alpha: u8) -> Self {
        Self::argb(alpha, self.red(), self.green(), self.blue())
    }

    pub fn blend_over(&self, source: &Self) -> Self {
        let sa = source.alpha();

        let r = source.red() + ((self.red() as u16 * (255 - sa as u16)) >> 8) as u8;
        let g = source.green() + ((self.green() as u16 * (255 - sa as u16)) >> 8) as u8;
        let b = source.blue() + ((self.blue() as u16 * (255 - sa as u16)) >> 8) as u8;
        let a = source.alpha() + ((self.alpha() as u16 * (255 - sa as u16)) >> 8) as u8;
        Self::argb(a, r, g, b)
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:#x}", self.0))
    }
}

impl From<Color> for i32 {
    fn from(c: Color) -> Self {
        c.0
    }
}

impl From<Color> for u32 {
    fn from(c: Color) -> Self {
        c.0 as u32
    }
}

impl From<i32> for Color {
    fn from(i: i32) -> Self {
        Color(i)
    }
}

bitflags! {
    pub struct ChannelOptions: u8 {
        const RED = 1 << 0;
        const GREEN = 1 << 1;
        const BLUE = 1 << 2;
        const ALPHA = 1 << 3;
        const RGB = Self::RED.bits | Self::GREEN.bits | Self::BLUE.bits;
    }
}

#[derive(Clone, Collect, Default, Debug)]
#[collect(no_drop)]
pub struct BitmapData<'gc> {
    /// The pixels in the bitmap, stored as a array of pre-multiplied ARGB colour values
    pixels: Vec<Color>,
    dirty: bool,
    width: u32,
    height: u32,
    transparency: bool,

    /// The bitmap handle for this data.
    ///
    /// This is lazily initialized; a value of `None` indicates that
    /// initialization has not yet happened.
    bitmap_handle: Option<BitmapHandle>,

    /// The AVM2 side of this `BitmapData`.
    ///
    /// AVM1 cannot retrieve `BitmapData` back from the display object tree, so
    /// this does not need to hold an AVM1 object.
    avm2_object: Option<Avm2Object<'gc>>,
}

impl<'gc> BitmapData<'gc> {
    pub fn init_pixels(&mut self, width: u32, height: u32, transparency: bool, fill_color: i32) {
        self.width = width;
        self.height = height;
        self.transparency = transparency;
        self.pixels = vec![
            Color(fill_color).to_premultiplied_alpha(self.transparency());
            width as usize * height as usize
        ];
        self.dirty = true;
    }

    pub fn dispose(&mut self) {
        self.width = 0;
        self.height = 0;
        self.pixels.clear();
        self.dirty = true;
    }

    pub fn bitmap_handle(&mut self, renderer: &mut dyn RenderBackend) -> Option<BitmapHandle> {
        if self.bitmap_handle.is_none() {
            let bitmap_handle =
                renderer.register_bitmap_raw(self.width(), self.height(), self.pixels_rgba());
            if let Err(e) = &bitmap_handle {
                log::warn!("Failed to register raw bitmap for BitmapData: {:?}", e);
            }
            self.bitmap_handle = bitmap_handle.ok();
        }

        self.bitmap_handle
    }

    pub fn transparency(&self) -> bool {
        self.transparency
    }

    pub fn set_transparency(&mut self, transparency: bool) {
        self.transparency = transparency;
    }

    pub fn dirty(&self) -> bool {
        self.dirty
    }

    pub fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
    }

    pub fn pixels(&self) -> &[Color] {
        &self.pixels
    }

    pub fn set_pixels(&mut self, width: u32, height: u32, transparency: bool, pixels: Vec<Color>) {
        self.width = width;
        self.height = height;
        self.transparency = transparency;
        self.pixels = pixels;
        self.dirty = true;
    }

    pub fn pixels_rgba(&self) -> Vec<u8> {
        // TODO: This could have been implemented as follows:
        //
        // self.pixels
        //     .iter()
        //     .flat_map(|p| [p.red(), p.green(), p.blue(), p.alpha()])
        //     .collect()
        //
        // But currently Rust emits suboptimal code in that case. For now we use
        // `Vec::with_capacity` manually to avoid unnecessary re-allocations.

        let mut output = Vec::with_capacity(self.pixels.len() * 4);
        for p in &self.pixels {
            output.extend_from_slice(&[p.red(), p.green(), p.blue(), p.alpha()])
        }
        output
    }

    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn is_point_in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.width() as i32 && y >= 0 && y < self.height() as i32
    }

    pub fn get_pixel_raw(&self, x: u32, y: u32) -> Option<Color> {
        if x >= self.width() || y >= self.height() {
            return None;
        }

        self.pixels.get((x + y * self.width()) as usize).copied()
    }

    pub fn get_pixel32(&self, x: i32, y: i32) -> Color {
        self.get_pixel_raw(x as u32, y as u32)
            .map(|f| f.to_un_multiplied_alpha())
            .unwrap_or_else(|| 0.into())
    }

    pub fn get_pixel(&self, x: i32, y: i32) -> i32 {
        if self.is_point_in_bounds(x, y) {
            self.get_pixel32(x, y).with_alpha(0x0).into()
        } else {
            0
        }
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        let current_alpha = self.get_pixel_raw(x, y).map(|p| p.alpha()).unwrap_or(0);
        self.set_pixel32(x as i32, y as i32, color.with_alpha(current_alpha));
    }

    pub fn set_pixel32_raw(&mut self, x: u32, y: u32, color: Color) {
        let width = self.width();
        self.pixels[(x + y * width) as usize] = color;
        self.dirty = true;
    }

    pub fn set_pixel32(&mut self, x: i32, y: i32, color: Color) {
        if self.is_point_in_bounds(x, y) {
            self.set_pixel32_raw(
                x as u32,
                y as u32,
                color.to_premultiplied_alpha(self.transparency()),
            )
        }
    }

    pub fn fill_rect(&mut self, x: u32, y: u32, width: u32, height: u32, color: Color) {
        for x_offset in 0..width {
            for y_offset in 0..height {
                self.set_pixel32((x + x_offset) as i32, (y + y_offset) as i32, color)
            }
        }
    }

    pub fn flood_fill(&mut self, x: u32, y: u32, replace_color: Color) {
        let expected_color = self.get_pixel_raw(x, y).unwrap_or_else(|| 0.into());

        let mut pending = vec![(x, y)];

        while !pending.is_empty() {
            if let Some((x, y)) = pending.pop() {
                if let Some(old_color) = self.get_pixel_raw(x, y) {
                    if old_color == expected_color {
                        if x > 0 {
                            pending.push((x - 1, y));
                        }
                        if y > 0 {
                            pending.push((x, y - 1));
                        }
                        if x < self.width() - 1 {
                            pending.push((x + 1, y))
                        }
                        if y < self.height() - 1 {
                            pending.push((x, y + 1));
                        }
                        self.set_pixel32_raw(x, y, replace_color);
                    }
                }
            }
        }
    }

    pub fn noise(
        &mut self,
        seed: i32,
        low: u8,
        high: u8,
        channel_options: ChannelOptions,
        gray_scale: bool,
    ) {
        let true_seed = if seed <= 0 {
            (-seed + 1) as u32
        } else {
            seed as u32
        };

        let mut rng = LehmerRng::with_seed(true_seed);

        for y in 0..self.height() {
            for x in 0..self.width() {
                let pixel_color = if gray_scale {
                    let gray = rng.gen_range(low..high);
                    let alpha = if channel_options.contains(ChannelOptions::ALPHA) {
                        rng.gen_range(low..high)
                    } else {
                        255
                    };

                    Color::argb(alpha, gray, gray, gray)
                } else {
                    let r = if channel_options.contains(ChannelOptions::RED) {
                        rng.gen_range(low..high)
                    } else {
                        0
                    };

                    let g = if channel_options.contains(ChannelOptions::GREEN) {
                        rng.gen_range(low..high)
                    } else {
                        0
                    };

                    let b = if channel_options.contains(ChannelOptions::BLUE) {
                        rng.gen_range(low..high)
                    } else {
                        0
                    };

                    let a = if channel_options.contains(ChannelOptions::ALPHA) {
                        rng.gen_range(low..high)
                    } else {
                        255
                    };

                    Color::argb(a, r, g, b)
                };

                self.set_pixel32_raw(x, y, pixel_color);
            }
        }
    }

    pub fn copy_channel(
        &mut self,
        dest_point: (u32, u32),
        src_rect: (u32, u32, u32, u32),
        source_bitmap: &Self,
        source_channel: i32,
        dest_channel: i32,
    ) {
        let (min_x, min_y) = dest_point;
        let (src_min_x, src_min_y, src_max_x, src_max_y) = src_rect;

        for x in src_min_x.max(0)..src_max_x.min(source_bitmap.width()) {
            for y in src_min_y.max(0)..src_max_y.min(source_bitmap.height()) {
                if self.is_point_in_bounds((x + min_x) as i32, (y + min_y) as i32) {
                    let original_color: u32 = self
                        .get_pixel_raw((x + min_x) as u32, (y + min_y) as u32)
                        .unwrap_or_else(|| 0.into())
                        .into();
                    let source_color: u32 = source_bitmap
                        .get_pixel_raw(x, y)
                        .unwrap_or_else(|| 0.into())
                        .into();

                    let channel_shift: u32 = match source_channel {
                        // red
                        1 => 16,
                        // green
                        2 => 8,
                        // blue
                        4 => 0,
                        // alpha
                        8 => 24,
                        _ => 0,
                    };

                    let source_part = (source_color >> channel_shift) & 0xFF;

                    let result_color: u32 = match dest_channel {
                        // red
                        1 => (original_color & 0xFF00FFFF) | source_part << 16,
                        // green
                        2 => (original_color & 0xFFFF00FF) | source_part << 8,
                        // blue
                        4 => (original_color & 0xFFFFFF00) | source_part,
                        // alpha
                        8 => (original_color & 0x00FFFFFF) | source_part << 24,
                        _ => original_color,
                    };

                    self.set_pixel32_raw(
                        (x + min_x) as u32,
                        (y + min_y) as u32,
                        (result_color as i32).into(),
                    );
                }
            }
        }
    }

    pub fn color_transform(
        &mut self,
        min_x: u32,
        min_y: u32,
        end_x: u32,
        end_y: u32,
        color_transform: &ColorTransformParams,
    ) {
        for x in min_x..end_x.min(self.width()) {
            for y in min_y..end_y.min(self.height()) {
                let color = self
                    .get_pixel_raw(x, y)
                    .unwrap_or_else(|| 0.into())
                    .to_un_multiplied_alpha();

                let alpha = ((color.alpha() as f32 * color_transform.alpha_multiplier as f32)
                    + color_transform.alpha_offset as f32) as u8;
                let red = ((color.red() as f32 * color_transform.red_multiplier as f32)
                    + color_transform.red_offset as f32) as u8;
                let green = ((color.green() as f32 * color_transform.green_multiplier as f32)
                    + color_transform.green_offset as f32) as u8;
                let blue = ((color.blue() as f32 * color_transform.blue_multiplier as f32)
                    + color_transform.blue_offset as f32) as u8;

                self.set_pixel32_raw(
                    x,
                    y,
                    Color::argb(alpha, red, green, blue)
                        .to_premultiplied_alpha(self.transparency()),
                )
            }
        }
    }

    pub fn color_bounds_rect(
        &self,
        find_color: bool,
        mask: i32,
        color: i32,
    ) -> (u32, u32, u32, u32) {
        let mut min_x = self.width();
        let mut max_x = 0;
        let mut min_y = self.height();
        let mut max_y = 0;

        for x in 0..self.width() {
            for y in 0..self.height() {
                let pixel_raw: i32 = self.get_pixel_raw(x, y).unwrap().into();
                let color_matches = if find_color {
                    (pixel_raw & mask) == color
                } else {
                    (pixel_raw & mask) != color
                };

                if color_matches {
                    min_x = min_x.min(x);
                    max_x = max_x.max(x);
                    min_y = min_y.min(y);
                    max_y = max_y.max(y);
                }
            }
        }

        // Flash treats a match of (0, 0) alone as none.
        if max_x > 0 || max_y > 0 {
            let x = min_x;
            let y = min_y;
            let w = max_x - min_x + 1;
            let h = max_y - min_y + 1;
            (x, y, w, h)
        } else {
            (0, 0, 0, 0)
        }
    }

    pub fn copy_pixels(
        &mut self,
        source_bitmap: &Self,
        src_rect: (i32, i32, i32, i32),
        dest_point: (i32, i32),
        alpha_source: Option<(&Self, (i32, i32), bool)>,
    ) {
        let (src_min_x, src_min_y, src_width, src_height) = src_rect;
        let (dest_min_x, dest_min_y) = dest_point;

        for src_y in src_min_y..(src_min_y + src_height) {
            for src_x in src_min_x..(src_min_x + src_width) {
                let dest_x = src_x - src_min_x + dest_min_x;
                let dest_y = src_y - src_min_y + dest_min_y;

                if !source_bitmap.is_point_in_bounds(src_x, src_y)
                    || !self.is_point_in_bounds(dest_x, dest_y)
                {
                    continue;
                }

                let source_color = source_bitmap
                    .get_pixel_raw(src_x as u32, src_y as u32)
                    .unwrap();

                let mut dest_color = self.get_pixel_raw(dest_x as u32, dest_y as u32).unwrap();

                if let Some((alpha_bitmap, (alpha_min_x, alpha_min_y), merge_alpha)) = alpha_source
                {
                    let alpha_x = src_x - src_min_x + alpha_min_x;
                    let alpha_y = src_y - src_min_y + alpha_min_y;

                    if alpha_bitmap.transparency
                        && !alpha_bitmap.is_point_in_bounds(alpha_x, alpha_y)
                    {
                        continue;
                    }

                    let final_alpha = if alpha_bitmap.transparency {
                        let a = alpha_bitmap
                            .get_pixel_raw(alpha_x as u32, alpha_y as u32)
                            .unwrap()
                            .alpha();

                        if source_bitmap.transparency {
                            ((a as u16 * source_color.alpha() as u16) >> 8) as u8
                        } else {
                            a
                        }
                    } else if source_bitmap.transparency {
                        source_color.alpha()
                    } else {
                        255
                    };

                    // there could be a faster or more accurate way to do this,
                    // (without converting to floats and back, twice),
                    // but for now this should suffice
                    let intermediate_color = source_color
                        .to_un_multiplied_alpha()
                        .with_alpha(final_alpha)
                        .to_premultiplied_alpha(true);

                    // there are some interesting conditions in the following
                    // lines, these are a result of comparing the output in
                    // many parameter combinations with that of Adobe's player,
                    // and finding patterns in the differences.
                    dest_color = if merge_alpha || !self.transparency {
                        dest_color.blend_over(&intermediate_color)
                    } else {
                        intermediate_color
                    };
                } else {
                    dest_color = if source_bitmap.transparency && !self.transparency {
                        dest_color.blend_over(&source_color)
                    } else {
                        source_color
                    };
                }

                self.set_pixel32_raw(dest_x as u32, dest_y as u32, dest_color);
            }
        }
    }

    pub fn merge(
        &mut self,
        source_bitmap: &Self,
        src_rect: (i32, i32, i32, i32),
        dest_point: (i32, i32),
        rgba_mult: (i32, i32, i32, i32),
    ) {
        let (src_min_x, src_min_y, src_width, src_height) = src_rect;
        let (dest_min_x, dest_min_y) = dest_point;

        for src_y in src_min_y..(src_min_y + src_height) {
            for src_x in src_min_x..(src_min_x + src_width) {
                let dest_x = src_x - src_min_x + dest_min_x;
                let dest_y = src_y - src_min_y + dest_min_y;

                if !self.is_point_in_bounds(dest_x, dest_y)
                    || !source_bitmap.is_point_in_bounds(src_x, src_y)
                {
                    continue;
                }

                let source_color = source_bitmap
                    .get_pixel_raw(src_x as u32, src_y as u32)
                    .unwrap()
                    .to_un_multiplied_alpha();

                let dest_color = self
                    .get_pixel_raw(dest_x as u32, dest_y as u32)
                    .unwrap()
                    .to_un_multiplied_alpha();

                let red_mult = rgba_mult.0.clamp(0, 256) as u16;
                let green_mult = rgba_mult.1.clamp(0, 256) as u16;
                let blue_mult = rgba_mult.2.clamp(0, 256) as u16;
                let alpha_mult = rgba_mult.3.clamp(0, 256) as u16;

                let red = (source_color.red() as u16 * red_mult
                    + dest_color.red() as u16 * (256 - red_mult))
                    / 256;
                let green = (source_color.green() as u16 * green_mult
                    + dest_color.green() as u16 * (256 - green_mult))
                    / 256;
                let blue = (source_color.blue() as u16 * blue_mult
                    + dest_color.blue() as u16 * (256 - blue_mult))
                    / 256;
                let alpha = (source_color.alpha() as u16 * alpha_mult
                    + dest_color.alpha() as u16 * (256 - alpha_mult))
                    / 256;

                let mix_color = Color::argb(alpha as u8, red as u8, green as u8, blue as u8);

                self.set_pixel32_raw(
                    dest_x as u32,
                    dest_y as u32,
                    mix_color.to_premultiplied_alpha(self.transparency),
                );
            }
        }
    }

    // Unlike `copy_channel` and `copy_pixels`, this function seems to
    // operate "in-place" if the source bitmap is the same object as `self`.
    // This means that we can't resolve this aliasing issue in Rust by a
    // simple clone in the caller. Instead, if the `source_bitmap` parameter
    // is `None`, it means that `self` should be used as source as well.
    pub fn palette_map(
        &mut self,
        source_bitmap: Option<&Self>,
        src_rect: (i32, i32, i32, i32),
        dest_point: (i32, i32),
        channel_arrays: ([u32; 256], [u32; 256], [u32; 256], [u32; 256]),
    ) {
        let (src_min_x, src_min_y, src_width, src_height) = src_rect;
        let (dest_min_x, dest_min_y) = dest_point;

        for src_y in src_min_y..(src_min_y + src_height) {
            for src_x in src_min_x..(src_min_x + src_width) {
                let dest_x = src_x - src_min_x + dest_min_x;
                let dest_y = src_y - src_min_y + dest_min_y;

                if !self.is_point_in_bounds(dest_x, dest_y)
                    || !source_bitmap
                        .unwrap_or(self)
                        .is_point_in_bounds(src_x, src_y)
                {
                    continue;
                }

                let source_color = source_bitmap
                    .unwrap_or(self)
                    .get_pixel_raw(src_x as u32, src_y as u32)
                    .unwrap()
                    .to_un_multiplied_alpha();

                let r = channel_arrays.0[source_color.red() as usize];
                let g = channel_arrays.1[source_color.green() as usize];
                let b = channel_arrays.2[source_color.blue() as usize];
                let a = channel_arrays.3[source_color.alpha() as usize];

                let sum = u32::wrapping_add(u32::wrapping_add(r, g), u32::wrapping_add(b, a));
                let mix_color = Color(sum as i32).to_premultiplied_alpha(true);

                self.set_pixel32_raw(dest_x as u32, dest_y as u32, mix_color);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn perlin_noise(
        &mut self,
        base: (f64, f64),
        num_octaves: usize,
        random_seed: i64,
        stitch: bool,
        fractal_noise: bool,
        channel_options: ChannelOptions,
        grayscale: bool,
        offsets: Vec<(f64, f64)>, // must contain `num_octaves` values
    ) {
        let turb = Turbulence::from_seed(random_seed);

        for y in 0..self.height() {
            for x in 0..self.width() {
                let px = x as f64;
                let py = y as f64;

                let mut noise = [0.0; 4];

                // grayscale mode is different enough to warrant its own branch
                if grayscale {
                    noise[0] = turb.turbulence(
                        0,
                        (px, py),
                        (1.0 / base.0, 1.0 / base.1),
                        num_octaves,
                        fractal_noise,
                        stitch,
                        (0.0, 0.0),
                        (self.width as f64, self.height as f64),
                        &offsets,
                    );

                    noise[1] = noise[0];
                    noise[2] = noise[0];

                    noise[3] = if channel_options.contains(ChannelOptions::ALPHA) {
                        turb.turbulence(
                            1,
                            (px, py),
                            (1.0 / base.0, 1.0 / base.1),
                            num_octaves,
                            fractal_noise,
                            stitch,
                            (0.0, 0.0),
                            (self.width as f64, self.height as f64),
                            &offsets,
                        )
                    } else {
                        1.0
                    };
                } else {
                    // Flash seems to pass the `color_channel` parameter to `turbulence`
                    // somewhat strangely. It's not always r=0, g=1, b=2, a=3; instead,
                    // it skips incrementing the parameter after channels that are
                    // not included in `channel_options`.
                    let mut channel = 0;

                    for (c, noise_c) in noise.iter_mut().enumerate() {
                        // this will work both in fractal_sum and turbulence "modes",
                        // because of the saturating conversion to u8
                        *noise_c = if c == 3 { 1.0 } else { -1.0 };

                        // SAFETY: `c` is always in 0..4, so `1 << c` is a valid `ChannelOptions`.
                        let c = unsafe { ChannelOptions::from_bits_unchecked(1 << c) };
                        if channel_options.contains(c) {
                            *noise_c = turb.turbulence(
                                channel,
                                (px, py),
                                (1.0 / base.0, 1.0 / base.1),
                                num_octaves,
                                fractal_noise,
                                stitch,
                                (0.0, 0.0),
                                (self.width as f64, self.height as f64),
                                &offsets,
                            );
                            channel += 1;
                        }
                    }
                }

                let mut color = [0_u8; 4];
                for chan in 0..4 {
                    // This is precisely how Adobe Flash converts the -1..1 or 0..1 floats to u8.
                    // Please don't touch, it was difficult to figure out the exact method. :)
                    color[chan] = (if fractal_noise {
                        // Yes, the + 0.5 for correct (nearest) rounding is done before the division by 2.0,
                        // making it technically less correct (I think), but this is how it is!
                        ((noise[chan] * 255.0 + 255.0) + 0.5) / 2.0
                    } else {
                        (noise[chan] * 255.0) + 0.5
                    }) as u8;
                }

                if !self.transparency {
                    color[3] = 255;
                }

                self.set_pixel32_raw(x, y, Color::argb(color[3], color[0], color[1], color[2]));
            }
        }
    }

    pub fn scroll(&mut self, x: i32, y: i32) {
        let width = self.width() as i32;
        let height = self.height() as i32;

        if (x == 0 && y == 0) || x.abs() >= width || y.abs() >= height {
            return; // no-op
        }

        // since this is an "in-place copy", we have to iterate from bottom to top
        // when scrolling downwards - so if y is positive
        let reverse_y = y > 0;
        // and if only scrolling horizontally, we have to iterate from right to left
        // when scrolling right - so if x is positive
        let reverse_x = y == 0 && x > 0;

        // iteration ranges to use as source for the copy, from is inclusive, to is exclusive
        let y_from = if reverse_y { height - y - 1 } else { -y };
        let y_to = if reverse_y { -1 } else { height };
        let dy = if reverse_y { -1 } else { 1 };

        let x_from = if reverse_x {
            // we know x > 0
            width - x - 1
        } else {
            // x can be any sign
            (-x).max(0)
        };
        let x_to = if reverse_x { -1 } else { width.min(width - x) };
        let dx = if reverse_x { -1 } else { 1 };

        let mut src_y = y_from;
        while src_y != y_to {
            let mut src_x = x_from;
            while src_x != x_to {
                let color = self.get_pixel_raw(src_x as u32, src_y as u32).unwrap();
                self.set_pixel32_raw((src_x + x) as u32, (src_y + y) as u32, color);
                src_x += dx;
            }
            src_y += dy;
        }
    }

    /// Compare two BitmapData objects.
    /// Returns `None` if the bitmaps are equivalent.
    pub fn compare(bitmap: &Self, other: &Self) -> Option<Self> {
        // This function expects that the two bitmaps have the same dimensions.
        // TODO: Relax this assumption and return a special value instead?
        debug_assert_eq!(bitmap.width, other.width);
        debug_assert_eq!(bitmap.height, other.height);

        let mut different = false;
        let pixels = bitmap
            .pixels
            .iter()
            .zip(&other.pixels)
            .map(|(bitmap_pixel, other_pixel)| {
                let bitmap_pixel = bitmap_pixel.to_un_multiplied_alpha();
                let other_pixel = other_pixel.to_un_multiplied_alpha();
                if bitmap_pixel == other_pixel {
                    Color::argb(0, 0, 0, 0)
                } else if bitmap_pixel.with_alpha(0) != other_pixel.with_alpha(0) {
                    different = true;
                    Color::argb(
                        0xff,
                        bitmap_pixel.red().wrapping_sub(other_pixel.red()),
                        bitmap_pixel.green().wrapping_sub(other_pixel.green()),
                        bitmap_pixel.blue().wrapping_sub(other_pixel.blue()),
                    )
                } else {
                    different = true;
                    Color::argb(
                        bitmap_pixel.alpha().wrapping_sub(other_pixel.alpha()),
                        0xff,
                        0xff,
                        0xff,
                    )
                }
            })
            .collect();

        if different {
            Some(Self {
                pixels,
                dirty: false,
                width: bitmap.width,
                height: bitmap.height,
                transparency: true,
                bitmap_handle: None,
                avm2_object: None,
            })
        } else {
            None
        }
    }

    pub fn object2(&self) -> Avm2Value<'gc> {
        self.avm2_object
            .map(|o| o.into())
            .unwrap_or(Avm2Value::Undefined)
    }

    pub fn init_object2(&mut self, object: Avm2Object<'gc>) {
        self.avm2_object = Some(object)
    }
}
