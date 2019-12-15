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
    let old_length = this.length();
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
    let old_length = this.length();
    let new_length = old_length + args.len();
    let offset = new_length - old_length;

    for i in (old_length - 1..new_length).rev() {
        this.set_array_element(i, this.array_element(i - offset), context.gc_context);
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
    let old_length = this.length();
    if old_length == 0 {
        return Ok(Value::Undefined.into());
    }

    let new_length = old_length - 1;

    let removed = this.array_element(0);

    for i in 0..new_length {
        this.set_array_element(i, this.array_element(i + 1), context.gc_context);
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
    let old_length = this.length();
    if old_length == 0 {
        return Ok(Value::Undefined.into());
    }

    let new_length = old_length - 1;

    let removed = this.array_element(new_length);
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
    let length = this.length();
    let mut values = this.array().to_vec();

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
    let values: Vec<Value<'gc>> = this.array();

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
        .map(|v| make_index_absolute(v as i32, this.length()))
        .unwrap_or(0);
    let end = args
        .get(1)
        .and_then(|v| v.as_number(avm, context).ok())
        .map(|v| make_index_absolute(v as i32, this.length()))
        .unwrap_or_else(|| this.length());

    let array = ScriptObject::array(context.gc_context, Some(avm.prototypes.array));

    if start < end {
        let length = end - start;
        array.set_length(context.gc_context, length);

        for i in 0..length {
            array.set_array_element(i, this.array_element(start + i), context.gc_context);
        }
    }

    Ok(array.into())
}

pub fn splice<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if args.is_empty() {
        return Ok(Value::Undefined.into());
    }

    let old_length = this.length();
    let start = args
        .get(0)
        .and_then(|v| v.as_number(avm, context).ok())
        .map(|v| make_index_absolute(v as i32, old_length))
        .unwrap_or(0);
    let count = args
        .get(1)
        .and_then(|v| v.as_number(avm, context).ok())
        .map(|v| v as i32)
        .unwrap_or(old_length as i32);
    if count < 0 {
        return Ok(Value::Undefined.into());
    }

    let removed = ScriptObject::array(context.gc_context, Some(avm.prototypes.array));
    let to_remove = count.min(old_length as i32 - start as i32).max(0) as usize;
    let to_add = if args.len() > 2 { &args[2..] } else { &[] };
    let offset = to_remove as i32 - to_add.len() as i32;
    let new_length = old_length + to_add.len() - to_remove;

    for i in start..start + to_remove {
        removed.set_array_element(i - start, this.array_element(i), context.gc_context);
    }
    removed.set_length(context.gc_context, to_remove);

    if to_remove == 2 && args.len() == 5 {
        dbg!(
            &start,
            &count,
            &removed,
            &to_remove,
            &to_add,
            &offset,
            &old_length,
            &new_length
        );
    }

    if offset < 0 {
        for i in (start + to_add.len()..new_length).rev() {
            if to_remove == 2 && args.len() == 5 {
                dbg!(&i, this.array_element((i as i32 + offset) as usize));
            }
            this.set_array_element(
                i,
                this.array_element((i as i32 + offset) as usize),
                context.gc_context,
            );
        }
    } else {
        for i in start + to_add.len()..new_length {
            this.set_array_element(
                i,
                this.array_element((i as i32 + offset) as usize),
                context.gc_context,
            );
        }
    }

    for i in 0..to_add.len() {
        this.set_array_element(
            start + i,
            to_add.get(i).unwrap().to_owned(),
            context.gc_context,
        );
    }

    for i in new_length..old_length {
        this.delete_array_element(i, context.gc_context);
        this.delete(context.gc_context, &i.to_string());
    }

    this.set_length(context.gc_context, new_length);

    Ok(removed.into())
}

pub fn concat<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let array = ScriptObject::array(context.gc_context, Some(avm.prototypes.array));
    let mut length = 0;

    for i in 0..this.length() {
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
                for i in 0..object.length() {
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

    Ok(array.into())
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
        "splice",
        splice,
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
