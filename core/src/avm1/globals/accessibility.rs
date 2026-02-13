//! Accessibility class

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::property_decl::{DeclContext, StaticDeclarations};
use crate::avm1::{Object, Value};
use crate::avm1_stub;

const OBJECT_DECLS: StaticDeclarations = declare_static_properties! {
    use fn method;
    "isActive" => method(IS_ACTIVE; DONT_DELETE | READ_ONLY | VERSION_6);
    "sendEvent" => method(SEND_EVENT; DONT_DELETE | READ_ONLY | VERSION_6);
    "updateProperties" => method(UPDATE_PROPERTIES; DONT_DELETE | READ_ONLY | VERSION_6);
};

pub fn create<'gc>(context: &mut DeclContext<'_, 'gc>) -> Object<'gc> {
    let accessibility = Object::new(context.strings, Some(context.object_proto));
    context.define_properties_on(accessibility, OBJECT_DECLS(context));
    accessibility
}

pub mod method {
    pub const IS_ACTIVE: u16 = 0;
    pub const SEND_EVENT: u16 = 1;
    pub const UPDATE_PROPERTIES: u16 = 2;
}

pub fn method<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
    index: u16,
) -> Result<Value<'gc>, Error<'gc>> {
    use method::*;
    const CNAME: &str = "Accessibility";

    match index {
        IS_ACTIVE => {
            avm1_stub!(activation, CNAME, "isActive");
            return Ok(false.into());
        }
        SEND_EVENT => avm1_stub!(activation, CNAME, "sendEvent"),
        UPDATE_PROPERTIES => avm1_stub!(activation, CNAME, "updateProperties"),
        _ => (),
    }

    Ok(Value::Undefined)
}
