use crate::avm1::object::Object;
use crate::avm1::property::Attribute::{DontDelete, DontEnum, ReadOnly};
use crate::avm1::return_value::ReturnValue;
use crate::avm1::value::Value::{Bool, Undefined};
use crate::avm1::{Avm1, Error, ScriptObject, TObject, Value};
use crate::context::UpdateContext;
use gc_arena::MutationContext;
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;

#[derive(Debug, Copy, Clone, TryFromPrimitive)]
#[repr(u8)]
enum SettingsPanel {
    Privacy = 0,
    LocalStorage = 1,
    Microphone = 2,
    Camera = 3,
}

pub fn set_clipboard<'gc>(
    avm: &mut Avm1<'gc>,
    action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let new_content = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .to_owned()
        .coerce_to_string(avm, action_context)?;

    log::warn!("System.setClipboard({}) not yet implemented", new_content);

    Ok(Undefined.into())
}

pub fn show_settings<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    //TODO: should default to the last panel displayed
    let last_panel = SettingsPanel::Privacy;

    let panel = args
        .get(0)
        // .map(|value| value.as_number(avm, action_context))
        .map(|v| match v {
            Value::Number(x) => SettingsPanel::try_from(*x as u8).unwrap_or(last_panel),
            _ => last_panel,
        })
        .unwrap_or(last_panel);

    log::warn!("System.showSettings({:?}) not not implemented", panel);
    Ok(Undefined.into())
}

pub fn on_status<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("System.onStatus() not implemented");
    Ok(Undefined.into())
}

pub fn create<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Option<Object<'gc>>,
    fn_proto: Option<Object<'gc>>,
) -> Object<'gc> {
    let mut system = ScriptObject::object(gc_context, proto);

    //TODO: default to true on fp>=7, false <= 6
    // TODO: if true, settings are linked to exact domains (abc.example.com),
    // if false then use the settings for the root domain (example.com)
    system.define_value(
        gc_context,
        "exactSettings",
        Bool(true),
        DontDelete | DontEnum,
    );

    //TODO: default to false on fp>=7, true <= 6
    // when true, external text data should be loaded using the system codepage instead of unicode
    system.define_value(
        gc_context,
        "useCodepage",
        Bool(false),
        DontDelete | DontEnum,
    );

    system.define_value(
        gc_context,
        "security",
        Undefined,
        DontDelete | ReadOnly | DontEnum,
    );

    system.define_value(
        gc_context,
        "capabilities",
        Undefined,
        DontDelete | ReadOnly | DontEnum,
    );

    system.define_value(
        gc_context,
        "IME",
        Undefined,
        DontDelete | ReadOnly | DontEnum,
    );

    system.force_set_function(
        "setClipboard",
        set_clipboard,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        fn_proto,
    );

    system.force_set_function(
        "showSettings",
        show_settings,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        fn_proto,
    );

    // Pretty sure this is a variable
    system.force_set_function(
        "onStatus",
        on_status,
        gc_context,
        DontDelete | DontEnum,
        fn_proto,
    );

    system.into()
}
