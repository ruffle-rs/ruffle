use crate::add_field_accessors;
use crate::avm1::error::Error;
use crate::avm1::{Object, ScriptObject, TObject, Value};
use crate::impl_custom_object_without_set;
use gc_arena::{Collect, GcCell, MutationContext};

use crate::avm1::activation::Activation;
use std::fmt;

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
    pixels: Vec<u32>,

    //todO: track width and height
}

impl fmt::Debug for BitmapDataObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let this = self.0.read();
        f.debug_struct("BitmapData")
            .field("pixels", &this.pixels)
            .finish()
    }
}

impl<'gc> BitmapDataObject<'gc> {
    pub fn empty_object(gc_context: MutationContext<'gc, '_>, proto: Option<Object<'gc>>) -> Self {
        BitmapDataObject(GcCell::allocate(
            gc_context,
            BitmapDataData {
                base: ScriptObject::object(gc_context, proto),
                pixels: Vec::new(),
            },
        ))
    }

    pub fn init_pixels(&self, gc_context: MutationContext<'gc, '_>, width: u32, height: u32, fill_color: u32) {
        self.0.write(gc_context).pixels = vec![fill_color; (width * height) as usize]
    }

    pub fn get_pixel32(&self, x: u32, y: u32) -> Option<u32> {
        //TODO: look into the effects of pre-multiplication
        self.0.read().pixels.get((x * y) as usize).cloned()
    }
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<u32> {
        self.get_pixel32(x, y).map(|p| p & 0xFFFFFF)
    }

    pub fn set_pixel32(&self, gc_context: MutationContext<'gc, '_>, x: u32, y: u32, color: u32) {
        //TODO: bounds check
        self.0.write(gc_context).pixels[(x * y) as usize] = color
    }

    pub fn set_pixel(&self, gc_context: MutationContext<'gc, '_>, x: u32, y: u32, color: u32) {
        let current_alpha = self.get_pixel32(x, y).unwrap_or(0);
        //todo: check this shift
        self.set_pixel32(gc_context, x, y, color | (current_alpha << 24))
    }

    pub fn dispose(&self, gc_context: MutationContext<'gc, '_>) {
        self.0.write(gc_context).pixels.clear()
    }

    //TODO: probably wont handle the edge cases correctly, also may have differences if we dont use premultiplied alpha in our impl (wonder if premultipliing only for functions that need it would be benificial in any way)
    pub fn threshold(&self, mask: u32, threshold: u32, new_color: u32, copy_source: bool) -> Vec<u32> {
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
