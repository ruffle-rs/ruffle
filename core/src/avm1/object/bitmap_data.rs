use crate::add_field_accessors;
use crate::avm1::error::Error;
use crate::avm1::{Object, ScriptObject, TObject, Value};
use crate::impl_custom_object_without_set;
use gc_arena::{Collect, GcCell, MutationContext};

use crate::avm1::activation::Activation;
use std::fmt;
use swf::Rectangle;

/// A BitmapData
#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct BitmapDataObject<'gc>(GcCell<'gc, BitmapDataData<'gc>>);

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct BitmapDataData<'gc> {
    /// The underlying script object.
    base: ScriptObject<'gc>,

    /// The pixels in the bitmap, stored as a array of ARGB colour values
    pixels: Vec<i32>,

    width: u32,
    height: u32,
    transparency: bool,
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
        [set_transparency, get_transparency, transparency, bool],
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
            },
        ))
    }

    pub fn get_pixels(&self) -> Vec<i32> {
        self.0.read().pixels.clone()
    }

    pub fn init_pixels(&self, gc_context: MutationContext<'gc, '_>, width: u32, height: u32, fill_color: i32) {
        self.0.write(gc_context).width = width;
        self.0.write(gc_context).height = height;
        self.0.write(gc_context).pixels = vec![fill_color; (width * height) as usize]
    }

    pub fn get_pixel32(&self, x: u32, y: u32) -> Option<i32> {
        //TODO: look into the effects of pre-multiplication
        self.0.read().pixels.get((x * y) as usize).cloned()
    }
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<i32> {
        self.get_pixel32(x, y).map(|p| p & 0xFFFFFF)
    }

    pub fn set_pixel32(&self, gc_context: MutationContext<'gc, '_>, x: u32, y: u32, color: i32) {
        let alpha = (color >> 24) & 0xFF;

        // Internally flash uses pre-multiplied values however that will cause issues with accuracy (assuming they are using fixed point like with other color types)
        // so we will just fake it for now, if the alpha is 0 then it will clear out the colors
        //tODO: how well does this handle less edge case values eg 0x12345678?
        let adjusted_color = if alpha == 0 && self.0.read().transparency {
            // Alpha = 0 and transparency is on so clear out rest of color
            0
        } else {
            // If we don't have transparency then force the alpha to 0xFF
            //TODO: could we do that earlier to make this cleaner
            if self.0.read().transparency {
                color
            } else {
                (color & 0xFFFFFF) | (0xFF << 24)
            }
        };

        //TODO: bounds check
        self.0.write(gc_context).pixels[(x * y) as usize] = adjusted_color;
    }

    pub fn set_pixel(&self, gc_context: MutationContext<'gc, '_>, x: u32, y: u32, color: i32) {
        let current_alpha = (self.get_pixel32(x, y).unwrap_or(0) >> 24) & 0xFF;
        //todo: check this shift
        self.set_pixel32(gc_context, x, y, color | (current_alpha << 24))
    }

    pub fn dispose(&self, gc_context: MutationContext<'gc, '_>) {
        self.0.write(gc_context).pixels.clear();
        self.0.write(gc_context).width = 0;
        self.0.write(gc_context).height = 0;
    }

    pub fn copy_channel(&self, gc_context: MutationContext<'gc, '_>, source: BitmapDataObject, /*rect: Rectangle, pnt: Point,*/ source_channel: u8, dest_channel: u8) {
        let other_pixels = source.get_pixels();

        let mut new_self_pixels = Vec::new();//(self.0.read().pixels.len());
        let width = self.get_width();
        let height = self.get_height();

        for x in 0..width {
            for y in 0..height {
                // TODO: if rect.contains((x, y)) and offset by pnt

                //TODO: how does this handle out of bounds
                let original_color = self.0.read().pixels[(y * width + x) as usize];

                //TODO: does this calculation work if they are different sizes
                let source_color = other_pixels[(y * width + x) as usize];

                //TODO: should this channel be an enum?
                //TODO: need to support multiple (how does this work if you copy red -> blue and green or any other multi copy)
                let channel_shift = match source_channel {
                    8 => 24,
                    4 => 16,
                    2 => 8,
                    1 => 0,
                    //TODO:
                    _ => {panic!()}
                };

                let source_part = (source_color >> channel_shift) & 0xFF;

                let dest_channel_shift = match dest_channel {
                    8 => 24,
                    4 => 16,
                    2 => 8,
                    1 => 0,
                    //TODO:
                    _ => {panic!()}
                };
                let original_color = if dest_channel_shift == 0 {
                    original_color & (4278255615_u32 as i32)//& 0xFF00FFFF
                } else {
                    original_color
                };

                let new_dest_color = original_color | ((source_part << dest_channel_shift) & 0xFF);

                //new_self_pixels.insert((y * width + x) as usize, new_dest_color);
                new_self_pixels.push(new_dest_color);
            }
        }

        self.0.write(gc_context).pixels = new_self_pixels;
    }

    //TODO: probably wont handle the edge cases correctly, also may have differences if we dont use premultiplied alpha in our impl (wonder if premultipliing only for functions that need it would be benificial in any way)
    pub fn threshold(&self, mask: i32, threshold: i32, new_color: i32, copy_source: bool) -> Vec<i32> {
        self.0.read().pixels.iter().cloned().map(|p| {
            //TODO: support other operations
            if (p & mask) == (threshold & mask) {
                new_color
            } else {
                if copy_source {
                    p
                } else {
                    0 //TODO: is this correct
                }
            }
        })
            .collect()
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
