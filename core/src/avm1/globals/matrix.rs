//! flash.geom.Matrix

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::{AvmString, Object, ScriptObject, TObject, Value};
use enumset::EnumSet;
use gc_arena::MutationContext;
use swf::{Matrix, Twips};

pub fn value_to_matrix<'gc>(
    value: Value<'gc>,
    activation: &mut Activation<'_, '_, 'gc, '_>,
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
    activation: &mut Activation<'_, '_, 'gc, '_>,
) -> Result<Matrix, Error<'gc>> {
    if object
        .get("matrixType", activation)?
        .coerce_to_string(activation)?
        == "box"
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
        // TODO: You can apparently pass a 3x3 matrix here. Did anybody actually? How does it work?
        object_to_matrix(object, activation)
    }
}

pub fn object_to_matrix<'gc>(
    object: Object<'gc>,
    activation: &mut Activation<'_, '_, 'gc, '_>,
) -> Result<Matrix, Error<'gc>> {
    let a = object.get("a", activation)?.coerce_to_f64(activation)? as f32;
    let b = object.get("b", activation)?.coerce_to_f64(activation)? as f32;
    let c = object.get("c", activation)?.coerce_to_f64(activation)? as f32;
    let d = object.get("d", activation)?.coerce_to_f64(activation)? as f32;
    let tx = Twips::from_pixels(object.get("tx", activation)?.coerce_to_f64(activation)?);
    let ty = Twips::from_pixels(object.get("ty", activation)?.coerce_to_f64(activation)?);

    Ok(Matrix { a, b, c, d, tx, ty })
}

// We'll need this soon!
#[allow(dead_code)]
pub fn matrix_to_object<'gc>(
    matrix: Matrix,
    activation: &mut Activation<'_, '_, 'gc, '_>,
) -> Result<Object<'gc>, Error<'gc>> {
    let args = [
        matrix.a.into(),
        matrix.b.into(),
        matrix.c.into(),
        matrix.d.into(),
        matrix.tx.to_pixels().into(),
        matrix.ty.to_pixels().into(),
    ];
    let constructor = activation.context.avm1.prototypes.matrix_constructor;
    let object = constructor.construct(activation, &args)?;
    Ok(object)
}

pub fn apply_matrix_to_object<'gc>(
    matrix: Matrix,
    object: Object<'gc>,
    activation: &mut Activation<'_, '_, 'gc, '_>,
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
    activation: &mut Activation<'_, '_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.is_empty() {
        apply_matrix_to_object(Matrix::identity(), this, activation)?;
    } else {
        if let Some(a) = args.get(0) {
            this.set("a", a.clone(), activation)?;
        }
        if let Some(b) = args.get(1) {
            this.set("b", b.clone(), activation)?;
        }
        if let Some(c) = args.get(2) {
            this.set("c", c.clone(), activation)?;
        }
        if let Some(d) = args.get(3) {
            this.set("d", d.clone(), activation)?;
        }
        if let Some(tx) = args.get(4) {
            this.set("tx", tx.clone(), activation)?;
        }
        if let Some(ty) = args.get(5) {
            this.set("ty", ty.clone(), activation)?;
        }
    }

    Ok(Value::Undefined)
}

fn identity<'gc>(
    activation: &mut Activation<'_, '_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    apply_matrix_to_object(Matrix::identity(), this, activation)?;
    Ok(Value::Undefined)
}

fn clone<'gc>(
    activation: &mut Activation<'_, '_, 'gc, '_>,
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
    let constructor = activation.context.avm1.prototypes.matrix_constructor;
    let cloned = constructor.construct(activation, &args)?;
    Ok(cloned.into())
}

fn scale<'gc>(
    activation: &mut Activation<'_, '_, 'gc, '_>,
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
    activation: &mut Activation<'_, '_, 'gc, '_>,
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
    activation: &mut Activation<'_, '_, 'gc, '_>,
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
    activation: &mut Activation<'_, '_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let mut matrix = object_to_matrix(this, activation)?;
    let other = value_to_matrix(args.get(0).unwrap_or(&Value::Undefined).clone(), activation)?;
    matrix = other * matrix;
    apply_matrix_to_object(matrix, this, activation)?;

    Ok(Value::Undefined)
}

fn invert<'gc>(
    activation: &mut Activation<'_, '_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let mut matrix = object_to_matrix(this, activation)?;
    matrix.invert();
    apply_matrix_to_object(matrix, this, activation)?;

    Ok(Value::Undefined)
}

fn create_box<'gc>(
    activation: &mut Activation<'_, '_, 'gc, '_>,
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

    let matrix = Matrix::create_box(
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
    activation: &mut Activation<'_, '_, 'gc, '_>,
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

fn to_string<'gc>(
    activation: &mut Activation<'_, '_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let a = this.get("a", activation)?;
    let b = this.get("b", activation)?;
    let c = this.get("c", activation)?;
    let d = this.get("d", activation)?;
    let tx = this.get("tx", activation)?;
    let ty = this.get("ty", activation)?;

    Ok(AvmString::new(
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
    gc_context: MutationContext<'gc, '_>,
    matrix_proto: Object<'gc>,
    fn_proto: Option<Object<'gc>>,
) -> Object<'gc> {
    FunctionObject::constructor(
        gc_context,
        Executable::Native(constructor),
        fn_proto,
        matrix_proto,
    )
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let mut object = ScriptObject::object(gc_context, Some(proto));

    object.force_set_function(
        "toString",
        to_string,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    object.force_set_function(
        "identity",
        identity,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    object.force_set_function("clone", clone, gc_context, EnumSet::empty(), Some(fn_proto));

    object.force_set_function("scale", scale, gc_context, EnumSet::empty(), Some(fn_proto));

    object.force_set_function(
        "rotate",
        rotate,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    object.force_set_function(
        "translate",
        translate,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    object.force_set_function(
        "concat",
        concat,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    object.force_set_function(
        "invert",
        invert,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    object.force_set_function(
        "createBox",
        create_box,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    object.force_set_function(
        "createGradientBox",
        create_gradient_box,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    object.into()
}
