//! `flash.ui.Keyboard` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::value::Value;
use crate::avm2::{Error, Object};
use crate::string::AvmString;

pub fn get_caps_lock<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    log::warn!("Keyboard.capsLock: not yet implemented");
    Ok(false.into())
}

pub fn get_has_virtual_keyboard<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    log::warn!("Keyboard.hasVirtualKeyboard: not yet implemented");
    Ok(false.into())
}

pub fn get_num_lock<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    log::warn!("Keyboard.numLock: not yet implemented");
    Ok(false.into())
}

pub fn get_physical_keyboard_type<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    log::warn!("Keyboard.physicalKeyboardType: not yet implemented");
    Ok(AvmString::new_utf8(activation.context.gc_context, "alphanumeric").into())
}

pub fn is_accessible<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    log::warn!("Keyboard.isAccessible: not yet implemented");
    Ok(true.into())
}
