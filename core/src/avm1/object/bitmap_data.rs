use crate::add_field_accessors;
use crate::avm1::error::Error;
use crate::avm1::{Object, ScriptObject, TObject, Value};
use crate::impl_custom_object_without_set;
use gc_arena::{Collect, GcCell, MutationContext};

use crate::avm1::activation::Activation;
use downcast_rs::__std::fmt::Formatter;
use std::fmt;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Collect)]
#[collect(no_drop)]
pub struct Color(i32);

impl Color {
    pub fn get_blue(&self) -> u8 {
        (self.0 & 0xFF) as u8
    }

    pub fn get_green(&self) -> u8 {
        ((self.0 >> 8) & 0xFF) as u8
    }

    pub fn get_red(&self) -> u8 {
        ((self.0 >> 16) & 0xFF) as u8
    }

    pub fn get_alpha(&self) -> u8 {
        ((self.0 >> 24) & 0xFF) as u8
    }

    pub fn to_premultiplied_alpha(&self, transparency: bool) -> Color {
        // This has some accuracy issues with some alpha values

        let old_alpha = if transparency { self.get_alpha() } else { 255 };

        let a = old_alpha as f64 / 255.0;

        let r = (self.get_red() as f64 * a).round() as u8;
        let g = (self.get_green() as f64 * a).round() as u8;
        let b = (self.get_blue() as f64 * a).round() as u8;

        Color::argb(old_alpha, r, g, b)
    }

    pub fn to_un_multiplied_alpha(&self) -> Color {
        let a = self.get_alpha() as f64 / 255.0;

        let r = (self.get_red() as f64 / a).round() as u8;
        let g = (self.get_green() as f64 / a).round() as u8;
        let b = (self.get_blue() as f64 / a).round() as u8;

        Color::argb(self.get_alpha(), r, g, b)
    }

    pub fn argb(alpha: u8, red: u8, green: u8, blue: u8) -> Color {
        Color(((alpha as i32) << 24) | (red as i32) << 16 | (green as i32) << 8 | (blue as i32))
    }

    pub fn with_alpha(&self, alpha: u8) -> Color {
        Color::argb(alpha, self.get_red(), self.get_green(), self.get_blue())
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

impl From<i32> for Color {
    fn from(i: i32) -> Self {
        Color(i)
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

    /// The pixels in the bitmap, stored as a array of pre-multiplied ARGB colour values
    pixels: Vec<Color>,

    width: u32,
    height: u32,
    transparency: bool,

    disposed: bool,
}

impl fmt::Debug for BitmapDataObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let this = self.0.read();
        f.debug_struct("BitmapData")
            .field("pixels", &this.pixels)
            .field("width", &this.width)
            .field("height", &this.height)
            .field("transparency", &this.transparency)
            .finish()
    }
}

impl<'gc> BitmapDataObject<'gc> {
    add_field_accessors!([set_transparency, get_transparency, transparency, bool],);

    pub fn empty_object(gc_context: MutationContext<'gc, '_>, proto: Option<Object<'gc>>) -> Self {
        BitmapDataObject(GcCell::allocate(
            gc_context,
            BitmapDataData {
                base: ScriptObject::object(gc_context, proto),
                pixels: Vec::new(),
                width: 0,
                height: 0,
                transparency: true,
                disposed: false,
            },
        ))
    }

    pub fn get_pixels_rgba(&self) -> Vec<u8> {
        self.0
            .read()
            .pixels
            .iter()
            .flat_map(|p| vec![p.get_red(), p.get_green(), p.get_blue(), p.get_alpha()])
            .collect()
    }

    pub fn get_disposed(&self) -> bool {
        self.0.read().disposed
    }

    pub fn get_pixels(&self) -> Vec<Color> {
        self.0.read().pixels.clone()
    }

    pub fn set_pixels(&self, gc_context: MutationContext<'gc, '_>, new_pixels: Vec<Color>) {
        self.0.write(gc_context).pixels = new_pixels
    }

    pub fn init_pixels(
        &self,
        gc_context: MutationContext<'gc, '_>,
        width: u32,
        height: u32,
        fill_color: i32,
    ) {
        self.0.write(gc_context).width = width;
        self.0.write(gc_context).height = height;
        self.0.write(gc_context).pixels = vec![
            Color(fill_color)
                .to_premultiplied_alpha(self.get_transparency());
            (width * height) as usize
        ]
    }

    fn is_point_in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.get_width() as i32 && y >= 0 && y < self.get_height() as i32
    }

    pub fn get_pixel_raw(&self, x: u32, y: u32) -> Option<Color> {
        if x > self.get_width() || y > self.get_height() {
            return None;
        }

        self.0
            .read()
            .pixels
            .get((x + y * self.get_width()) as usize)
            .copied()
    }

    pub fn get_pixel32(&self, x: i32, y: i32) -> Color {
        self.get_pixel_raw(x as u32, y as u32)
            .map(|f| f.to_un_multiplied_alpha())
            .unwrap_or(0.into())
    }

    pub fn get_pixel(&self, x: i32, y: i32) -> i32 {
        if !self.is_point_in_bounds(x, y) {
            0
        } else {
            self.get_pixel32(x, y).with_alpha(0x0).into()
        }
    }

    //TODO: private?
    pub fn set_pixel32_raw(
        &self,
        gc_context: MutationContext<'gc, '_>,
        x: u32,
        y: u32,
        color: Color,
    ) {
        let width = self.get_width();
        //TODO: bounds check
        self.0.write(gc_context).pixels[(x + y * width) as usize] = color;
    }

    pub fn set_pixel32(&self, gc_context: MutationContext<'gc, '_>, x: i32, y: i32, color: Color) {
        //TODO: what does flash do on set out of bounds
        if self.is_point_in_bounds(x, y) {
            self.set_pixel32_raw(
                gc_context,
                x as u32,
                y as u32,
                color.to_premultiplied_alpha(self.get_transparency()),
            )
        }
    }

    pub fn set_pixel(&self, gc_context: MutationContext<'gc, '_>, x: u32, y: u32, color: Color) {
        let current_alpha = self.get_pixel_raw(x, y).map(|p| p.get_alpha()).unwrap_or(0);
        self.set_pixel32(
            gc_context,
            x as i32,
            y as i32,
            color.with_alpha(current_alpha),
        )

        // self.set_pixel32_raw(
        //     gc_context,
        //     x,
        //     y,
        //     color
        //         .with_alpha(current_alpha)
        //         .to_premultiplied_alpha(self.get_transparency()),
        // )
    }

    pub fn dispose(&self, gc_context: MutationContext<'gc, '_>) {
        self.0.write(gc_context).pixels.clear();
        self.0.write(gc_context).width = 0;
        self.0.write(gc_context).height = 0;
        self.0.write(gc_context).disposed = true;
    }

    pub fn copy_channel(
        &self,
        gc_context: MutationContext<'gc, '_>,
        source: BitmapDataObject,
        /*rect: Rectangle, pnt: Point,*/ source_channel: u8,
        dest_channel: u8,
    ) {
        let mut new_self_pixels = Vec::new(); //(self.0.read().pixels.len());
        let width = self.get_width();
        let height = self.get_height();

        for x in 0..width {
            for y in 0..height {
                // TODO: if rect.contains((x, y)) and offset by pnt

                //TODO: how does this handle out of bounds
                let original_color =
                    self.get_pixel_raw(x as u32, y as u32).unwrap_or(0.into()).0 as u32;

                //TODO: does this calculation work if they are different sizes (might be fixed now)
                let source_color = source.get_pixel_raw(x, y).unwrap_or(0.into()).0 as u32;

                //TODO: should this channel be an enum?
                //TODO: need to support multiple (how does this work if you copy red -> blue and green or any other multi copy)

                //ARGB
                //0xFF

                let channel_shift: u32 = match source_channel {
                    // Alpha
                    8 => 24,
                    // red
                    1 => 16,
                    // green
                    2 => 8,
                    // blue
                    4 => 0,
                    //TODO:
                    _ => panic!(),
                };

                let source_part = (source_color >> channel_shift) & 0xFF;

                let dest_channel_shift: u32 = match dest_channel {
                    // Alpha
                    8 => 24,
                    // red
                    1 => 16,
                    // green
                    2 => 8,
                    // blue
                    4 => 0,
                    //TODO:
                    _ => panic!(),
                };
                let original_color = if dest_channel_shift == 16 {
                    original_color & (4278255615_u32 as u32) //& 0xFF00FFFF
                } else {
                    original_color
                };

                let new_dest_color = original_color | (source_part << dest_channel_shift);

                // println!("x: {}, y: {}, source_color: {}, source_part: {}, original_color: {}, new_color: {}", x, y, Color(source_color as i32), source_part, Color(original_color as i32), Color(new_dest_color as i32));

                //new_self_pixels.insert((y * width + x) as usize, new_dest_color);
                new_self_pixels.push(Color(new_dest_color as i32));
            }
        }

        self.0.write(gc_context).pixels = new_self_pixels;
    }

    //TODO: probably wont handle the edge cases correctly, also may have differences if we dont use premultiplied alpha in our impl (wonder if premultipliing only for functions that need it would be benificial in any way)
    // pub fn threshold(&self, mask: i32, threshold: i32, new_color: i32, copy_source: bool) -> Vec<i32> {
    //     self.0.read().pixels.iter().cloned().map(|p| {
    //         //TODO: support other operations
    //         if (p & mask) == (threshold & mask) {
    //             new_color
    //         } else {
    //             if copy_source {
    //                 p
    //             } else {
    //                 0 //TODO: is this correct
    //             }
    //         }
    //     })
    //         .collect()
    // }

    pub fn fill_rect(
        &self,
        gc_context: MutationContext<'gc, '_>,
        min_x: u32,
        min_y: u32,
        max_x: u32,
        max_y: u32,
        color: Color,
    ) {
        let color = color.to_premultiplied_alpha(self.get_transparency());
        for x in min_x..max_x {
            for y in min_y..max_y {
                self.set_pixel32_raw(gc_context, x, y, color)
            }
        }
    }

    pub fn color_transform(
        &self,
        gc_context: MutationContext<'gc, '_>,
        min_x: u32,
        min_y: u32,
        max_x: u32,
        max_y: u32,
        a_mult: f32,
        a_add: f32,
        r_mult: f32,
        r_add: f32,
        g_mult: f32,
        g_add: f32,
        b_mult: f32,
        b_add: f32,
    ) {
        for x in min_x.max(0)..max_x.min(self.get_width()) {
            for y in min_y.max(0)..max_y.min(self.get_height()) {
                let color = self
                    .get_pixel_raw(x, y)
                    .unwrap_or(0.into())
                    .to_un_multiplied_alpha();
                let a = ((color.get_alpha() as f32 * a_mult) + a_add) as u8;
                let r = ((color.get_red() as f32 * r_mult) + r_add) as u8;
                let g = ((color.get_green() as f32 * g_mult) + g_add) as u8;
                let b = ((color.get_blue() as f32 * b_mult) + b_add) as u8;

                self.set_pixel32_raw(
                    gc_context,
                    x,
                    y,
                    Color::argb(a, r, g, b).to_premultiplied_alpha(self.get_transparency()),
                )
            }
        }
    }

    pub fn get_color_bounds_rect(
        &self,
        mask: i32,
        color: i32,
        find_color: bool,
    ) -> (u32, u32, u32, u32) {
        let mut min_x = Option::<i32>::None;
        let mut max_x = Option::<i32>::None;
        let mut min_y = Option::<i32>::None;
        let mut max_y = Option::<i32>::None;

        for x in 0..self.get_width() {
            for y in 0..self.get_height() {
                let pixel_raw = self.get_pixel_raw(x, y).unwrap_or(0.into()).0;
                let color_matches = if find_color {
                    (pixel_raw & mask) == color
                } else {
                    (pixel_raw & mask) != color
                };

                if color_matches {
                    if (x as i32) < min_x.unwrap_or(self.get_width() as i32) {
                        min_x = Some(x as i32)
                    }
                    if (x as i32) > max_x.unwrap_or(-1) {
                        max_x = Some(x as i32 + 1)
                    }

                    if (y as i32) < min_y.unwrap_or(self.get_height() as i32) {
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

        (
            min_x as u32,
            min_y as u32,
            (max_x - min_x) as u32,
            (max_y - min_y) as u32,
        )
    }

    pub fn get_width(&self) -> u32 {
        self.0.read().width
    }
    pub fn get_height(&self) -> u32 {
        self.0.read().height
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
