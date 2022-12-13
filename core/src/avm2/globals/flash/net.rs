//! `flash.net` namespace

use crate::avm2::object::TObject;
use crate::avm2::{Activation, Error, Multiname, Object, Value};

pub mod object_encoding;
pub mod sharedobject;
pub mod url_loader;

/// Implements `flash.net.navigateToURL`
pub fn navigate_to_url<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let request = args
        .get(0)
        .ok_or("navigateToURL: not enough arguments")?
        .coerce_to_object(activation)?;

    let target = args
        .get(1)
        .ok_or("navigateToURL: not enough arguments")?
        .coerce_to_string(activation)?;

    let url = request
        .get_property(&Multiname::public("url"), activation)?
        .coerce_to_string(activation)?;

    activation
        .context
        .navigator
        .navigate_to_url(url.to_string(), target.to_string(), None);

    Ok(Value::Undefined)
}
