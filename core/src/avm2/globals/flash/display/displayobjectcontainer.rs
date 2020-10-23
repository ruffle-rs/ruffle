//! `flash.display.DisplayObjectContainer` builtin/prototype

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

/// Implements `flash.display.DisplayObjectContainer`'s instance constructor.
pub fn instance_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements `flash.display.DisplayObjectContainer`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements `DisplayObjectContainer.getChildAt`
pub fn get_child_at<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        let id = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_i32(activation)?;
        let child = dobj.get_child_by_id(id + 1).ok_or_else(|| {
            format!(
                "RangeError: Display object container has no child with id {}",
                id
            )
        })?;

        return Ok(child.object2());
    }

    Ok(Value::Undefined)
}

/// Implements `DisplayObjectContainer.getChildByName`
pub fn get_child_by_name<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        let name = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_string(activation)?;
        let child = dobj.get_child_by_name(&name, false).ok_or_else(|| {
            format!(
                "RangeError: Display object container has no child with name {}",
                name
            )
        })?;

        return Ok(child.object2());
    }

    Ok(Value::Undefined)
}

/// Implements `DisplayObjectContainer.numChildren`
pub fn num_children<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(parent) = this.and_then(|this| this.as_display_object()) {
        return Ok(parent.children().count().into());
    }

    Ok(Value::Undefined)
}

/// Construct `DisplayObjectContainer`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(
            Namespace::package("flash.display"),
            "DisplayObjectContainer",
        ),
        Some(QName::new(Namespace::package("flash.display"), "InteractiveObject").into()),
        Method::from_builtin(instance_init),
        Method::from_builtin(class_init),
        mc,
    );

    let mut write = class.write(mc);

    write.define_instance_trait(Trait::from_method(
        QName::new(Namespace::public_namespace(), "getChildAt"),
        Method::from_builtin(get_child_at),
    ));
    write.define_instance_trait(Trait::from_method(
        QName::new(Namespace::public_namespace(), "getChildByName"),
        Method::from_builtin(get_child_by_name),
    ));
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public_namespace(), "numChildren"),
        Method::from_builtin(num_children),
    ));

    class
}
