use crate::avm1::object::Object;
use crate::avm1::property::Attribute::{DontDelete, DontEnum, ReadOnly};
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Error, ScriptObject, TObject, Value};
use crate::context::UpdateContext;
use gc_arena::MutationContext;
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;
use crate::avm1::function::Executable;

#[derive(Debug, Copy, Clone, TryFromPrimitive)]
#[repr(u8)]
enum SettingsPanel {
    Privacy = 0,
    LocalStorage = 1,
    Microphone = 2,
    Camera = 3,
}

/// The properties modified by 'System'
pub struct SystemProperties {
    /// If true then settings should be saved and read from the exact same domain of the player
    /// If false then they should be saved to the super domain
    pub exact_settings: bool,
    /// If true then the system codepage should be used instead of unicode for text files
    /// If false then unicode should be used
    pub use_codepage: bool
}

impl Default for SystemProperties {
    fn default() -> Self {
        SystemProperties {
            //TODO: default to true on fp>=7, false <= 6
            exact_settings: true,
            //TODO: default to false on fp>=7, true <= 6
            use_codepage: false
        }
    }
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

    Ok(Value::Undefined.into())
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
        .map(|v| match v {
            Value::Number(x) => SettingsPanel::try_from(*x as u8).unwrap_or(last_panel),
            _ => last_panel,
        })
        .unwrap_or(last_panel);

    log::warn!("System.showSettings({:?}) not not implemented", panel);
    Ok(Value::Undefined.into())
}

pub fn set_use_code_page<'gc>(
    avm: &mut Avm1<'gc>,
    action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let value = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .to_owned()
        .as_bool(avm.current_swf_version());

    action_context.system.use_codepage = value;

    Ok(Value::Undefined.into())
}

pub fn get_use_code_page<'gc>(
    _avm: &mut Avm1<'gc>,
    action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    Ok(action_context.system.use_codepage.into())
}

pub fn set_exact_settings<'gc>(
    avm: &mut Avm1<'gc>,
    action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let value = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .to_owned()
        .as_bool(avm.current_swf_version());

    action_context.system.exact_settings = value;

    Ok(Value::Undefined.into())
}

pub fn get_exact_settings<'gc>(
    _avm: &mut Avm1<'gc>,
    action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    Ok(action_context.system.exact_settings.into())
}

pub fn on_status<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("System.onStatus() not implemented");
    Ok(Value::Undefined.into())
}

pub fn create<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Option<Object<'gc>>,
    fn_proto: Option<Object<'gc>>,
) -> Object<'gc> {
    let mut system = ScriptObject::object(gc_context, proto);

    system.add_property(
        gc_context,
        "exactSettings",
        Executable::Native(get_exact_settings),
        Some(Executable::Native(set_exact_settings)),
        DontDelete | DontEnum,
    );


    system.add_property(
        gc_context,
        "useCodepage",
        Executable::Native(get_use_code_page),
        Some(Executable::Native(set_use_code_page)),
        DontDelete | DontEnum,
    );

    system.define_value(
        gc_context,
        "security",
        Value::Undefined,
        DontDelete | ReadOnly | DontEnum,
    );

    system.define_value(
        gc_context,
        "capabilities",
        crate::avm1::globals::system_capabilities::create(gc_context, proto).into(),
        DontDelete | ReadOnly | DontEnum,
    );

    system.define_value(
        gc_context,
        "IME",
        Value::Undefined,
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
