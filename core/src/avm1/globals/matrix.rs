//! flash.geom.Matrix object

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::globals::point::{point_to_object, value_to_point};
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Object, ScriptObject, TObject, Value};
use crate::string::{AvmString, StringContext};

use ruffle_render::matrix::Matrix;
use swf::Twips;

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "toString" => method(to_string);
    "identity" => method(identity);
    "clone" => method(clone);
    "scale" => method(scale);
    "rotate" => method(rotate);
    "translate" => method(translate);
    "concat" => method(concat);
    "invert" => method(invert);
    "createBox" => method(create_box);
    "createGradientBox" => method(create_gradient_box);
    "transformPoint" => method(transform_point);
    "deltaTransformPoint" => method(delta_transform_point);
};

pub fn value_to_matrix<'gc>(
    value: Value<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Matrix, Error<'gc>> {
    let a = value
        .coerce_to_object(activation)
        .get("a", activation)?
        .coerce_to_f64(activation)? as f32;
    let b = value
        .coerce_to_object(activation)
        .get("b", activation)?
        .coerce_to_f64(activation)? as f32;
    let c = value
        .coerce_to_object(activation)
        .get("c", activation)?
        .coerce_to_f64(activation)? as f32;
    let d = value
        .coerce_to_object(activation)
        .get("d", activation)?
        .coerce_to_f64(activation)? as f32;
    let tx = Twips::from_pixels(
        value
            .coerce_to_object(activation)
            .get("tx", activation)?
            .coerce_to_f64(activation)?,
    );
    let ty = Twips::from_pixels(
        value
            .coerce_to_object(activation)
            .get("ty", activation)?
            .coerce_to_f64(activation)?,
    );

    Ok(Matrix { a, b, c, d, tx, ty })
}

pub fn gradient_object_to_matrix<'gc>(
    object: Object<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Matrix, Error<'gc>> {
    if &object
        .get("matrixType", activation)?
        .coerce_to_string(activation)?
        == b"box"
    {
        let width = object.get("w", activation)?.coerce_to_f64(activation)?;
        let height = object.get("h", activation)?.coerce_to_f64(activation)?;
        let rotation = object.get("r", activation)?.coerce_to_f64(activation)?;
        let tx = object.get("x", activation)?.coerce_to_f64(activation)?;
        let ty = object.get("y", activation)?.coerce_to_f64(activation)?;
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
    let a = object.get("a", activation)?.coerce_to_f64(activation)? as f32;
    let b = object.get("b", activation)?.coerce_to_f64(activation)? as f32;
    let c = object.get("c", activation)?.coerce_to_f64(activation)? as f32;
    let d = object.get("d", activation)?.coerce_to_f64(activation)? as f32;
    let tx = Twips::from_pixels(object.get("tx", activation)?.coerce_to_f64(activation)?);
    let ty = Twips::from_pixels(object.get("ty", activation)?.coerce_to_f64(activation)?);

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
        object.get_local_stored("a", activation, false),
        object.get_local_stored("b", activation, false),
        object.get_local_stored("c", activation, false),
        object.get_local_stored("d", activation, false),
        object.get_local_stored("tx", activation, false),
        object.get_local_stored("ty", activation, false),
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
    let args = [
        matrix.a.into(),
        matrix.b.into(),
        matrix.c.into(),
        matrix.d.into(),
        matrix.tx.to_pixels().into(),
        matrix.ty.to_pixels().into(),
    ];
    let constructor = activation.context.avm1.prototypes().matrix_constructor;
    let object = constructor.construct(activation, &args)?;
    Ok(object)
}

pub fn apply_matrix_to_object<'gc>(
    matrix: Matrix,
    object: Object<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<(), Error<'gc>> {
    object.set("a", matrix.a.into(), activation)?;
    object.set("b", matrix.b.into(), activation)?;
    object.set("c", matrix.c.into(), activation)?;
    object.set("d", matrix.d.into(), activation)?;
    object.set("tx", matrix.tx.to_pixels().into(), activation)?;
    object.set("ty", matrix.ty.to_pixels().into(), activation)?;
    Ok(())
}

fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.is_empty() {
        apply_matrix_to_object(Matrix::IDENTITY, this, activation)?;
    } else {
        if let Some(a) = args.get(0) {
            this.set("a", *a, activation)?;
        }
        if let Some(b) = args.get(1) {
            this.set("b", *b, activation)?;
        }
        if let Some(c) = args.get(2) {
            this.set("c", *c, activation)?;
        }
        if let Some(d) = args.get(3) {
            this.set("d", *d, activation)?;
        }
        if let Some(tx) = args.get(4) {
            this.set("tx", *tx, activation)?;
        }
        if let Some(ty) = args.get(5) {
            this.set("ty", *ty, activation)?;
        }
    }

    Ok(this.into())
}

fn identity<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    apply_matrix_to_object(Matrix::IDENTITY, this, activation)?;
    Ok(Value::Undefined)
}

fn clone<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let args = [
        this.get("a", activation)?,
        this.get("b", activation)?,
        this.get("c", activation)?,
        this.get("d", activation)?,
        this.get("tx", activation)?,
        this.get("ty", activation)?,
    ];
    let constructor = activation.context.avm1.prototypes().matrix_constructor;
    let cloned = constructor.construct(activation, &args)?;
    Ok(cloned)
}

fn scale<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let scale_x = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_f64(activation)?;
    let scale_y = args
        .get(1)
        .unwrap_or(&Value::Undefined)
        .coerce_to_f64(activation)?;
    let mut matrix = Matrix::scale(scale_x as f32, scale_y as f32);
    matrix *= object_to_matrix(this, activation)?;
    apply_matrix_to_object(matrix, this, activation)?;

    Ok(Value::Undefined)
}

fn rotate<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let angle = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_f64(activation)?;
    let mut matrix = Matrix::rotate(angle as f32);
    matrix *= object_to_matrix(this, activation)?;
    apply_matrix_to_object(matrix, this, activation)?;

    Ok(Value::Undefined)
}

fn translate<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let translate_x = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_f64(activation)?;
    let translate_y = args
        .get(1)
        .unwrap_or(&Value::Undefined)
        .coerce_to_f64(activation)?;
    let mut matrix = Matrix::translate(
        Twips::from_pixels(translate_x),
        Twips::from_pixels(translate_y),
    );
    matrix *= object_to_matrix(this, activation)?;
    apply_matrix_to_object(matrix, this, activation)?;

    Ok(Value::Undefined)
}

fn concat<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let mut matrix = object_to_matrix(this, activation)?;
    let other = value_to_matrix(*args.get(0).unwrap_or(&Value::Undefined), activation)?;
    matrix = other * matrix;
    apply_matrix_to_object(matrix, this, activation)?;

    Ok(Value::Undefined)
}

fn invert<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // FIXME:
    // 1) `invert` and other Matrix methods need to operate on `f64`, not with `ruffle_render::Matrix`.
    // 2) If non-invertible, we are always setting to an identity matrix. But Flash only return identity
    //    if `c != 0 && b != 0`? Otherwise it results in a matrix with infinities.
    let matrix = object_to_matrix(this, activation)?
        .inverse()
        .unwrap_or_default();
    apply_matrix_to_object(matrix, this, activation)?;

    Ok(Value::Undefined)
}

fn create_box<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let scale_x = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_f64(activation)?;
    let scale_y = args
        .get(1)
        .unwrap_or(&Value::Undefined)
        .coerce_to_f64(activation)?;
    // [NA] Docs say rotation is optional and defaults to 0, but that's wrong?
    let rotation = args
        .get(2)
        .unwrap_or(&Value::Undefined)
        .coerce_to_f64(activation)?;
    let translate_x = if let Some(value) = args.get(3) {
        value.coerce_to_f64(activation)?
    } else {
        0.0
    };
    let translate_y = if let Some(value) = args.get(4) {
        value.coerce_to_f64(activation)?
    } else {
        0.0
    };

    let matrix = Matrix::create_box_with_rotation(
        scale_x as f32,
        scale_y as f32,
        rotation as f32,
        Twips::from_pixels(translate_x),
        Twips::from_pixels(translate_y),
    );
    apply_matrix_to_object(matrix, this, activation)?;

    Ok(Value::Undefined)
}

fn create_gradient_box<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let width = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_f64(activation)?;
    let height = args
        .get(1)
        .unwrap_or(&Value::Undefined)
        .coerce_to_f64(activation)?;
    let rotation = if let Some(value) = args.get(2) {
        value.coerce_to_f64(activation)?
    } else {
        0.0
    };
    let translate_x = if let Some(value) = args.get(3) {
        value.coerce_to_f64(activation)?
    } else {
        0.0
    };
    let translate_y = if let Some(value) = args.get(4) {
        value.coerce_to_f64(activation)?
    } else {
        0.0
    };

    let matrix = Matrix::create_gradient_box(
        width as f32,
        height as f32,
        rotation as f32,
        Twips::from_pixels(translate_x),
        Twips::from_pixels(translate_y),
    );
    apply_matrix_to_object(matrix, this, activation)?;

    Ok(Value::Undefined)
}

fn transform_point<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let matrix = object_to_matrix(this, activation)?;
    let point = value_to_point(
        args.get(0).unwrap_or(&Value::Undefined).to_owned(),
        activation,
    )?;

    let x = point.0 * matrix.a as f64 + point.1 * matrix.c as f64 + matrix.tx.to_pixels();
    let y = point.0 * matrix.b as f64 + point.1 * matrix.d as f64 + matrix.ty.to_pixels();
    let object = point_to_object((x, y), activation)?;
    Ok(object)
}

fn delta_transform_point<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let matrix = object_to_matrix(this, activation)?;
    let point = value_to_point(
        args.get(0).unwrap_or(&Value::Undefined).to_owned(),
        activation,
    )?;

    let x = point.0 * matrix.a as f64 + point.1 * matrix.c as f64;
    let y = point.0 * matrix.b as f64 + point.1 * matrix.d as f64;
    let object = point_to_object((x, y), activation)?;
    Ok(object)
}

fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let a = this.get("a", activation)?;
    let b = this.get("b", activation)?;
    let c = this.get("c", activation)?;
    let d = this.get("d", activation)?;
    let tx = this.get("tx", activation)?;
    let ty = this.get("ty", activation)?;

    Ok(AvmString::new_utf8(
        activation.context.gc_context,
        format!(
            "(a={}, b={}, c={}, d={}, tx={}, ty={})",
            a.coerce_to_string(activation)?,
            b.coerce_to_string(activation)?,
            c.coerce_to_string(activation)?,
            d.coerce_to_string(activation)?,
            tx.coerce_to_string(activation)?,
            ty.coerce_to_string(activation)?
        ),
    )
    .into())
}

pub fn create_matrix_object<'gc>(
    context: &mut StringContext<'gc>,
    matrix_proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    FunctionObject::constructor(
        context.gc_context,
        Executable::Native(constructor),
        constructor_to_fn!(constructor),
        fn_proto,
        matrix_proto,
    )
}

pub fn create_proto<'gc>(
    context: &mut StringContext<'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let object = ScriptObject::new(context.gc_context, Some(proto));
    define_properties_on(PROTO_DECLS, context, object, fn_proto);
    object.into()
}
