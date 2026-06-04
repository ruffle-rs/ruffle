use crate::avm1::Object;
use crate::avm1::property_decl::{DeclContext, SystemClass};

pub fn create_class<'gc>(
    context: &mut DeclContext<'_, 'gc>,
    super_proto: Object<'gc>,
) -> SystemClass<'gc> {
    context.empty_class(super_proto)
}
