//! Array class

use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::property::Attribute;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Error, Object, ScriptObject, TObject, UpdateContext, Value};
use enumset::EnumSet;
use gc_arena::MutationContext;
use std::cmp::Ordering;

// Flags used by `Array.sort` and `sortOn`.
const CASE_INSENSITIVE: i32 = 1;
const DESCENDING: i32 = 2;
const UNIQUE_SORT: i32 = 4;
const RETURN_INDEXED_ARRAY: i32 = 8;
const NUMERIC: i32 = 16;

// Default ordering to return if comparison is invalid.
// TODO: This won't work accurately in cases like NaN/undefined.
// We need to actually match Flash's sorting algorithm and not use Rust's Vec::sort.
const DEFAULT_ORDERING: Ordering = Ordering::Equal;

// Compare function used by sort and sortOn.
type CompareFn<'a, 'gc> = Box<
    dyn 'a
        + FnMut(&mut Avm1<'gc>, &mut UpdateContext<'_, 'gc, '_>, &Value<'gc>, &Value<'gc>) -> Ordering,
>;

pub fn create_array_object<'gc>(
    gc_context: MutationContext<'gc, '_>,
    array_proto: Option<Object<'gc>>,
    fn_proto: Option<Object<'gc>>,
) -> Object<'gc> {
    let array = FunctionObject::function(
        gc_context,
        Executable::Native(constructor),
        fn_proto,
        array_proto,
    );
    let object = array.as_script_object().unwrap();

    // TODO: These were added in Flash Player 7, but are available even to SWFv6 and lower
    // when run in Flash Player 7. Make these conditional if we add a parameter to control
    // target Flash Player version.
    object.define_value(
        gc_context,
        "CASEINSENSITIVE",
        1.into(),
        Attribute::DontEnum | Attribute::DontDelete | Attribute::ReadOnly,
    );

    object.define_value(
        gc_context,
        "DESCENDING",
        2.into(),
        Attribute::DontEnum | Attribute::DontDelete | Attribute::ReadOnly,
    );

    object.define_value(
        gc_context,
        "UNIQUESORT",
        4.into(),
        Attribute::DontEnum | Attribute::DontDelete | Attribute::ReadOnly,
    );

    object.define_value(
        gc_context,
        "RETURNINDEXEDARRAY",
        8.into(),
        Attribute::DontEnum | Attribute::DontDelete | Attribute::ReadOnly,
    );

    object.define_value(
        gc_context,
        "NUMERIC",
        16.into(),
        Attribute::DontEnum | Attribute::DontDelete | Attribute::ReadOnly,
    );

    array
}

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
    avm: &mut Avm1<'gc>,
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
    this.delete(avm, context.gc_context, &new_length.to_string());

    this.set_length(context.gc_context, new_length);

    Ok(removed.into())
}

pub fn pop<'gc>(
    avm: &mut Avm1<'gc>,
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
    this.delete(avm, context.gc_context, &new_length.to_string());

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

    if offset < 0 {
        for i in (start + to_add.len()..new_length).rev() {
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
        this.delete(avm, context.gc_context, &i.to_string());
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

fn sort<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    // Overloads:
    // 1) a.sort(flags: Number = 0): Sorts with the given flags.
    // 2) a.sort(compare_fn: Object, flags: Number = 0): Sorts using the given compare function and flags.
    use crate::avm1::value::f64_to_wrapping_i32;
    let (compare_fn, flags) = match args {
        [Value::Number(_), Value::Number(n), ..] => (None, f64_to_wrapping_i32(*n)),
        [Value::Number(n), ..] => (None, f64_to_wrapping_i32(*n)),
        [compare_fn @ Value::Object(_), Value::Number(n), ..] => {
            (Some(compare_fn), f64_to_wrapping_i32(*n))
        }
        [compare_fn @ Value::Object(_), ..] => (Some(compare_fn), 0),
        [] => (None, 0),
        _ => return Ok(Value::Undefined.into()),
    };

    let numeric = (flags & NUMERIC) != 0;
    let case_insensitive = (flags & CASE_INSENSITIVE) != 0;

    let string_compare_fn = if case_insensitive {
        sort_compare_string_ignore_case
    } else {
        sort_compare_string
    };

    let compare_fn: CompareFn<'_, 'gc> = if let Some(f) = compare_fn {
        let this = crate::avm1::value_object::ValueObject::boxed(avm, context, Value::Undefined);
        // this is undefined in the compare function
        Box::new(move |avm, context, a: &Value<'gc>, b: &Value<'gc>| {
            sort_compare_custom(avm, context, this, a, b, &f)
        })
    } else if numeric {
        Box::new(sort_compare_numeric(string_compare_fn))
    } else {
        Box::new(string_compare_fn)
    };

    sort_with_function(avm, context, this, compare_fn, flags)
}

fn sort_on<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    // a.sortOn(field_name, flags: Number = 0): Sorts with the given flags.
    // a.sortOn(field_names: Array, flags: Number = 0): Sorts with fields in order of precedence with the given flags.
    // a.sortOn(field_names: Array, flags: Array: Sorts with fields in order of precedence with the given flags respectively.
    let fields = match args.get(0) {
        Some(Value::Object(array)) => {
            // Array of field names.
            let mut field_names = vec![];
            for name in array.array() {
                field_names.push(name.coerce_to_string(avm, context)?);
            }
            field_names
        }
        Some(field_name) => {
            // Single field.
            vec![field_name.clone().coerce_to_string(avm, context)?]
        }
        None => return Ok(Value::Undefined.into()),
    };

    // Bail out if we don't have any fields.
    if fields.is_empty() {
        return Ok(this.into());
    }

    let flags = match args.get(1) {
        Some(Value::Object(array)) => {
            // Array of field names.
            if array.length() == fields.len() {
                let mut flags = vec![];
                for flag in array.array() {
                    flags.push(flag.coerce_to_i32(avm, context)?);
                }
                flags
            } else {
                // If the lengths of the flags and fields array do not match, the flags array is ignored.
                std::iter::repeat(0).take(fields.len()).collect()
            }
        }
        Some(flags) => {
            // Single field.
            let flags = flags.coerce_to_i32(avm, context)?;
            std::iter::repeat(flags).take(fields.len()).collect()
        }
        None => std::iter::repeat(0).take(fields.len()).collect(),
    };

    // CASEINSENSITIVE, UNIQUESORT, and RETURNINDEXEDARRAY are taken from the first set of flags in the array.
    let main_flags = flags[0];

    // Generate a list of compare functions to use for each field in the array.
    let field_compare_fns: Vec<CompareFn<'_, 'gc>> = flags
        .into_iter()
        .map(|flags| {
            let numeric = (flags & NUMERIC) != 0;
            let case_insensitive = (flags & CASE_INSENSITIVE) != 0;

            let string_compare_fn = if case_insensitive {
                sort_compare_string_ignore_case
            } else {
                sort_compare_string
            };

            if numeric {
                Box::new(sort_compare_numeric(string_compare_fn))
            } else {
                Box::new(string_compare_fn) as CompareFn<'_, 'gc>
            }
        })
        .collect();

    let compare_fn = sort_compare_fields(fields, field_compare_fns);

    sort_with_function(avm, context, this, compare_fn, main_flags)
}

fn sort_with_function<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    mut compare_fn: impl FnMut(
        &mut Avm1<'gc>,
        &mut UpdateContext<'_, 'gc, '_>,
        &Value<'gc>,
        &Value<'gc>,
    ) -> Ordering,
    flags: i32,
) -> Result<ReturnValue<'gc>, Error> {
    let length = this.length();
    let mut values: Vec<(usize, Value<'gc>)> = this.array().into_iter().enumerate().collect();
    let array_proto = avm.prototypes.array;

    let descending = (flags & DESCENDING) != 0;
    let unique_sort = (flags & UNIQUE_SORT) != 0;
    let return_indexed_array = (flags & RETURN_INDEXED_ARRAY) != 0;

    let mut is_unique = true;
    values.sort_unstable_by(|a, b| {
        let mut ret = compare_fn(avm, context, &a.1, &b.1);
        if descending {
            ret = ret.reverse();
        }
        if ret == Ordering::Equal {
            is_unique = false;
        }
        ret
    });

    if unique_sort && !is_unique {
        // Check for uniqueness. Return 0 if there is a duplicated value.
        if !is_unique {
            return Ok(0.into());
        }
    }

    if return_indexed_array {
        // Array.RETURNINDEXEDARRAY returns an array containing the sorted indices, and does not modify
        // the original array.
        let array = ScriptObject::array(context.gc_context, Some(array_proto));
        array.set_length(context.gc_context, length);
        for (i, value) in values.into_iter().enumerate() {
            array.set_array_element(i, Value::Number(value.0 as f64), context.gc_context);
        }
        Ok(array.into())
    } else {
        // Standard sort modifies the original array, and returns it.
        // AS2 reference incorrectly states this returns nothing, but it returns the original array, sorted.
        for (i, value) in values.into_iter().enumerate() {
            this.set_array_element(i, value.1, context.gc_context);
        }
        Ok(this.into())
    }
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
    object.force_set_function(
        "sort",
        sort,
        gc_context,
        Attribute::DontEnum,
        Some(fn_proto),
    );
    object.force_set_function(
        "sortOn",
        sort_on,
        gc_context,
        Attribute::DontEnum,
        Some(fn_proto),
    );

    array.into()
}

fn sort_compare_string<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    a: &Value<'gc>,
    b: &Value<'gc>,
) -> Ordering {
    let a_str = a.clone().coerce_to_string(avm, context);
    let b_str = b.clone().coerce_to_string(avm, context);
    // TODO: Handle errors.
    if let (Ok(a_str), Ok(b_str)) = (a_str, b_str) {
        a_str.cmp(&b_str)
    } else {
        DEFAULT_ORDERING
    }
}

fn sort_compare_string_ignore_case<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    a: &Value<'gc>,
    b: &Value<'gc>,
) -> Ordering {
    let a_str = a.clone().coerce_to_string(avm, context);
    let b_str = b.clone().coerce_to_string(avm, context);
    // TODO: Handle errors.
    if let (Ok(a_str), Ok(b_str)) = (a_str, b_str) {
        crate::string_utils::swf_string_cmp_ignore_case(&a_str, &b_str)
    } else {
        DEFAULT_ORDERING
    }
}

fn sort_compare_numeric<'gc>(
    mut string_compare_fn: impl FnMut(
        &mut Avm1<'gc>,
        &mut UpdateContext<'_, 'gc, '_>,
        &Value<'gc>,
        &Value<'gc>,
    ) -> Ordering,
) -> impl FnMut(&mut Avm1<'gc>, &mut UpdateContext<'_, 'gc, '_>, &Value<'gc>, &Value<'gc>) -> Ordering
{
    move |avm, context, a, b| {
        if let (Value::Number(a), Value::Number(b)) = (a, b) {
            a.partial_cmp(b).unwrap_or(DEFAULT_ORDERING)
        } else {
            string_compare_fn(avm, context, a, b)
        }
    }
}

fn sort_compare_fields<'a, 'gc: 'a>(
    field_names: Vec<String>,
    mut compare_fns: Vec<CompareFn<'a, 'gc>>,
) -> impl 'a + FnMut(&mut Avm1<'gc>, &mut UpdateContext<'_, 'gc, '_>, &Value<'gc>, &Value<'gc>) -> Ordering
{
    use crate::avm1::value_object::ValueObject;
    move |avm, context, a, b| {
        for (field_name, compare_fn) in field_names.iter().zip(compare_fns.iter_mut()) {
            let a_object = ValueObject::boxed(avm, context, a.clone());
            let b_object = ValueObject::boxed(avm, context, b.clone());
            let a_prop = a_object
                .get(field_name, avm, context)
                .unwrap()
                .resolve(avm, context)
                .unwrap();
            let b_prop = b_object
                .get(field_name, avm, context)
                .unwrap()
                .resolve(avm, context)
                .unwrap();

            let result = compare_fn(avm, context, &a_prop, &b_prop);
            if result != Ordering::Equal {
                return result;
            }
        }
        // Got through all fields; must be equal.
        Ordering::Equal
    }
}

// Returning an impl Trait here doesn't work yet because of https://github.com/rust-lang/rust/issues/65805 (?)
fn sort_compare_custom<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    a: &Value<'gc>,
    b: &Value<'gc>,
    compare_fn: &Value<'gc>,
) -> Ordering {
    // TODO: Handle errors.
    let args = [a.clone(), b.clone()];
    let ret = compare_fn
        .call(avm, context, this, &args)
        .and_then(|v| v.resolve(avm, context))
        .unwrap_or(Value::Undefined);
    match ret {
        Value::Number(n) if n > 0.0 => Ordering::Greater,
        Value::Number(n) if n < 0.0 => Ordering::Less,
        Value::Number(n) if n == 0.0 => Ordering::Equal,
        _ => DEFAULT_ORDERING,
    }
}
