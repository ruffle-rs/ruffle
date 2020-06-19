//! flash.geom.Matrix

use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Error, Object, ScriptObject, TObject, Value};
use crate::context::UpdateContext;
use enumset::EnumSet;
use gc_arena::MutationContext;
use swf::{Matrix, Twips};

pub fn value_to_matrix<'gc>(
    value: Value<'gc>,
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
) -> Result<Matrix, Error> {
    let a = value
        .coerce_to_object(avm, context)
        .get("a", avm, context)?
        .coerce_to_f64(avm, context)? as f32;
    let b = value
        .coerce_to_object(avm, context)
        .get("b", avm, context)?
        .coerce_to_f64(avm, context)? as f32;
    let c = value
        .coerce_to_object(avm, context)
        .get("c", avm, context)?
        .coerce_to_f64(avm, context)? as f32;
    let d = value
        .coerce_to_object(avm, context)
        .get("d", avm, context)?
        .coerce_to_f64(avm, context)? as f32;
    let tx = Twips::from_pixels(
        value
            .coerce_to_object(avm, context)
            .get("tx", avm, context)?
            .coerce_to_f64(avm, context)?,
    );
    let ty = Twips::from_pixels(
        value
            .coerce_to_object(avm, context)
            .get("ty", avm, context)?
            .coerce_to_f64(avm, context)?,
    );

    Ok(Matrix { a, b, c, d, tx, ty })
}

pub fn gradient_object_to_matrix<'gc>(
    object: Object<'gc>,
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
) -> Result<Matrix, Error> {
    if object
        .get("matrixType", avm, context)?
        .coerce_to_string(avm, context)?
        == "box"
    {
        let width = object.get("w", avm, context)?.coerce_to_f64(avm, context)?;
        let height = object.get("h", avm, context)?.coerce_to_f64(avm, context)?;
        let rotation = object.get("r", avm, context)?.coerce_to_f64(avm, context)?;
        let tx = object.get("x", avm, context)?.coerce_to_f64(avm, context)?;
        let ty = object.get("y", avm, context)?.coerce_to_f64(avm, context)?;
        Ok(Matrix::create_gradient_box(
            width as f32,
            height as f32,
            rotation as f32,
            Twips::from_pixels(tx),
            Twips::from_pixels(ty),
        ))
    } else {
        // TODO: You can apparently pass a 3x3 matrix here. Did anybody actually? How does it work?
        object_to_matrix(object, avm, context)
    }
}

pub fn object_to_matrix<'gc>(
    object: Object<'gc>,
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
) -> Result<Matrix, Error> {
    let a = object.get("a", avm, context)?.coerce_to_f64(avm, context)? as f32;
    let b = object.get("b", avm, context)?.coerce_to_f64(avm, context)? as f32;
    let c = object.get("c", avm, context)?.coerce_to_f64(avm, context)? as f32;
    let d = object.get("d", avm, context)?.coerce_to_f64(avm, context)? as f32;
    let tx = Twips::from_pixels(
        object
            .get("tx", avm, context)?
            .coerce_to_f64(avm, context)?,
    );
    let ty = Twips::from_pixels(
        object
            .get("ty", avm, context)?
            .coerce_to_f64(avm, context)?,
    );

    Ok(Matrix { a, b, c, d, tx, ty })
}

// We'll need this soon!
#[allow(dead_code)]
pub fn matrix_to_object<'gc>(
    matrix: Matrix,
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
) -> Result<Object<'gc>, Error> {
    let proto = context.system_prototypes.matrix;
    let args = [
        matrix.a.into(),
        matrix.b.into(),
        matrix.c.into(),
        matrix.d.into(),
        matrix.tx.to_pixels().into(),
        matrix.ty.to_pixels().into(),
    ];
    let object = proto.new(avm, context, proto, &args)?;
    let _ = constructor(avm, context, object, &args)?;
    Ok(object)
}

pub fn apply_matrix_to_object<'gc>(
    matrix: Matrix,
    object: Object<'gc>,
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
) -> Result<(), Error> {
    object.set("a", matrix.a.into(), avm, context)?;
    object.set("b", matrix.b.into(), avm, context)?;
    object.set("c", matrix.c.into(), avm, context)?;
    object.set("d", matrix.d.into(), avm, context)?;
    object.set("tx", matrix.tx.to_pixels().into(), avm, context)?;
    object.set("ty", matrix.ty.to_pixels().into(), avm, context)?;
    Ok(())
}

fn constructor<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if args.is_empty() {
        apply_matrix_to_object(Matrix::identity(), this, avm, context)?;
    } else {
        if let Some(a) = args.get(0) {
            this.set("a", a.clone(), avm, context)?;
        }
        if let Some(b) = args.get(1) {
            this.set("b", b.clone(), avm, context)?;
        }
        if let Some(c) = args.get(2) {
            this.set("c", c.clone(), avm, context)?;
        }
        if let Some(d) = args.get(3) {
            this.set("d", d.clone(), avm, context)?;
        }
        if let Some(tx) = args.get(4) {
            this.set("tx", tx.clone(), avm, context)?;
        }
        if let Some(ty) = args.get(5) {
            this.set("ty", ty.clone(), avm, context)?;
        }
    }

    Ok(Value::Undefined.into())
}

fn identity<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    apply_matrix_to_object(Matrix::identity(), this, avm, context)?;
    Ok(Value::Undefined.into())
}

fn clone<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let proto = context.system_prototypes.matrix;
    let args = [
        this.get("a", avm, context)?,
        this.get("b", avm, context)?,
        this.get("c", avm, context)?,
        this.get("d", avm, context)?,
        this.get("tx", avm, context)?,
        this.get("ty", avm, context)?,
    ];
    let cloned = proto.new(avm, context, proto, &args)?;
    let _ = constructor(avm, context, cloned, &args)?;
    Ok(cloned.into())
}

fn scale<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let scale_x = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_f64(avm, context)?;
    let scale_y = args
        .get(1)
        .unwrap_or(&Value::Undefined)
        .coerce_to_f64(avm, context)?;
    let mut matrix = Matrix::scale(scale_x as f32, scale_y as f32);
    matrix *= object_to_matrix(this, avm, context)?;
    apply_matrix_to_object(matrix, this, avm, context)?;

    Ok(Value::Undefined.into())
}

fn rotate<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let angle = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_f64(avm, context)?;
    let mut matrix = Matrix::rotate(angle as f32);
    matrix *= object_to_matrix(this, avm, context)?;
    apply_matrix_to_object(matrix, this, avm, context)?;

    Ok(Value::Undefined.into())
}

fn translate<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let translate_x = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_f64(avm, context)?;
    let translate_y = args
        .get(1)
        .unwrap_or(&Value::Undefined)
        .coerce_to_f64(avm, context)?;
    let mut matrix = Matrix::translate(
        Twips::from_pixels(translate_x),
        Twips::from_pixels(translate_y),
    );
    matrix *= object_to_matrix(this, avm, context)?;
    apply_matrix_to_object(matrix, this, avm, context)?;

    Ok(Value::Undefined.into())
}

fn concat<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let mut matrix = object_to_matrix(this, avm, context)?;
    let other = value_to_matrix(
        args.get(0).unwrap_or(&Value::Undefined).clone(),
        avm,
        context,
    )?;
    matrix = other * matrix;
    apply_matrix_to_object(matrix, this, avm, context)?;

    Ok(Value::Undefined.into())
}

fn invert<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let mut matrix = object_to_matrix(this, avm, context)?;
    matrix.invert();
    apply_matrix_to_object(matrix, this, avm, context)?;

    Ok(Value::Undefined.into())
}

fn create_box<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let scale_x = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_f64(avm, context)?;
    let scale_y = args
        .get(1)
        .unwrap_or(&Value::Undefined)
        .coerce_to_f64(avm, context)?;
    // [NA] Docs say rotation is optional and defaults to 0, but that's wrong?
    let rotation = args
        .get(2)
        .unwrap_or(&Value::Undefined)
        .coerce_to_f64(avm, context)?;
    let translate_x = if let Some(value) = args.get(3) {
        value.coerce_to_f64(avm, context)?
    } else {
        0.0
    };
    let translate_y = if let Some(value) = args.get(4) {
        value.coerce_to_f64(avm, context)?
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
    apply_matrix_to_object(matrix, this, avm, context)?;

    Ok(Value::Undefined.into())
}

fn create_gradient_box<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let width = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_f64(avm, context)?;
    let height = args
        .get(1)
        .unwrap_or(&Value::Undefined)
        .coerce_to_f64(avm, context)?;
    let rotation = if let Some(value) = args.get(2) {
        value.coerce_to_f64(avm, context)?
    } else {
        0.0
    };
    let translate_x = if let Some(value) = args.get(3) {
        value.coerce_to_f64(avm, context)?
    } else {
        0.0
    };
    let translate_y = if let Some(value) = args.get(4) {
        value.coerce_to_f64(avm, context)?
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
    apply_matrix_to_object(matrix, this, avm, context)?;

    Ok(Value::Undefined.into())
}

fn to_string<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let a = this.get("a", avm, context)?;
    let b = this.get("b", avm, context)?;
    let c = this.get("c", avm, context)?;
    let d = this.get("d", avm, context)?;
    let tx = this.get("tx", avm, context)?;
    let ty = this.get("ty", avm, context)?;

    Ok(format!(
        "(a={}, b={}, c={}, d={}, tx={}, ty={})",
        a.coerce_to_string(avm, context)?,
        b.coerce_to_string(avm, context)?,
        c.coerce_to_string(avm, context)?,
        d.coerce_to_string(avm, context)?,
        tx.coerce_to_string(avm, context)?,
        ty.coerce_to_string(avm, context)?
    )
    .into())
}

pub fn create_matrix_object<'gc>(
    gc_context: MutationContext<'gc, '_>,
    matrix_proto: Option<Object<'gc>>,
    fn_proto: Option<Object<'gc>>,
) -> Object<'gc> {
    FunctionObject::function(
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
