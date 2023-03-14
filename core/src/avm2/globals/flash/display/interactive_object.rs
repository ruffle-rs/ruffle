//! `flash.display.InteractiveObject` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::object::{Object, TObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::Multiname;
use crate::avm2::Namespace;
use crate::display_object::{TDisplayObject, TInteractiveObject};
use crate::{avm2_stub_getter, avm2_stub_setter};

/// Implements `flash.display.InteractiveObject`'s native instance constructor.
pub fn native_instance_init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        activation.super_init(this, &[])?;
    }

    Ok(Value::Undefined)
}

/// Implements `InteractiveObject.mouseEnabled`'s getter.
pub fn get_mouse_enabled<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(int) = this
        .and_then(|t| t.as_display_object())
        .and_then(|dobj| dobj.as_interactive())
    {
        return Ok(int.mouse_enabled().into());
    }

    Ok(Value::Undefined)
}

/// Implements `InteractiveObject.mouseEnabled`'s setter.
pub fn set_mouse_enabled<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(int) = this
        .and_then(|t| t.as_display_object())
        .and_then(|dobj| dobj.as_interactive())
    {
        let value = args.get_bool(0);
        int.set_mouse_enabled(activation.context.gc_context, value);
    }

    Ok(Value::Undefined)
}

/// Implements `InteractiveObject.doubleClickEnabled`'s getter.
pub fn get_double_click_enabled<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(int) = this
        .and_then(|t| t.as_display_object())
        .and_then(|dobj| dobj.as_interactive())
    {
        return Ok(int.double_click_enabled().into());
    }

    Ok(Value::Undefined)
}

/// Implements `InteractiveObject.doubleClickEnabled`'s setter.
pub fn set_double_click_enabled<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(int) = this
        .and_then(|t| t.as_display_object())
        .and_then(|dobj| dobj.as_interactive())
    {
        let value = args.get_bool(0);
        int.set_double_click_enabled(activation.context.gc_context, value);
    }

    Ok(Value::Undefined)
}

/// Implements `InteractiveObject.contextMenu`'s getter.
pub fn get_context_menu<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(int) = this
        .and_then(|t| t.as_display_object())
        .and_then(|dobj| dobj.as_interactive())
    {
        return Ok(int.context_menu());
    }

    Ok(Value::Undefined)
}

/// Implements `InteractiveObject.contextMenu`'s setter.
pub fn set_context_menu<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(int) = this
        .and_then(|t| t.as_display_object())
        .and_then(|dobj| dobj.as_interactive())
    {
        let cls_name = Multiname::new(
            Namespace::package("flash.display", activation.context.gc_context),
            "NativeMenu",
        );
        let cls = activation.resolve_class(&cls_name)?;
        let value = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_type(activation, cls)?;
        int.set_context_menu(activation.context.gc_context, value);
    }

    Ok(Value::Undefined)
}

pub fn get_tab_enabled<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(activation, "flash.display.InteractiveObject", "tabEnabled");

    Ok(false.into())
}

pub fn set_tab_enabled<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_setter!(activation, "flash.display.InteractiveObject", "tabIndex");

    Ok(Value::Undefined)
}

pub fn get_tab_index<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(activation, "flash.display.InteractiveObject", "tabIndex");

    Ok((-1).into())
}

pub fn set_tab_index<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_setter!(activation, "flash.display.InteractiveObject", "tabIndex");

    Ok(Value::Undefined)
}

pub fn get_focus_rect<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(activation, "flash.display.InteractiveObject", "focusRect");
    Ok(Value::Null)
}

pub fn set_focus_rect<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // NOTE: all values other than true or null are converted to false. (false/null do differ)

    // let's only warn on true, as games sometimes just set focusRect to false for some reason.
    if matches!(args.get(0), Some(Value::Bool(true))) {
        avm2_stub_setter!(activation, "flash.display.InteractiveObject", "focusRect");
    }

    Ok(Value::Null)
}
