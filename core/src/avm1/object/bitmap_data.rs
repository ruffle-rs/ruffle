use crate::add_field_accessors;
use crate::avm1::error::Error;
use crate::avm1::{Object, ScriptObject, TObject, Value};
use crate::impl_custom_object_without_set;
use gc_arena::{Collect, GcCell, MutationContext};

use crate::avm1::activation::Activation;
use crate::avm1::object::color_transform_object::ColorTransformObject;
use crate::backend::render::{BitmapHandle, RenderBackend};
use downcast_rs::__std::fmt::Formatter;
use rand::prelude::SmallRng;
use rand::Rng;
use std::fmt;

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

    pub fn to_premultiplied_alpha(&self, transparency: bool) -> Color {
        // This has some accuracy issues with some alpha values

        let old_alpha = if transparency { self.alpha() } else { 255 };

        let a = old_alpha as f64 / 255.0;

        let r = (self.red() as f64 * a).round() as u8;
        let g = (self.green() as f64 * a).round() as u8;
        let b = (self.blue() as f64 * a).round() as u8;

        Color::argb(old_alpha, r, g, b)
    }

    pub fn to_un_multiplied_alpha(&self) -> Color {
        let a = self.alpha() as f64 / 255.0;

        let r = (self.red() as f64 / a).round() as u8;
        let g = (self.green() as f64 / a).round() as u8;
        let b = (self.blue() as f64 / a).round() as u8;

        Color::argb(self.alpha(), r, g, b)
    }

    pub fn argb(alpha: u8, red: u8, green: u8, blue: u8) -> Color {
        Color(((alpha as i32) << 24) | (red as i32) << 16 | (green as i32) << 8 | (blue as i32))
    }

    pub fn with_alpha(&self, alpha: u8) -> Color {
        Color::argb(alpha, self.red(), self.green(), self.blue())
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

pub struct ChannelOptions(pub u32);

impl ChannelOptions {
    pub fn alpha(&self) -> bool {
        self.0 & 8 == 8
    }
    pub fn red(&self) -> bool {
        self.0 & 1 == 1
    }
    pub fn green(&self) -> bool {
        self.0 & 2 == 2
    }
    pub fn blue(&self) -> bool {
        self.0 & 4 == 4
    }

    pub fn rgb() -> Self {
        (1 | 2 | 4).into()
    }
}

impl From<u32> for ChannelOptions {
    fn from(v: u32) -> Self {
        Self { 0: v }
    }
}

#[derive(Clone, Collect, Default, Debug)]
#[collect(no_drop)]
pub struct BitmapData {
    /// The pixels in the bitmap, stored as a array of pre-multiplied ARGB colour values
    pub pixels: Vec<Color>,
    dirty: bool,
    width: u32,
    height: u32,
    transparency: bool,

    bitmap_handle: Option<BitmapHandle>,
}

impl BitmapData {
    pub fn init_pixels(&mut self, width: u32, height: u32, fill_color: i32, transparency: bool) {
        self.width = width;
        self.height = height;
        self.transparency = transparency;
        self.pixels = vec![
            Color(fill_color).to_premultiplied_alpha(self.transparency());
            (width * height) as usize
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
            match renderer.register_bitmap_raw(self.width(), self.height(), self.pixels_rgba()) {
                Ok(bitmap_handle) => self.bitmap_handle = Some(bitmap_handle),
                Err(err) => log::warn!("Failed to register raw bitmap for BitmapData: {:?}", err),
            }
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

    pub fn set_pixels(&mut self, pixels: Vec<Color>) {
        self.pixels = pixels;
    }

    pub fn pixels_rgba(&self) -> Vec<u8> {
        let mut output = Vec::new();

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
        if x > self.width() || y > self.height() {
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
        if !self.is_point_in_bounds(x, y) {
            0
        } else {
            self.get_pixel32(x, y).with_alpha(0x0).into()
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

        let mut pending = Vec::new();
        pending.push((x, y));

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
        rng: &mut SmallRng,
        _seed: u32,
        low: u8,
        high: u8,
        channel_options: ChannelOptions,
        gray_scale: bool,
    ) {
        for x in 0..self.width() {
            for y in 0..self.height() {
                let pixel_color = if gray_scale {
                    let gray = rng.gen_range(low..high);
                    Color::argb(
                        if channel_options.alpha() {
                            rng.gen_range(low..high)
                        } else {
                            255
                        },
                        gray,
                        gray,
                        gray,
                    )
                } else {
                    Color::argb(
                        if channel_options.alpha() {
                            rng.gen_range(low..high)
                        } else {
                            255
                        },
                        if channel_options.red() {
                            rng.gen_range(low..high)
                        } else {
                            0
                        },
                        if channel_options.green() {
                            rng.gen_range(low..high)
                        } else {
                            0
                        },
                        if channel_options.blue() {
                            rng.gen_range(low..high)
                        } else {
                            0
                        },
                    )
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
                        // Alpha
                        8 => 24,
                        // red
                        1 => 16,
                        // green
                        2 => 8,
                        // blue
                        4 => 0,
                        _ => 0,
                    };

                    let source_part = (source_color >> channel_shift) & 0xFF;

                    let result_color: u32 = match dest_channel {
                        // Alpha
                        8 => (original_color & 0x00FFFFFF) | source_part << 24,
                        // red
                        1 => (original_color & 0xFF00FFFF) | source_part << 16,
                        // green
                        2 => (original_color & 0xFFFF00FF) | source_part << 8,
                        // blue
                        4 => (original_color & 0xFFFFFF00) | source_part,
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
        color_transform: ColorTransformObject,
    ) {
        for x in min_x..end_x.min(self.width()) {
            for y in min_y..end_y.min(self.height()) {
                let color = self
                    .get_pixel_raw(x, y)
                    .unwrap_or_else(|| 0.into())
                    .to_un_multiplied_alpha();

                let alpha = ((color.alpha() as f32 * color_transform.get_alpha_multiplier() as f32)
                    + color_transform.get_alpha_offset() as f32) as u8;
                let red = ((color.red() as f32 * color_transform.get_red_multiplier() as f32)
                    + color_transform.get_red_offset() as f32) as u8;
                let green = ((color.green() as f32 * color_transform.get_green_multiplier() as f32)
                    + color_transform.get_green_offset() as f32) as u8;
                let blue = ((color.blue() as f32 * color_transform.get_blue_multiplier() as f32)
                    + color_transform.get_blue_offset() as f32) as u8;

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
        let mut min_x = Option::<i32>::None;
        let mut max_x = Option::<i32>::None;
        let mut min_y = Option::<i32>::None;
        let mut max_y = Option::<i32>::None;

        for x in 0..self.width() {
            for y in 0..self.height() {
                let pixel_raw: i32 = self.get_pixel_raw(x, y).unwrap_or_else(|| 0.into()).into();
                let color_matches = if find_color {
                    (pixel_raw & mask) == color
                } else {
                    (pixel_raw & mask) != color
                };

                if color_matches {
                    if (x as i32) < min_x.unwrap_or(self.width() as i32) {
                        min_x = Some(x as i32)
                    }
                    if (x as i32) > max_x.unwrap_or(-1) {
                        max_x = Some(x as i32 + 1)
                    }

                    if (y as i32) < min_y.unwrap_or(self.height() as i32) {
                        min_y = Some(y as i32)
                    }
                    if (y as i32) > max_y.unwrap_or(-1) {
                        max_y = Some(y as i32 + 1)
                    }
                }
            }
        }

        let min_x = min_x.unwrap_or(0);
        let min_y = min_y.unwrap_or(0);
        let max_x = max_x.unwrap_or(0);
        let max_y = max_y.unwrap_or(0);

        let x = min_x as u32;
        let y = min_y as u32;
        let w = (max_x - min_x) as u32;
        let h = (max_y - min_y) as u32;

        (x, y, w, h)
    }
}

/// A BitmapData
#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct BitmapDataObject<'gc>(GcCell<'gc, BitmapDataData<'gc>>);

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct BitmapDataData<'gc> {
    /// The underlying script object.
    base: ScriptObject<'gc>,
    data: GcCell<'gc, BitmapData>,
    disposed: bool,
}

impl fmt::Debug for BitmapDataObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let this = self.0.read();
        f.debug_struct("BitmapData")
            .field("data", &this.data)
            .finish()
    }
}

impl<'gc> BitmapDataObject<'gc> {
    add_field_accessors!(
        [disposed, bool, get => disposed],
        [data, GcCell<'gc, BitmapData>, get => bitmap_data],
    );

    pub fn empty_object(gc_context: MutationContext<'gc, '_>, proto: Option<Object<'gc>>) -> Self {
        BitmapDataObject(GcCell::allocate(
            gc_context,
            BitmapDataData {
                base: ScriptObject::object(gc_context, proto),
                disposed: false,
                data: GcCell::allocate(gc_context, BitmapData::default()),
            },
        ))
    }

    pub fn dispose(&self, gc_context: MutationContext<'gc, '_>) {
        self.bitmap_data().write(gc_context).dispose();
        self.0.write(gc_context).disposed = true;
    }
}

impl<'gc> TObject<'gc> for BitmapDataObject<'gc> {
    impl_custom_object_without_set!(base);

    fn set(
        &self,
        name: &str,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<(), Error<'gc>> {
        let base = self.0.read().base;
        base.internal_set(
            name,
            value,
            activation,
            (*self).into(),
            Some(activation.context.avm1.prototypes.bitmap_data),
        )
    }

    fn as_bitmap_data_object(&self) -> Option<BitmapDataObject<'gc>> {
        Some(*self)
    }

    fn create_bare_object(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        _this: Object<'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        Ok(BitmapDataObject::empty_object(
            activation.context.gc_context,
            Some(activation.context.avm1.prototypes.bitmap_data),
        )
        .into())
    }
}
