//! `flash.utils.Dictionary` native methods

pub use crate::avm2::object::dictionary_allocator;

use crate::avm2::Error;
use crate::avm2::activation::Activation;
use crate::avm2::object::DictionaryObject;
use crate::avm2::value::Value;

/// Custom constructor for `flash.utils.Dictionary`.
///
/// Reads the `weakKeys` argument and instantiates a `DictionaryObject`
/// with the corresponding flag set. This is the only place the
/// weak-keys flag is set on a fresh dictionary — the AS3 constructor
/// body is empty (the `[Ruffle(CustomConstructor)]` metadata routes
/// `new flash.utils.Dictionary(weakKeys)` here directly).
pub fn dictionary_constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let weak_keys = args.first().map(|v| v.coerce_to_boolean()).unwrap_or(false);

    let class = activation.avm2().classes().dictionary;
    let dict = DictionaryObject::new(class, weak_keys, activation);

    Ok(dict.into())
}
