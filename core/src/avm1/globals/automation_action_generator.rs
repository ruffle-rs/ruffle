use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::property_decl::{DeclContext, StaticDeclarations, SystemClass};
use crate::avm1::{Object, Value};
use crate::avm1_stub;

const PROTO_DECLS: StaticDeclarations = declare_static_properties! {
    use fn method;
    "generateAction" => method(GENERATE_ACTION);
    "generateActions" => method(GENERATE_ACTIONS);
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
    pub const GENERATE_ACTION: u16 = 0;
    pub const GENERATE_ACTIONS: u16 = 1;
    pub const VALUE_OF: u16 = 3;
    pub const TO_STRING: u16 = 4;
}

pub fn method<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
    index: u16,
) -> Result<Value<'gc>, Error<'gc>> {
    use method::*;
    const CNAME: &str = "flash.automation.ActionGenerator";

    match index {
        GENERATE_ACTION => avm1_stub!(activation, CNAME, "generateAction"),
        GENERATE_ACTIONS => avm1_stub!(activation, CNAME, "generateActions"),
        VALUE_OF => avm1_stub!(activation, CNAME, "valueOf"),
        TO_STRING => avm1_stub!(activation, CNAME, "toString"),
        _ => (),
    }

    Ok(Value::Undefined)
}
