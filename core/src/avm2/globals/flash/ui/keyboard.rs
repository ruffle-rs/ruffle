//! `flash.ui.Keyboard` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::value::Value;
use crate::avm2::{Error, Object};
use crate::avm2_stub_getter;
use crate::string::AvmString;

pub fn get_caps_lock<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(activation, "flash.ui.Keyboard", "capsLock");
    Ok(false.into())
}

pub fn get_has_virtual_keyboard<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(activation, "flash.ui.Keyboard", "hasVirtualKeyboard");
    Ok(false.into())
}

pub fn get_num_lock<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(activation, "flash.ui.Keyboard", "numLock");
    Ok(false.into())
}

pub fn get_physical_keyboard_type<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(activation, "flash.ui.Keyboard", "physicalKeyboardType");
    Ok(AvmString::new_utf8(activation.context.gc_context, "alphanumeric").into())
}

pub fn is_accessible<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(activation, "flash.ui.Keyboard", "isAccessible");
    Ok(true.into())
}
