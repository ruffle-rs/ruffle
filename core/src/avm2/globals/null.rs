use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::QName;
use ruffle_macros::istr;

pub fn create_class<'gc>(activation: &mut Activation<'_, 'gc>) -> Class<'gc> {
    let mc = activation.gc();
    let class = Class::custom_new(
        QName::new(activation.avm2().namespaces.public_all(), istr!("null")),
        None,
        None,
        Box::new([]),
        mc,
    );
    class.set_attributes(ClassAttributes::FINAL | ClassAttributes::SEALED);

    // The `null` class has no interfaces, so use `init_vtable_with_interfaces`
    // and pass an empty list
    class.init_vtable_with_interfaces(activation.context, Box::new([]));

    class
}
