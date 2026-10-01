//! flash.geom.Point

use crate::avm1::Value;
use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use ruffle_macros::istr;

pub fn point_to_object<'gc, T: Into<Value<'gc>>>(
    point: (T, T),
    activation: &mut Activation<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let path = [istr!("flash"), istr!("geom"), istr!("Point")];
    let args = [point.0.into(), point.1.into()];
    activation.instantiate_class_as_script(path, &args)
}
