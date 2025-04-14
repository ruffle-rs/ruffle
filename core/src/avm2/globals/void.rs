use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::QName;
use ruffle_macros::istr;

pub fn create_class<'gc>(activation: &mut Activation<'_, 'gc>) -> Class<'gc> {
    let mc = activation.gc();
    let class = Class::custom_new(
        QName::new(activation.avm2().namespaces.public_all(), istr!("void")),
        None,
        vec![],
        mc,
    );
    class.set_attributes(mc, ClassAttributes::FINAL | ClassAttributes::SEALED);

    class
}
