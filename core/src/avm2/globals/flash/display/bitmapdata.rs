//! `flash.display.BitmapData` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{bitmapdata_allocator, Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::bitmap::bitmap_data::BitmapData;
use crate::bitmap::is_size_valid;
use crate::character::Character;
use gc_arena::{GcCell, MutationContext};

/// Implements `flash.display.BitmapData`'s instance constructor.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        activation.super_init(this, &[])?;

        let name = this
            .as_class_object()
            .and_then(|t| t.as_class())
            .map(|c| c.read().name().clone());
        let character = this
            .as_class_object()
            .and_then(|t| {
                activation
                    .context
                    .library
                    .avm2_class_registry()
                    .class_symbol(t)
            })
            .and_then(|(movie, chara_id)| {
                activation
                    .context
                    .library
                    .library_for_movie_mut(movie)
                    .character_by_id(chara_id)
                    .cloned()
            });

        let new_bitmap_data =
            GcCell::allocate(activation.context.gc_context, BitmapData::default());

        if let Some(Character::Bitmap(bd)) = character {
            let bitmap_handle = bd.bitmap_handle();

            if let Some(bitmap_pixels) =
                activation.context.renderer.get_bitmap_pixels(bitmap_handle)
            {
                let bitmap_pixels: Vec<i32> = bitmap_pixels.data.into();
                new_bitmap_data
                    .write(activation.context.gc_context)
                    .set_pixels(
                        bd.width().into(),
                        bd.height().into(),
                        true,
                        bitmap_pixels.into_iter().map(|p| p.into()).collect(),
                    );
            } else {
                log::warn!(
                    "Could not read bitmap data associated with class {:?}",
                    name
                );
            }
        } else {
            if character.is_some() {
                //TODO: Determine if mismatched symbols will still work as a
                //regular BitmapData subclass, or if this should throw
                log::warn!(
                    "BitmapData subclass {:?} is associated with a non-bitmap symbol",
                    name
                );
            }

            let width = args
                .get(0)
                .unwrap_or(&Value::Undefined)
                .coerce_to_i32(activation)? as u32;
            let height = args
                .get(1)
                .unwrap_or(&Value::Undefined)
                .coerce_to_i32(activation)? as u32;
            let transparency = args
                .get(2)
                .unwrap_or(&Value::Bool(true))
                .coerce_to_boolean();
            let fill_color = args
                .get(3)
                .unwrap_or(&Value::Unsigned(0xFFFFFFFF))
                .coerce_to_u32(activation)?;

            if !is_size_valid(activation.context.swf.version(), width, height) {
                return Err("Bitmap size is not valid".into());
            }

            new_bitmap_data
                .write(activation.context.gc_context)
                .init_pixels(width, height, transparency, fill_color as i32);
        }

        new_bitmap_data
            .write(activation.context.gc_context)
            .init_object2(this);
        this.init_bitmap_data(activation.context.gc_context, new_bitmap_data);
    }

    Ok(Value::Undefined)
}

/// Implements `flash.display.BitmapData`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements BitmapData.width`'s getter.
pub fn width<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(bitmap_data) = this.and_then(|t| t.as_bitmap_data()) {
        return Ok((bitmap_data.read().width() as i32).into());
    }

    Ok(Value::Undefined)
}

/// Implements BitmapData.height`'s getter.
pub fn height<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(bitmap_data) = this.and_then(|t| t.as_bitmap_data()) {
        return Ok((bitmap_data.read().height() as i32).into());
    }

    Ok(Value::Undefined)
}

/// Implements BitmapData.transparent`'s getter.
pub fn transparent<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(bitmap_data) = this.and_then(|t| t.as_bitmap_data()) {
        return Ok(bitmap_data.read().transparency().into());
    }

    Ok(Value::Undefined)
}

/// Construct `BitmapData`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.display"), "BitmapData"),
        Some(QName::new(Namespace::package(""), "Object").into()),
        Method::from_builtin(instance_init, "<BitmapData instance initializer>", mc),
        Method::from_builtin(class_init, "<BitmapData class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    write.set_attributes(ClassAttributes::SEALED);
    write.set_instance_allocator(bitmapdata_allocator);

    const PUBLIC_INSTANCE_PROPERTIES: &[(
        &str,
        Option<NativeMethodImpl>,
        Option<NativeMethodImpl>,
    )] = &[
        ("width", Some(width), None),
        ("height", Some(height), None),
        ("transparent", Some(transparent), None),
    ];
    write.define_public_builtin_instance_properties(mc, PUBLIC_INSTANCE_PROPERTIES);

    class
}
