//! `flash.ui.MouseCursorData` builtin

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::Object;
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{GcCell, MutationContext};

fn instance_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

fn data<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    log::warn!("MouseCursorData.data: not yet implemented");
    Ok(Value::Undefined)
}

fn set_data<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    log::warn!("MouseCursorData.data: not yet implemented");
    Ok(Value::Undefined)
}

fn frame_rate<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    log::warn!("MouseCursorData.frameRate: not yet implemented");
    Ok(Value::Undefined)
}

fn set_frame_rate<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    log::warn!("MouseCursorData.frameRate: not yet implemented");
    Ok(Value::Undefined)
}

fn hot_spot<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    log::warn!("MouseCursorData.hotSpot: not yet implemented");
    Ok(Value::Undefined)
}

fn set_hot_spot<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    log::warn!("MouseCursorData.hotSpot: not yet implemented");
    Ok(Value::Undefined)
}

pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.ui"), "MouseCursorData"),
        Some(QName::new(Namespace::package(""), "Object").into()),
        Method::from_builtin(instance_init, "<MouseCursorData instance initializer>", mc),
        Method::from_builtin(class_init, "<MouseCursorData class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    write.set_attributes(ClassAttributes::FINAL);

    const PUBLIC_INSTANCE_PROPERTIES: &[(
        &str,
        Option<NativeMethodImpl>,
        Option<NativeMethodImpl>,
    )] = &[
        ("data", Some(data), Some(set_data)),
        ("frameRate", Some(frame_rate), Some(set_frame_rate)),
        ("hotSpot", Some(hot_spot), Some(set_hot_spot)),
    ];
    write.define_public_builtin_instance_properties(mc, PUBLIC_INSTANCE_PROPERTIES);

    class
}
