//! flash.geom.Matrix

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::{Object, Value};

use ruffle_macros::istr;
use ruffle_render::matrix::Matrix;
use swf::Twips;

pub fn gradient_object_to_matrix<'gc>(
    object: Object<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Matrix, Error<'gc>> {
    if &object
        .get(istr!("matrixType"), activation)?
        .coerce_to_string(activation)?
        == b"box"
    {
        let width = object
            .get(istr!("w"), activation)?
            .coerce_to_f64(activation)?;
        let height = object
            .get(istr!("h"), activation)?
            .coerce_to_f64(activation)?;
        let rotation = object
            .get(istr!("r"), activation)?
            .coerce_to_f64(activation)?;
        let tx = object
            .get(istr!("x"), activation)?
            .coerce_to_f64(activation)?;
        let ty = object
            .get(istr!("y"), activation)?
            .coerce_to_f64(activation)?;
        Ok(Matrix::create_gradient_box(
            width as f32,
            height as f32,
            rotation as f32,
            Twips::from_pixels(tx),
            Twips::from_pixels(ty),
        ))
    } else {
        // TODO: You can also pass a 3x3 matrix here. How does it work?
        // For instance: {a:200, b:0, c:0, d:0, e:200, f:0, g:200, h:200, i:1}
        object_to_matrix(object, activation)
    }
}

pub fn object_to_matrix<'gc>(
    object: Object<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Matrix, Error<'gc>> {
    let a = object
        .get(istr!("a"), activation)?
        .coerce_to_f64(activation)? as f32;
    let b = object
        .get(istr!("b"), activation)?
        .coerce_to_f64(activation)? as f32;
    let c = object
        .get(istr!("c"), activation)?
        .coerce_to_f64(activation)? as f32;
    let d = object
        .get(istr!("d"), activation)?
        .coerce_to_f64(activation)? as f32;
    let tx = Twips::from_pixels(
        object
            .get(istr!("tx"), activation)?
            .coerce_to_f64(activation)?,
    );
    let ty = Twips::from_pixels(
        object
            .get(istr!("ty"), activation)?
            .coerce_to_f64(activation)?,
    );

    Ok(Matrix { a, b, c, d, tx, ty })
}

/// Returns a `Matrix` with the properties from `object`.
///
/// Returns the identity matrix if any of the `a`, `b`, `c`, `d`, `tx` or `ty` properties do not exist.
pub fn object_to_matrix_or_default<'gc>(
    object: Object<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Matrix, Error<'gc>> {
    if let (Some(a), Some(b), Some(c), Some(d), Some(tx), Some(ty)) = (
        // These lookups do not search the prototype chain and ignore virtual properties.
        object.get_local_stored(istr!("a"), activation),
        object.get_local_stored(istr!("b"), activation),
        object.get_local_stored(istr!("c"), activation),
        object.get_local_stored(istr!("d"), activation),
        object.get_local_stored(istr!("tx"), activation),
        object.get_local_stored(istr!("ty"), activation),
    ) {
        let a = a.coerce_to_f64(activation)? as f32;
        let b = b.coerce_to_f64(activation)? as f32;
        let c = c.coerce_to_f64(activation)? as f32;
        let d = d.coerce_to_f64(activation)? as f32;
        let tx = Twips::from_pixels(tx.coerce_to_f64(activation)?);
        let ty = Twips::from_pixels(ty.coerce_to_f64(activation)?);
        Ok(Matrix { a, b, c, d, tx, ty })
    } else {
        Ok(Matrix::IDENTITY)
    }
}

pub fn matrix_to_value<'gc>(
    matrix: &Matrix,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let path = [istr!("flash"), istr!("geom"), istr!("Matrix")];
    let args = [
        matrix.a.into(),
        matrix.b.into(),
        matrix.c.into(),
        matrix.d.into(),
        matrix.tx.to_pixels().into(),
        matrix.ty.to_pixels().into(),
    ];
    activation.instantiate_class_as_script(path, &args)
}
