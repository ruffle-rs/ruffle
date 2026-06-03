use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::property_decl::{DeclContext, PropertyOrder, StaticDeclarations, SystemClass};
use crate::avm1::{Object, Value};
use crate::avm1_stub;

const OBJECT_DECLS: StaticDeclarations = declare_static_properties! {
    "purge" => method(purge; DONT_ENUM);
    "getDiskUsage" => method(get_disk_usage; DONT_ENUM);
};

pub fn create_class<'gc>(
    context: &mut DeclContext<'_, 'gc>,
    super_proto: Object<'gc>,
) -> SystemClass<'gc> {
    let class = context.empty_class(super_proto, PropertyOrder::PrototypeLast);
    context.define_properties_on(class.constr, OBJECT_DECLS(context));
    class
}

fn get_disk_usage<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "AssetCache", "getDiskUsage");
    Ok(0.into())
}

fn purge<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "AssetCache", "purge");
    Ok(Value::Undefined)
}
