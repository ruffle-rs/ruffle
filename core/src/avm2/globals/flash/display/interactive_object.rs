//! `flash.display.InteractiveObject` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::error::make_error_2027;
use crate::avm2::object::{Object, TObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::display_object::{TDisplayObject, TInteractiveObject};

/// Implements `flash.display.InteractiveObject`'s native instance constructor.
pub fn native_instance_init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    activation.super_init(this, &[])?;

    Ok(Value::Undefined)
}

/// Implements `InteractiveObject.mouseEnabled`'s getter.
pub fn get_mouse_enabled<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(int) = this
        .as_display_object()
        .and_then(|dobj| dobj.as_interactive())
    {
        return Ok(int.mouse_enabled().into());
    }

    Ok(Value::Undefined)
}

/// Implements `InteractiveObject.mouseEnabled`'s setter.
pub fn set_mouse_enabled<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(int) = this
        .as_display_object()
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
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(int) = this
        .as_display_object()
        .and_then(|dobj| dobj.as_interactive())
    {
        return Ok(int.double_click_enabled().into());
    }

    Ok(Value::Undefined)
}

/// Implements `InteractiveObject.doubleClickEnabled`'s setter.
pub fn set_double_click_enabled<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(int) = this
        .as_display_object()
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
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(int) = this
        .as_display_object()
        .and_then(|dobj| dobj.as_interactive())
    {
        return Ok(int.context_menu());
    }

    Ok(Value::Undefined)
}

/// Implements `InteractiveObject.contextMenu`'s setter.
pub fn set_context_menu<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(int) = this
        .as_display_object()
        .and_then(|dobj| dobj.as_interactive())
    {
        let value = args.get_value(0);
        int.set_context_menu(activation.context.gc_context, value);
    }

    Ok(Value::Undefined)
}

pub fn get_tab_enabled<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(obj) = this.as_display_object().and_then(|o| o.as_interactive()) {
        Ok(Value::Bool(obj.tab_enabled(activation.context)))
    } else {
        Ok(Value::Undefined)
    }
}

pub fn set_tab_enabled<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(obj) = this
        .as_display_object()
        .and_then(|this| this.as_interactive())
    {
        let value = args.get_bool(0);
        obj.set_tab_enabled(activation.context, value);
    }

    Ok(Value::Undefined)
}

pub fn get_tab_index<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(obj) = this
        .as_display_object()
        .and_then(|this| this.as_interactive())
    {
        Ok(Value::Number(obj.tab_index().unwrap_or(-1) as f64))
    } else {
        Ok(Value::Undefined)
    }
}

pub fn set_tab_index<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(obj) = this.as_display_object().and_then(|o| o.as_interactive()) {
        let value = args.get_i32(activation, 0)?;
        // Despite throwing an error that tabIndex cannot be negative,
        // the value of -1 is allowed, and it means that tabIndex is unset.
        if value < -1 {
            return Err(make_error_2027(activation, value));
        }
        obj.set_tab_index(activation.context, Some(value));
    }

    Ok(Value::Undefined)
}

pub fn get_focus_rect<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(obj) = this.as_display_object().and_then(|o| o.as_interactive()) {
        Ok(obj.focus_rect().map(Value::Bool).unwrap_or(Value::Null))
    } else {
        Ok(Value::Null)
    }
}

pub fn set_focus_rect<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(obj) = this
        .as_display_object()
        .and_then(|this| this.as_interactive())
    {
        let value = match args.get(0) {
            Some(Value::Bool(true)) => Some(true),
            Some(Value::Null) => None,
            // everything else sets focusRect to false
            _ => Some(false),
        };
        obj.set_focus_rect(activation.context.gc(), value);
    }

    Ok(Value::Null)
}
