use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::property_decl::{DeclContext, StaticDeclarations};
use crate::avm1::{Object, Value};
use crate::avm1_stub;

const OBJECT_DECLS: StaticDeclarations = declare_static_properties! {
    "exactSettings" => property(get_exact_settings, set_exact_settings; VERSION_6);
    "useCodepage" => property(get_use_code_page, set_use_code_page);
    "setClipboard" => method(set_clipboard);
    "showSettings" => method(show_settings);
    // Pretty sure this is a variable
    "onStatus" => method(on_status);
};

pub fn create<'gc>(context: &mut DeclContext<'_, 'gc>) -> Object<'gc> {
    let system = Object::new(context.strings, Some(context.object_proto));
    context.define_properties_on(system, OBJECT_DECLS(context));
    system
}

#[derive(Debug, Copy, Clone, FromPrimitive)]
enum SettingsPanel {
    Privacy = 0,
    LocalStorage = 1,
    Microphone = 2,
    Camera = 3,
}

impl SettingsPanel {
    pub fn from_u8(n: u8) -> Option<Self> {
        num_traits::FromPrimitive::from_u8(n)
    }
}

pub fn set_clipboard<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let new_content = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?
        .to_string();

    activation.context.ui.set_clipboard_content(new_content);

    Ok(Value::Undefined)
}

pub fn show_settings<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    //TODO: should default to the last panel displayed
    let last_panel_pos = 0;

    let panel_pos = args
        .get(0)
        .unwrap_or(&last_panel_pos.into())
        .coerce_to_i32(activation)?;

    let _panel = SettingsPanel::from_u8(panel_pos as u8).unwrap_or(SettingsPanel::Privacy);

    avm1_stub!(activation, "System", "showSettings");
    Ok(Value::Undefined)
}

pub fn set_use_code_page<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let value = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .to_owned()
        .as_bool(activation.swf_version());

    activation.context.system.use_codepage = value;

    Ok(Value::Undefined)
}

pub fn get_use_code_page<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(activation.context.system.use_codepage.into())
}

pub fn set_exact_settings<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let value = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .to_owned()
        .as_bool(activation.swf_version());

    activation.context.system.exact_settings = value;

    Ok(Value::Undefined)
}

pub fn get_exact_settings<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(activation.context.system.exact_settings.into())
}

pub fn on_status<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "System", "onStatus");
    Ok(Value::Undefined)
}
