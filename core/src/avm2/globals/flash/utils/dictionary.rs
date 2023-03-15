pub use crate::avm2::object::dictionary_allocator;

use crate::avm2::activation::Activation;
use crate::avm2::object::{Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;

pub fn make_weak<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.and_then(|t| t.as_dictionary_object()) {
        this.make_weak(activation.context.gc_context);
    }
    Ok(Value::Undefined)
}
