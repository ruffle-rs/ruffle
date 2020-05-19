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
    object_to_matrix(value.as_object()?, avm, context)
}

pub fn object_to_matrix<'gc>(
    object: Object<'gc>,
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
) -> Result<Matrix, Error> {
    let a = object
        .get("a", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)? as f32;
    let b = object
        .get("b", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)? as f32;
    let c = object
        .get("c", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)? as f32;
    let d = object
        .get("d", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)? as f32;
    let tx = Twips::from_pixels(
        object
            .get("tx", avm, context)?
            .resolve(avm, context)?
            .as_number(avm, context)?,
    );
    let ty = Twips::from_pixels(
        object
            .get("ty", avm, context)?
            .resolve(avm, context)?
            .as_number(avm, context)?,
    );

    Ok(Matrix { a, b, c, d, tx, ty })
}

pub fn matrix_to_object<'gc>(
    matrix: Matrix,
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
) -> Result<Object<'gc>, Error> {
    let proto = context.system_prototypes.matrix;
    proto.new(
        avm,
        context,
        proto,
        &[
            matrix.a.into(),
            matrix.b.into(),
            matrix.c.into(),
            matrix.d.into(),
            matrix.tx.to_pixels().into(),
            matrix.ty.to_pixels().into(),
        ],
    )
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

fn to_string<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let a = this
        .get("a", avm, context)?
        .resolve(avm, context)?
        .coerce_to_string(avm, context)?;
    let b = this
        .get("b", avm, context)?
        .resolve(avm, context)?
        .coerce_to_string(avm, context)?;
    let c = this
        .get("c", avm, context)?
        .resolve(avm, context)?
        .coerce_to_string(avm, context)?;
    let d = this
        .get("d", avm, context)?
        .resolve(avm, context)?
        .coerce_to_string(avm, context)?;
    let tx = this
        .get("tx", avm, context)?
        .resolve(avm, context)?
        .coerce_to_string(avm, context)?;
    let ty = this
        .get("ty", avm, context)?
        .resolve(avm, context)?
        .coerce_to_string(avm, context)?;

    Ok(format!("(a={}, b={}, c={}, d={}, tx={}, ty={})", a, b, c, d, tx, ty).into())
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

    object.into()
}
