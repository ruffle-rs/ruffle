//! `flash.display.Bitmap` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::globals::flash::display::bitmap_data::fill_bitmap_data_from_symbol;
use crate::avm2::globals::flash::display::display_object::initialize_for_allocator;
use crate::avm2::object::{BitmapDataObject, ClassObject, Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use ruffle_macros::istr;
use ruffle_render::bitmap::PixelSnapping;

use crate::avm2::error::make_error_2008;
use crate::avm2::parameters::ParametersExt;
use crate::bitmap::bitmap_data::BitmapDataWrapper;
use crate::character::Character;
use crate::display_object::{Bitmap, TDisplayObject};

pub fn bitmap_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let bitmap_cls = activation.avm2().class_defs().bitmap;
    let bitmapdata_cls = activation.context.avm2.classes().bitmapdata;

    let mut class_def = Some(class.inner_class_definition());
    let orig_class = class;
    while let Some(class) = class_def {
        if class == bitmap_cls {
            let bitmap_data = BitmapDataWrapper::dummy(activation.gc());
            let display_object = Bitmap::new_with_bitmap_data(
                activation.gc(),
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
                    BitmapDataWrapper::dummy(activation.gc()),
                    bitmapdata_cls,
                )?;
                bitmap_data_obj.init_bitmap_data(activation.gc(), new_bitmap_data);
                new_bitmap_data.init_object2(activation.gc(), bitmap_data_obj);

                let child = Bitmap::new_with_bitmap_data(
                    activation.gc(),
                    0,
                    new_bitmap_data,
                    false,
                    &activation.caller_movie_or_root(),
                );

                return initialize_for_allocator(activation, child.into(), orig_class);
            }
        }
        class_def = class.super_class();
    }
    unreachable!("A Bitmap subclass should have Bitmap in superclass chain");
}

/// Implements `flash.display.Bitmap`'s `init` method, which is called from the constructor
pub fn init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let bitmap_data = args
        .try_get_object(activation, 0)
        .and_then(|o| o.as_bitmap_data());

    let pixel_snapping = args.get_string(activation, 1)?;

    let pixel_snapping = if &pixel_snapping == b"always" {
        PixelSnapping::Always
    } else if &pixel_snapping == b"auto" {
        PixelSnapping::Auto
    } else if &pixel_snapping == b"never" {
        PixelSnapping::Never
    } else {
        return Err(make_error_2008(activation, "pixelSnapping"));
    };

    let smoothing = args.get_bool(2);

    if let Some(bitmap) = this.as_display_object().and_then(|dobj| dobj.as_bitmap()) {
        if let Some(bitmap_data) = bitmap_data {
            bitmap.set_bitmap_data(activation.context, bitmap_data);
        }
        bitmap.set_smoothing(smoothing);
        bitmap.set_pixel_snapping(pixel_snapping);
    } else {
        unreachable!();
    }

    Ok(Value::Undefined)
}

/// Implements `Bitmap.bitmapData`'s getter.
pub fn get_bitmap_data<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

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
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(bitmap) = this.as_display_object().and_then(|dobj| dobj.as_bitmap()) {
        let bitmap_data = args.try_get_object(activation, 0);

        let bitmap_data = if let Some(bitmap_data) = bitmap_data {
            bitmap_data.as_bitmap_data().expect("Must be a BitmapData")
        } else {
            // Passing null results in a dummy BitmapData being set.
            BitmapDataWrapper::dummy(activation.gc())
        };

        bitmap.set_bitmap_data(activation.context, bitmap_data);
    }

    Ok(Value::Undefined)
}

/// Stub `Bitmap.pixelSnapping`'s getter
pub fn get_pixel_snapping<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(bitmap) = this.as_display_object().and_then(|dobj| dobj.as_bitmap()) {
        let pixel_snapping = match bitmap.pixel_snapping() {
            PixelSnapping::Always => istr!("always"),
            PixelSnapping::Auto => istr!("auto"),
            PixelSnapping::Never => istr!("never"),
        };

        return Ok(pixel_snapping.into());
    }
    Ok(Value::Undefined)
}

/// Stub `Bitmap.pixelSnapping`'s setter
pub fn set_pixel_snapping<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(bitmap) = this.as_display_object().and_then(|dobj| dobj.as_bitmap()) {
        let value = args.get_string(activation, 0)?;

        let pixel_snapping = if &value == b"always" {
            PixelSnapping::Always
        } else if &value == b"auto" {
            PixelSnapping::Auto
        } else if &value == b"never" {
            PixelSnapping::Never
        } else {
            return Err(make_error_2008(activation, "pixelSnapping"));
        };

        bitmap.set_pixel_snapping(pixel_snapping);
    }
    Ok(Value::Undefined)
}

/// Implement `Bitmap.smoothing`'s getter
pub fn get_smoothing<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(bitmap) = this.as_display_object().and_then(|dobj| dobj.as_bitmap()) {
        return Ok(bitmap.smoothing().into());
    }

    Ok(Value::Undefined)
}

/// Implement `Bitmap.smoothing`'s setter
pub fn set_smoothing<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(bitmap) = this.as_display_object().and_then(|dobj| dobj.as_bitmap()) {
        let smoothing = args.get_bool(0);
        bitmap.set_smoothing(smoothing);
    }

    Ok(Value::Undefined)
}
