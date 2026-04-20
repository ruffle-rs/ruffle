use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::property_decl::{DeclContext, StaticDeclarations, SystemClass};
use crate::avm1::{Object, Value};
use crate::avm1_stub;

const PROTO_DECLS: StaticDeclarations = declare_static_properties! {
    use fn method;
    "getTestAutomationConfiguration" => method(GET_TEST_AUTOMATION_CONFIGURATION);
    "getDeviceConfiguration" => method(GET_DEVICE_CONFIGURATION);
    "setDeviceConfiguration" => method(SET_DEVICE_CONFIGURATION);
    "valueOf" => method(VALUE_OF);
    "toString" => method(TO_STRING);
};

pub fn create_class<'gc>(
    context: &mut DeclContext<'_, 'gc>,
    super_proto: Object<'gc>,
) -> SystemClass<'gc> {
    let class = context.empty_class(super_proto);
    context.define_properties_on(class.proto, PROTO_DECLS(context));
    class
}

pub mod method {
    pub const GET_TEST_AUTOMATION_CONFIGURATION: u16 = 0;
    pub const GET_DEVICE_CONFIGURATION: u16 = 1;
    pub const SET_DEVICE_CONFIGURATION: u16 = 2;
    pub const VALUE_OF: u16 = 4;
    pub const TO_STRING: u16 = 5;
}

pub fn method<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
    index: u16,
) -> Result<Value<'gc>, Error<'gc>> {
    use method::*;
    const CNAME: &str = "flash.automation.Configuration";

    match index {
        GET_TEST_AUTOMATION_CONFIGURATION => {
            avm1_stub!(activation, CNAME, "getTestAutomationConfiguration")
        }
        GET_DEVICE_CONFIGURATION => avm1_stub!(activation, CNAME, "getDeviceConfiguration"),
        SET_DEVICE_CONFIGURATION => avm1_stub!(activation, CNAME, "setDeviceConfiguration"),
        VALUE_OF => avm1_stub!(activation, CNAME, "valueOf"),
        TO_STRING => avm1_stub!(activation, CNAME, "toString"),
        _ => (),
    }

    Ok(Value::Undefined)
}
