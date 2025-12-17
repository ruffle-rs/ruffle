//! TextSnapshot object

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::property_decl::{DeclContext, StaticDeclarations, SystemClass};
use crate::avm1::{Object, Value};
use crate::avm1_stub;

const PROTO_DECLS: StaticDeclarations = declare_static_properties! {
    "getCount" => method(get_count; DONT_DELETE | VERSION_6);
    "setSelected" => method(set_selected; DONT_DELETE | VERSION_6);
    "getSelected" => method(get_selected; DONT_DELETE | VERSION_6);
    "getText" => method(get_text; DONT_DELETE | VERSION_6);
    "getSelectedText" => method(get_selected_text; DONT_DELETE | VERSION_6);
    "hitTestTextNearPos" => method(hit_test_text_near_pos; DONT_DELETE | VERSION_6);
    "findText" => method(find_text; DONT_DELETE | VERSION_6);
    "setSelectColor" => method(set_select_color; DONT_DELETE | VERSION_6);
    "getTextRunInfo" => method(get_text_run_info; DONT_DELETE | VERSION_6);
};

pub fn create_class<'gc>(
    context: &mut DeclContext<'_, 'gc>,
    super_proto: Object<'gc>,
) -> SystemClass<'gc> {
    let class = context.class(constructor, super_proto);
    context.define_properties_on(class.proto, PROTO_DECLS(context));
    class
}

fn constructor<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Value::Undefined)
}

fn get_count<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "TextSnapshot", "getCount");
    Ok(Value::Undefined)
}

fn set_selected<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "TextSnapshot", "setSelected");
    Ok(Value::Undefined)
}

fn get_selected<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "TextSnapshot", "getSelected");
    Ok(Value::Undefined)
}

fn get_text<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "TextSnapshot", "getText");
    Ok(Value::Undefined)
}

fn get_selected_text<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "TextSnapshot", "getSelectedText");
    Ok(Value::Undefined)
}

fn hit_test_text_near_pos<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "TextSnapshot", "hitTestTextNearPos");
    Ok(Value::Undefined)
}

fn set_select_color<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "TextSnapshot", "setSelectColor");
    Ok(Value::Undefined)
}

fn find_text<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "TextSnapshot", "findText");
    Ok(Value::Undefined)
}

fn get_text_run_info<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "TextSnapshot", "getTextRunInfo");
    Ok(Value::Undefined)
}
