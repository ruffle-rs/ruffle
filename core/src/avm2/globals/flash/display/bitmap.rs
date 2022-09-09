//! `flash.display.Bitmap` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::object::{Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::Multiname;
use crate::avm2::Namespace;
use crate::avm2::QName;
use crate::display_object::{Bitmap, TDisplayObject};
use gc_arena::{GcCell, MutationContext};

/// Implements `flash.display.Bitmap`'s instance constructor.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut this) = this {
        activation.super_init(this, &[])?;

        let bitmap_data = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Null)
            .as_object()
            .and_then(|bd| bd.as_bitmap_data());
        //TODO: Pixel snapping is not supported
        let _pixel_snapping = args
            .get(1)
            .cloned()
            .unwrap_or_else(|| "auto".into())
            .coerce_to_string(activation)?;
        let smoothing = args
            .get(2)
            .cloned()
            .unwrap_or_else(|| false.into())
            .coerce_to_boolean();

        if let Some(bitmap) = this.as_display_object().and_then(|dobj| dobj.as_bitmap()) {
            if bitmap.bitmap_data().is_none() {
                //We are being initialized by the movie. This means that we
                //need to create bitmap data right away, since all AVM2 bitmaps
                //hold bitmap data.

                if let Some(bd_class) = bitmap.avm2_bitmapdata_class() {
                    let bd_object = bd_class.construct(activation, &[])?;

                    this.set_property(
                        &Multiname::public("bitmapData"),
                        bd_object.into(),
                        activation,
                    )?;
                } else {
                    return Err(
                        "Cannot instantiate Bitmap from timeline without associated symbol class"
                            .into(),
                    );
                }
            }

            bitmap.set_smoothing(activation.context.gc_context, smoothing);
        } else {
            //We are being initialized by AVM2.
            let bitmap_handle = if let Some(bd) = bitmap_data {
                bd.write(activation.context.gc_context)
                    .bitmap_handle(activation.context.renderer)
            } else {
                None
            };

            let width = bitmap_data.map(|bd| bd.read().width()).unwrap_or(0) as u16;
            let height = bitmap_data.map(|bd| bd.read().height()).unwrap_or(0) as u16;

            let bitmap = Bitmap::new_with_bitmap_data(
                &mut activation.context,
                0,
                bitmap_handle,
                width,
                height,
                bitmap_data,
                smoothing,
            );

            this.init_display_object(activation.context.gc_context, bitmap.into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `flash.display.Bitmap`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements `Bitmap.bitmapData`'s getter.
pub fn bitmap_data<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(bitmap) = this
        .and_then(|this| this.as_display_object())
        .and_then(|dobj| dobj.as_bitmap())
    {
        return Ok(bitmap
            .bitmap_data()
            .map(|bd| bd.read().object2())
            .unwrap_or(Value::Null));
    }

    Ok(Value::Undefined)
}

/// Implements `Bitmap.bitmapData`'s setter.
pub fn set_bitmap_data<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(bitmap) = this
        .and_then(|this| this.as_display_object())
        .and_then(|dobj| dobj.as_bitmap())
    {
        let bitmap_data = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Null)
            .as_object()
            .and_then(|bd| bd.as_bitmap_data());

        bitmap.set_bitmap_data(&mut activation.context, bitmap_data);
    }

    Ok(Value::Undefined)
}

/// Stub `Bitmap.pixelSnapping`'s getter
pub fn pixel_snapping<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok("auto".into())
}

/// Stub `Bitmap.pixelSnapping`'s setter
pub fn set_pixel_snapping<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Bitmap.pixelSnapping is a stub".into())
}

/// Implement `Bitmap.smoothing`'s getter
pub fn smoothing<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
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
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(bitmap) = this
        .and_then(|this| this.as_display_object())
        .and_then(|dobj| dobj.as_bitmap())
    {
        let smoothing = args.get(0).unwrap_or(&Value::Undefined).coerce_to_boolean();
        bitmap.set_smoothing(activation.context.gc_context, smoothing);
    }

    Ok(Value::Undefined)
}

/// Construct `Bitmap`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.display"), "Bitmap"),
        Some(Multiname::new(
            Namespace::package("flash.display"),
            "DisplayObject",
        )),
        Method::from_builtin(instance_init, "<Bitmap instance initializer>", mc),
        Method::from_builtin(class_init, "<Bitmap class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    write.set_attributes(ClassAttributes::SEALED);

    const PUBLIC_INSTANCE_PROPERTIES: &[(
        &str,
        Option<NativeMethodImpl>,
        Option<NativeMethodImpl>,
    )] = &[
        ("bitmapData", Some(bitmap_data), Some(set_bitmap_data)),
        (
            "pixelSnapping",
            Some(pixel_snapping),
            Some(set_pixel_snapping),
        ),
        ("smoothing", Some(smoothing), Some(set_smoothing)),
    ];
    write.define_public_builtin_instance_properties(mc, PUBLIC_INSTANCE_PROPERTIES);

    class
}
