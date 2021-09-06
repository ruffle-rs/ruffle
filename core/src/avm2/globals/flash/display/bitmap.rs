//! `flash.display.Bitmap` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
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

        if let Some(bitmap) = this.as_display_object().and_then(|dobj| dobj.as_bitmap()) {
            if bitmap.bitmap_data().is_none() {
                //We are being initialized by the movie. This means that we
                //need to create bitmap data right away, since all AVM2 bitmaps
                //hold bitmap data.

                if let Some(bd_class) = bitmap.avm2_bitmapdata_class() {
                    let bd_object = bd_class.construct(activation, &[])?;

                    this.set_property(
                        this,
                        &QName::new(Namespace::public(), "bitmapData"),
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
        } else {
            //We are being initialized by AVM2.
            let bitmap_data = args
                .get(0)
                .cloned()
                .unwrap_or(Value::Null)
                .coerce_to_object(activation)
                .ok()
                .and_then(|bd| bd.as_bitmap_data());
            //TODO: Pixel snapping is not supported
            let _pixel_snapping = args
                .get(0)
                .cloned()
                .unwrap_or_else(|| "auto".into())
                .coerce_to_string(activation)?;
            let smoothing = args
                .get(0)
                .cloned()
                .unwrap_or_else(|| false.into())
                .coerce_to_boolean();

            let bitmap_handle = if let Some(bd) = bitmap_data {
                bd.write(activation.context.gc_context)
                    .bitmap_handle(activation.context.renderer)
                    .ok_or("Bitmap data missing it's handle!")?
            } else {
                //TODO: Should Bitmap's BitmapHandle be nullable?
                return Err("Null bitmap data not yet implemented".into());
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
            .unwrap_or(Value::Undefined));
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
        let bitmap_data_object = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_object(activation)
            .map_err(|_| "Attempted to set Bitmap.bitmapData with a non-BitmapData object")?;
        if let Some(bitmap_data) = bitmap_data_object.as_bitmap_data() {
            bitmap.set_bitmap_data(&mut activation.context, bitmap_data);
        }
    }

    Ok(Value::Undefined)
}

/// Construct `Bitmap`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.display"), "Bitmap"),
        Some(QName::new(Namespace::package("flash.display"), "DisplayObject").into()),
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
    )] = &[("bitmapData", Some(bitmap_data), Some(set_bitmap_data))];
    write.define_public_builtin_instance_properties(mc, PUBLIC_INSTANCE_PROPERTIES);

    class
}
