//! Array prototype

use crate::avm1::property::Attribute;

use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Error, Object, ScriptObject, TObject, UpdateContext, Value};

use enumset::EnumSet;
use gc_arena::MutationContext;

/// Implements `Array`
pub fn constructor<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let mut consumed = false;

    if args.len() == 1 {
        let arg = args.get(0).unwrap();
        if let Ok(length) = arg.as_number(avm, context) {
            if length >= 0.0 {
                this.set_length(context.gc_context, length as usize);
                consumed = true;
            }
        }
    }

    if !consumed {
        let mut length = 0;
        for arg in args {
            this.define_value(
                context.gc_context,
                &length.to_string(),
                arg.to_owned(),
                EnumSet::empty(),
            );
            length += 1;
        }
        this.set_length(context.gc_context, length);
    }

    Ok(Value::Undefined.into())
}

pub fn push<'gc>(
    _avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let old_length = this.get_length();
    let new_length = old_length + args.len();
    this.set_length(context.gc_context, new_length);

    for i in 0..args.len() {
        this.set_array_element(
            old_length + i,
            args.get(i).unwrap().to_owned(),
            context.gc_context,
        );
    }

    Ok((new_length as f64).into())
}

pub fn unshift<'gc>(
    _avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let old_length = this.get_length();
    let new_length = old_length + args.len();
    let offset = new_length - old_length;

    for i in (old_length - 1..new_length).rev() {
        this.set_array_element(
            i,
            this.get_array_element(dbg!(i - offset)),
            context.gc_context,
        );
    }

    for i in 0..args.len() {
        this.set_array_element(i, args.get(i).unwrap().to_owned(), context.gc_context);
    }

    this.set_length(context.gc_context, new_length);

    Ok((new_length as f64).into())
}

pub fn shift<'gc>(
    _avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let old_length = this.get_length();
    if old_length == 0 {
        return Ok(Value::Undefined.into());
    }

    let new_length = old_length - 1;

    let removed = this.get_array_element(0);

    for i in 0..new_length {
        this.set_array_element(i, this.get_array_element(i + 1), context.gc_context);
    }

    this.delete_array_element(new_length, context.gc_context);
    this.delete(context.gc_context, &new_length.to_string());

    this.set_length(context.gc_context, new_length);

    Ok(removed.into())
}

pub fn pop<'gc>(
    _avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let old_length = this.get_length();
    if old_length == 0 {
        return Ok(Value::Undefined.into());
    }

    let new_length = old_length - 1;

    let removed = this.get_array_element(new_length);
    this.delete_array_element(new_length, context.gc_context);
    this.delete(context.gc_context, &new_length.to_string());

    this.set_length(context.gc_context, new_length);

    Ok(removed.into())
}

pub fn reverse<'gc>(
    _avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let length = this.get_length();
    let mut values = this.get_array().to_vec();

    for i in 0..length {
        this.set_array_element(i, values.pop().unwrap(), context.gc_context);
    }

    Ok(Value::Undefined.into())
}

pub fn join<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let separator = args
        .get(0)
        .and_then(|v| v.to_owned().coerce_to_string(avm, context).ok())
        .unwrap_or_else(|| ",".to_owned());
    let values: Vec<Value<'gc>> = this.get_array();

    Ok(values
        .iter()
        .map(|v| {
            v.to_owned()
                .coerce_to_string(avm, context)
                .unwrap_or_else(|_| "undefined".to_string())
        })
        .collect::<Vec<String>>()
        .join(&separator)
        .into())
}

fn make_index_absolute(mut index: i32, length: usize) -> usize {
    if index < 0 {
        index += length as i32;
    }
    if index < 0 {
        0
    } else {
        index as usize
    }
}

pub fn slice<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let start = args
        .get(0)
        .and_then(|v| v.as_number(avm, context).ok())
        .map(|v| make_index_absolute(v as i32, this.get_length()))
        .unwrap_or(0);
    let end = args
        .get(1)
        .and_then(|v| v.as_number(avm, context).ok())
        .map(|v| make_index_absolute(v as i32, this.get_length()))
        .unwrap_or_else(|| this.get_length());

    let array = ScriptObject::array(context.gc_context, Some(avm.prototypes.array));

    if start < end {
        let length = end - start;
        array.set_length(context.gc_context, length);

        for i in 0..length {
            array.set_array_element(i, this.get_array_element(start + i), context.gc_context);
        }
    }

    Ok(Value::Object(array.into()).into())
}

pub fn concat<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let array = ScriptObject::array(context.gc_context, Some(avm.prototypes.array));
    let mut length = 0;

    for i in 0..this.get_length() {
        let old = this
            .get(&i.to_string(), avm, context)
            .and_then(|v| v.resolve(avm, context))
            .unwrap_or(Value::Undefined);
        array.define_value(
            context.gc_context,
            &length.to_string(),
            old,
            EnumSet::empty(),
        );
        length += 1;
    }

    for arg in args {
        let mut added = false;

        if let Value::Object(object) = arg {
            let object = *object;
            if avm.prototypes.array.is_prototype_of(object) {
                added = true;
                for i in 0..object.get_length() {
                    let old = object
                        .get(&i.to_string(), avm, context)
                        .and_then(|v| v.resolve(avm, context))
                        .unwrap_or(Value::Undefined);
                    array.define_value(
                        context.gc_context,
                        &length.to_string(),
                        old,
                        EnumSet::empty(),
                    );
                    length += 1;
                }
            }
        }

        if !added {
            array.define_value(
                context.gc_context,
                &length.to_string(),
                arg.clone(),
                EnumSet::empty(),
            );
            length += 1;
        }
    }

    array.set_length(context.gc_context, length);

    Ok(Value::Object(array.into()).into())
}

pub fn to_string<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    join(avm, context, this, &[])
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let array = ScriptObject::array(gc_context, Some(proto));
    let mut object = array.as_script_object().unwrap();

    object.force_set_function(
        "push",
        push,
        gc_context,
        Attribute::DontEnum,
        Some(fn_proto),
    );
    object.force_set_function(
        "unshift",
        unshift,
        gc_context,
        Attribute::DontEnum,
        Some(fn_proto),
    );
    object.force_set_function(
        "shift",
        shift,
        gc_context,
        Attribute::DontEnum,
        Some(fn_proto),
    );
    object.force_set_function("pop", pop, gc_context, Attribute::DontEnum, Some(fn_proto));
    object.force_set_function(
        "reverse",
        reverse,
        gc_context,
        Attribute::DontEnum,
        Some(fn_proto),
    );
    object.force_set_function(
        "join",
        join,
        gc_context,
        Attribute::DontEnum,
        Some(fn_proto),
    );
    object.force_set_function(
        "slice",
        slice,
        gc_context,
        Attribute::DontEnum,
        Some(fn_proto),
    );
    object.force_set_function(
        "concat",
        concat,
        gc_context,
        Attribute::DontEnum,
        Some(fn_proto),
    );
    object.force_set_function(
        "toString",
        to_string,
        gc_context,
        Attribute::DontEnum,
        Some(fn_proto),
    );

    array.into()
}
