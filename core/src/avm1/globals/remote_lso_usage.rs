use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::property_decl::{DeclContext, StaticDeclarations, SystemClass};
use crate::avm1::{Object, Value};
use crate::avm1_stub;

const OBJECT_DECLS: StaticDeclarations = declare_static_properties! {
    "getURLPageSupport" => method(get_url_page_support; DONT_ENUM);
};

pub fn create_class<'gc>(
    context: &mut DeclContext<'_, 'gc>,
    super_proto: Object<'gc>,
) -> SystemClass<'gc> {
    let class = context.empty_class(super_proto);
    context.define_properties_on(class.constr, OBJECT_DECLS(context));
    class
}

fn get_url_page_support<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "RemoteLSOUsage", "getURLPageSupport");
    Ok(Value::Undefined)
}
