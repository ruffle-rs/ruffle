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
    add_field_accessors!(
        [transparency, bool, set => set_transparency, get => get_transparency],
        [disposed, bool, get => get_disposed],
        [pixels, Vec<Color>, set => set_pixels],
        [width, u32, get => get_width],
        [height, u32, get => get_height],
    );

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

    pub fn get_pixels(&self) -> Vec<Color> {
        self.0.read().pixels.clone()
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

    pub fn is_point_in_bounds(&self, x: i32, y: i32) -> bool {
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
            .unwrap_or_else(|| 0.into())
    }

    pub fn get_pixel(&self, x: i32, y: i32) -> i32 {
        if !self.is_point_in_bounds(x, y) {
            0
        } else {
            self.get_pixel32(x, y).with_alpha(0x0).into()
        }
    }

    pub fn set_pixel32_raw(
        &self,
        gc_context: MutationContext<'gc, '_>,
        x: u32,
        y: u32,
        color: Color,
    ) {
        let width = self.get_width();
        self.0.write(gc_context).pixels[(x + y * width) as usize] = color;
    }

    pub fn set_pixel32(&self, gc_context: MutationContext<'gc, '_>, x: i32, y: i32, color: Color) {
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
    }

    pub fn dispose(&self, gc_context: MutationContext<'gc, '_>) {
        self.0.write(gc_context).pixels.clear();
        self.0.write(gc_context).width = 0;
        self.0.write(gc_context).height = 0;
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
