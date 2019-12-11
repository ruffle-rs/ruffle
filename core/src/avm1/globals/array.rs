//! Array prototype

use crate::avm1::property::Attribute;

use crate::avm1::return_value::ReturnValue;
use crate::avm1::{ArrayObject, Avm1, Error, Object, TObject, UpdateContext, Value};

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
            this.set_length(context.gc_context, length as i32);
            consumed = true;
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
    let new_length = old_length + args.len() as i32;

    for i in 0..args.len() {
        this.define_value(
            context.gc_context,
            &(old_length + i as i32).to_string(),
            args.get(i).unwrap().to_owned(),
            EnumSet::empty(),
        );
    }

    this.set_length(context.gc_context, new_length);

    Ok(new_length.into())
}

pub fn unshift<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let old_length = this.get_length();
    let new_length = old_length + args.len() as i32;
    let offset = new_length - old_length;

    for i in (old_length - 1..new_length).rev() {
        let old = this
            .get(&(i - offset).to_string(), avm, context)
            .and_then(|v| v.resolve(avm, context))
            .unwrap_or(Value::Undefined);
        this.define_value(context.gc_context, &i.to_string(), old, EnumSet::empty());
    }

    for i in 0..args.len() {
        this.define_value(
            context.gc_context,
            &(i as i32).to_string(),
            args.get(i).unwrap().to_owned(),
            EnumSet::empty(),
        );
    }

    this.set_length(context.gc_context, new_length);

    Ok(new_length.into())
}

pub fn shift<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let old_length = this.get_length();
    if old_length <= 0 {
        return Ok(Value::Undefined.into());
    }

    let new_length = old_length - 1;

    let removed = this
        .get("0", avm, context)
        .and_then(|v| v.resolve(avm, context))
        .unwrap_or(Value::Undefined);

    for i in 0..new_length {
        let old = this
            .get(&(i + 1).to_string(), avm, context)
            .and_then(|v| v.resolve(avm, context))
            .unwrap_or(Value::Undefined);
        this.define_value(context.gc_context, &i.to_string(), old, EnumSet::empty());
    }

    this.delete(context.gc_context, &new_length.to_string());

    this.set_length(context.gc_context, new_length);

    Ok(removed.into())
}

pub fn pop<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let old_length = this.get_length();
    if old_length <= 0 {
        return Ok(Value::Undefined.into());
    }

    let new_length = old_length - 1;

    let removed = this
        .get(&new_length.to_string(), avm, context)
        .and_then(|v| v.resolve(avm, context))
        .unwrap_or(Value::Undefined);
    this.delete(context.gc_context, &new_length.to_string());

    this.set_length(context.gc_context, new_length);

    Ok(removed.into())
}

pub fn reverse<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let length = this.get_length();
    let mut values = Vec::with_capacity(length as usize);

    for i in 0..length {
        values.push(
            this.get(&i.to_string(), avm, context)
                .and_then(|v| v.resolve(avm, context))
                .unwrap_or(Value::Undefined),
        );
    }

    for i in 0..length {
        this.define_value(
            context.gc_context,
            &i.to_string(),
            values.pop().unwrap(),
            EnumSet::empty(),
        );
    }

    Ok(Value::Undefined.into())
}

pub fn join<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let length = this.get_length();
    if length < 0 {
        return Ok("".into());
    }

    let separator = args
        .get(0)
        .and_then(|v| v.to_owned().coerce_to_string(avm, context).ok())
        .unwrap_or_else(|| ",".to_owned());
    let mut values = Vec::with_capacity(length as usize);

    for i in 0..length {
        values.push(
            this.get(&i.to_string(), avm, context)
                .and_then(|v| v.resolve(avm, context))
                .and_then(|v| v.coerce_to_string(avm, context))
                .unwrap_or_else(|_| "undefined".to_string()),
        );
    }

    Ok(values.join(&separator).into())
}

pub fn slice<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let mut start = args
        .get(0)
        .and_then(|v| v.as_number(avm, context).ok())
        .map(|v| v as i32)
        .unwrap_or(0);
    let mut end = args
        .get(1)
        .and_then(|v| v.as_number(avm, context).ok())
        .map(|v| v as i32)
        .unwrap_or_else(|| this.get_length());

    if start < 0 {
        start += this.get_length();
    }
    if end < 0 {
        end += this.get_length();
    }

    let length = end - start;
    let array = ArrayObject::array(context.gc_context, Some(avm.prototypes.array));
    array.set_length(context.gc_context, length);

    for i in 0..length {
        let old = this
            .get(&(start + i).to_string(), avm, context)
            .and_then(|v| v.resolve(avm, context))
            .unwrap_or(Value::Undefined);
        array.define_value(context.gc_context, &i.to_string(), old, EnumSet::empty());
    }

    Ok(Value::Object(array.into()).into())
}

pub fn concat<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let array = ArrayObject::array(context.gc_context, Some(avm.prototypes.array));
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
    let array = ArrayObject::array(gc_context, Some(proto));
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
