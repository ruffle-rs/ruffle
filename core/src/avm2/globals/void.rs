use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::Method;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::QName;

fn void_init<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    unreachable!()
}

pub fn create_class<'gc>(activation: &mut Activation<'_, 'gc>) -> Class<'gc> {
    let mc = activation.gc();
    let class = Class::custom_new(
        QName::new(activation.avm2().namespaces.public_all(), "void"),
        None,
        Method::from_builtin(void_init, "", mc),
        mc,
    );
    class.set_attributes(mc, ClassAttributes::FINAL | ClassAttributes::SEALED);

    class.mark_traits_loaded(activation.gc());

    class
}
