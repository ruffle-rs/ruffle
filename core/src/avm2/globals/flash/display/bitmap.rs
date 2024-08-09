//! `flash.display.Bitmap` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::globals::flash::display::bitmap_data::fill_bitmap_data_from_symbol;
use crate::avm2::globals::flash::display::display_object::initialize_for_allocator;
use crate::avm2::object::{BitmapDataObject, ClassObject, Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use ruffle_render::bitmap::PixelSnapping;
use ruffle_wstr::WStr;

use crate::avm2::error::make_error_2008;
use crate::avm2::parameters::ParametersExt;
use crate::bitmap::bitmap_data::BitmapDataWrapper;
use crate::character::Character;
use crate::display_object::{Bitmap, TDisplayObject};

pub fn bitmap_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let bitmap_cls = activation.avm2().classes().bitmap.inner_class_definition();
    let bitmapdata_cls = activation.context.avm2.classes().bitmapdata;

    let mut class_def = Some(class.inner_class_definition());
    let orig_class = class;
    while let Some(class) = class_def {
        if class == bitmap_cls {
            let bitmap_data = BitmapDataWrapper::dummy(activation.context.gc_context);
            let display_object = Bitmap::new_with_bitmap_data(
                activation.context.gc_context,
                0,
                bitmap_data,
                false,
                &activation.caller_movie_or_root(),
            )
            .into();
            return initialize_for_allocator(activation, display_object, orig_class);
        }

        if let Some((movie, symbol)) = activation
            .context
            .library
            .avm2_class_registry()
            .class_symbol(class)
        {
            if let Some(Character::Bitmap {
                compressed,
                avm2_bitmapdata_class: _,
                handle: _,
            }) = activation
                .context
                .library
                .library_for_movie_mut(movie)
                .character_by_id(symbol)
                .cloned()
            {
                let new_bitmap_data = fill_bitmap_data_from_symbol(activation, &compressed);
                let bitmap_data_obj = BitmapDataObject::from_bitmap_data_internal(
                    activation,
                    BitmapDataWrapper::dummy(activation.context.gc_context),
                    bitmapdata_cls,
                )?;
                bitmap_data_obj.init_bitmap_data(activation.context.gc_context, new_bitmap_data);
                new_bitmap_data.init_object2(activation.context.gc_context, bitmap_data_obj);

                let child = Bitmap::new_with_bitmap_data(
                    activation.context.gc_context,
                    0,
                    new_bitmap_data,
                    false,
                    &activation.caller_movie_or_root(),
                );

                let obj = initialize_for_allocator(activation, child.into(), orig_class)?;
                obj.set_public_property("bitmapData", bitmap_data_obj.into(), activation)?;
                return Ok(obj);
            }
        }
        class_def = class.super_class();
    }
    unreachable!("A Bitmap subclass should have Bitmap in superclass chain");
}

/// Implements `flash.display.Bitmap`'s `init` method, which is called from the constructor
pub fn init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let bitmap_data = args
        .try_get_object(activation, 0)
        .and_then(|o| o.as_bitmap_data());
    let Some(pixel_snapping) = PixelSnapping::from_wstr(&args.get_string(activation, 1)?) else {
        return Err(make_error_2008(activation, "pixelSnapping"));
    };
    let smoothing = args.get_bool(2);

    if let Some(bitmap) = this.as_display_object().and_then(|dobj| dobj.as_bitmap()) {
        if let Some(bitmap_data) = bitmap_data {
            bitmap.set_bitmap_data(activation.context, bitmap_data);
        }
        bitmap.set_smoothing(activation.context.gc_context, smoothing);
        bitmap.set_pixel_snapping(activation.context.gc_context, pixel_snapping);
    } else {
        unreachable!();
    }

    Ok(Value::Undefined)
}

/// Implements `Bitmap.bitmapData`'s getter.
pub fn get_bitmap_data<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap) = this.as_display_object().and_then(|dobj| dobj.as_bitmap()) {
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
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap) = this.as_display_object().and_then(|dobj| dobj.as_bitmap()) {
        let bitmap_data = args.get(0).unwrap_or(&Value::Null);
        let bitmap_data = if matches!(bitmap_data, Value::Null) {
            BitmapDataWrapper::dummy(activation.context.gc_context)
        } else {
            bitmap_data
                .coerce_to_object(activation)?
                .as_bitmap_data()
                .ok_or_else(|| Error::RustError("Argument was not a BitmapData".into()))?
        };
        bitmap.set_bitmap_data(activation.context, bitmap_data);
    }

    Ok(Value::Undefined)
}

/// Stub `Bitmap.pixelSnapping`'s getter
pub fn get_pixel_snapping<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap) = this.as_display_object().and_then(|dobj| dobj.as_bitmap()) {
        let value: &WStr = bitmap.pixel_snapping().into();
        return Ok(Value::String(value.into()));
    }
    Ok(Value::Undefined)
}

/// Stub `Bitmap.pixelSnapping`'s setter
pub fn set_pixel_snapping<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap) = this.as_display_object().and_then(|dobj| dobj.as_bitmap()) {
        let Some(value) = PixelSnapping::from_wstr(&args.get_string(activation, 0)?) else {
            return Err(make_error_2008(activation, "pixelSnapping"));
        };
        bitmap.set_pixel_snapping(activation.context.gc_context, value);
    }
    Ok(Value::Undefined)
}

/// Implement `Bitmap.smoothing`'s getter
pub fn get_smoothing<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap) = this.as_display_object().and_then(|dobj| dobj.as_bitmap()) {
        return Ok(bitmap.smoothing().into());
    }

    Ok(Value::Undefined)
}

/// Implement `Bitmap.smoothing`'s setter
pub fn set_smoothing<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap) = this.as_display_object().and_then(|dobj| dobj.as_bitmap()) {
        let smoothing = args.get_bool(0);
        bitmap.set_smoothing(activation.context.gc_context, smoothing);
    }

    Ok(Value::Undefined)
}
