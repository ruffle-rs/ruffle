use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::method::Method;
use crate::avm2::object::Object;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::QName;

fn void_init<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    unreachable!()
}

pub fn create_class<'gc>(activation: &mut Activation<'_, 'gc>) -> Class<'gc> {
    let mc = activation.context.gc_context;
    let class = Class::custom_new(
        QName::new(activation.avm2().public_namespace_base_version, "void"),
        None,
        Method::from_builtin(void_init, "", mc),
        mc,
    );

    class.mark_traits_loaded(activation.context.gc_context);

    class
}
