//! `flash.display.Bitmap` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::globals::flash::display::bitmap_data::fill_bitmap_data_from_symbol;
use crate::avm2::object::{BitmapDataObject, Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;

use crate::avm2::parameters::ParametersExt;
use crate::bitmap::bitmap_data::{BitmapData, BitmapDataWrapper};
use crate::character::Character;
use crate::display_object::{Bitmap, TDisplayObject};
use crate::{avm2_stub_getter, avm2_stub_setter};
use gc_arena::GcCell;

/// Implements `flash.display.Bitmap`'s `init` method, which is called from the constructor
pub fn init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut this) = this {
        activation.super_init(this, &[])?;

        let bitmap_data = args
            .try_get_object(activation, 0)
            .and_then(|o| o.as_bitmap_data_wrapper());
        //TODO: Pixel snapping is not supported
        let _pixel_snapping = args.get_string(activation, 1);
        let smoothing = args.get_bool(2);

        if let Some(bitmap) = this.as_display_object().and_then(|dobj| dobj.as_bitmap()) {
            //We are being initialized by the movie. This means that we
            //need to create bitmap data right away, since all AVM2 bitmaps
            //hold bitmap data.

            let bd_object = if let Some(bd_class) = bitmap.avm2_bitmapdata_class() {
                // We call the custom BitmapData class with width and height...
                // but, it always seems to be 1 in Flash Player when constructed from timeline?
                bd_class.construct(activation, &[1.into(), 1.into()])?
            } else if let Some(b_class) = bitmap.avm2_bitmap_class() {
                // Instantiating Bitmap from a Flex-style bitmap asset.
                // Contrary to the above comment, this code path DOES
                // trigger from AVM2, since the DisplayObject instantiation
                // logic does its job in this case.
                if let Some((movie, symbol_id)) = activation
                    .context
                    .library
                    .avm2_class_registry()
                    .class_symbol(b_class)
                {
                    if let Some(Character::Bitmap(bitmap)) = activation
                        .context
                        .library
                        .library_for_movie_mut(movie)
                        .character_by_id(symbol_id)
                        .cloned()
                    {
                        let new_bitmap_data =
                            GcCell::allocate(activation.context.gc_context, BitmapData::default());

                        fill_bitmap_data_from_symbol(activation, bitmap, new_bitmap_data);
                        BitmapDataObject::from_bitmap_data(
                            activation,
                            new_bitmap_data,
                            activation.context.avm2.classes().bitmapdata,
                        )?
                    } else {
                        //Class association not to a Bitmap
                        return Err("Attempted to instantiate Bitmap from timeline with symbol class associated to non-Bitmap!".into());
                    }
                } else {
                    //Class association not bidirectional
                    return Err("Cannot instantiate Bitmap from timeline without bidirectional symbol class association".into());
                }
            } else {
                // No class association
                return Err(
                    "Cannot instantiate Bitmap from timeline without associated symbol class"
                        .into(),
                );
            };

            this.set_public_property("bitmapData", bd_object.into(), activation)?;

            bitmap.set_smoothing(activation.context.gc_context, smoothing);
        } else {
            //We are being initialized by AVM2 (and aren't associated with a
            //Bitmap subclass).

            let bitmap_data = bitmap_data.unwrap_or_else(|| {
                BitmapDataWrapper::new(GcCell::allocate(
                    activation.context.gc_context,
                    BitmapData::dummy(),
                ))
            });

            let bitmap =
                Bitmap::new_with_bitmap_data(&mut activation.context, 0, bitmap_data, smoothing);

            this.init_display_object(&mut activation.context, bitmap.into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Bitmap.bitmapData`'s getter.
pub fn get_bitmap_data<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap) = this
        .and_then(|this| this.as_display_object())
        .and_then(|dobj| dobj.as_bitmap())
    {
        let mut value = bitmap.bitmap_data_wrapper().object2();

        // AS3 expects an unset BitmapData to be null, not 'undefined'
        if matches!(value, Value::Undefined) {
            value = Value::Null;
        }
        return Ok(value);
    }

    Ok(Value::Undefined)
}

/// Implements `Bitmap.bitmapData`'s setter.
pub fn set_bitmap_data<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap) = this
        .and_then(|this| this.as_display_object())
        .and_then(|dobj| dobj.as_bitmap())
    {
        let bitmap_data = args.get(0).unwrap_or(&Value::Null);
        let bitmap_data = if matches!(bitmap_data, Value::Null) {
            GcCell::allocate(activation.context.gc_context, BitmapData::dummy())
        } else {
            bitmap_data
                .coerce_to_object(activation)?
                .as_bitmap_data()
                .ok_or_else(|| Error::RustError("Argument was not a BitmapData".into()))?
        };
        bitmap.set_bitmap_data(&mut activation.context, bitmap_data);
    }

    Ok(Value::Undefined)
}

/// Stub `Bitmap.pixelSnapping`'s getter
pub fn get_pixel_snapping<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(activation, "flash.display.Bitmap", "pixelSnapping");
    Ok("auto".into())
}

/// Stub `Bitmap.pixelSnapping`'s setter
pub fn set_pixel_snapping<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_setter!(activation, "flash.display.Bitmap", "pixelSnapping");
    Ok(Value::Undefined)
}

/// Implement `Bitmap.smoothing`'s getter
pub fn get_smoothing<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap) = this
        .and_then(|this| this.as_display_object())
        .and_then(|dobj| dobj.as_bitmap())
    {
        return Ok(bitmap.smoothing().into());
    }

    Ok(Value::Undefined)
}

/// Implement `Bitmap.smoothing`'s setter
pub fn set_smoothing<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap) = this
        .and_then(|this| this.as_display_object())
        .and_then(|dobj| dobj.as_bitmap())
    {
        let smoothing = args.get_bool(0);
        bitmap.set_smoothing(activation.context.gc_context, smoothing);
    }

    Ok(Value::Undefined)
}
