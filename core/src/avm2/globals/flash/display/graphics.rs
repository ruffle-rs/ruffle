//! `flash.display.Graphics` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::method::Method;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{Object, TObject};
use crate::avm2::traits::Trait;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::display_object::TDisplayObject;
use gc_arena::{GcCell, MutationContext};
use swf::{Color, FillStyle};

/// Implements `flash.display.Graphics`'s instance constructor.
pub fn instance_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Graphics cannot be constructed directly.".into())
}

/// Implements `flash.display.Graphics`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements `Graphics.beginFill`.
pub fn begin_fill<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(dobj) = this.as_display_object() {
            if let Some(mc) = dobj.as_movie_clip() {
                let color = args
                    .get(0)
                    .cloned()
                    .unwrap_or(Value::Undefined)
                    .coerce_to_u32(activation)?;
                let r = (color & 0xFF0000 >> 16) as u8;
                let g = (color & 0x00FF00 >> 8) as u8;
                let b = (color & 0x0000FF) as u8;
                let a = (args
                    .get(1)
                    .cloned()
                    .unwrap_or_else(|| 1.0.into())
                    .coerce_to_number(activation)?
                    * 255.0) as u8;

                let color = Color { r, g, b, a };

                mc.set_fill_style(&mut activation.context, Some(FillStyle::Color(color)));
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Graphics.clear`
pub fn clear<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(dobj) = this.as_display_object() {
            if let Some(mc) = dobj.as_movie_clip() {
                mc.clear(&mut activation.context)
            }
        }
    }

    Ok(Value::Undefined)
}

/// Construct `Graphics`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.display"), "Graphics"),
        Some(QName::new(Namespace::public(), "Object").into()),
        Method::from_builtin(instance_init),
        Method::from_builtin(class_init),
        mc,
    );

    let mut write = class.write(mc);

    write.define_instance_trait(Trait::from_method(
        QName::new(Namespace::public(), "beginFill"),
        Method::from_builtin(begin_fill),
    ));
    write.define_instance_trait(Trait::from_method(
        QName::new(Namespace::public(), "clear"),
        Method::from_builtin(clear),
    ));

    class
}
