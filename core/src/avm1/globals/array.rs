//! Array class

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{ArrayObject, Object, TObject, Value};
use crate::string::AvmString;
use bitflags::bitflags;
use gc_arena::MutationContext;
use std::cmp::Ordering;

bitflags! {
    /// Flags used by `Array.sort` and `Array.sortOn`.
    struct SortFlags: i32 {
        const CASE_INSENSITIVE     = 1 << 0;
        const DESCENDING           = 1 << 1;
        const UNIQUE_SORT          = 1 << 2;
        const RETURN_INDEXED_ARRAY = 1 << 3;
        const NUMERIC              = 1 << 4;
    }
}

// TODO: This won't work accurately in cases like NaN/undefined.
// We need to actually match Flash's sorting algorithm and not use Rust's Vec::sort.
/// Default ordering to return if comparison is invalid.
const DEFAULT_ORDERING: Ordering = Ordering::Equal;

/// Compare function used by `Array.sort` and `Array.sortOn`.
type CompareFn<'a, 'gc> =
    Box<dyn 'a + FnMut(&mut Activation<'_, 'gc, '_>, &Value<'gc>, &Value<'gc>) -> Ordering>;

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "push" => method(push; DONT_ENUM);
    "unshift" => method(unshift; DONT_ENUM);
    "shift" => method(shift; DONT_ENUM);
    "pop" => method(pop; DONT_ENUM);
    "reverse" => method(reverse; DONT_ENUM);
    "join" => method(join; DONT_ENUM);
    "slice" => method(slice; DONT_ENUM);
    "splice" => method(splice; DONT_ENUM);
    "concat" => method(concat; DONT_ENUM);
    "toString" => method(to_string; DONT_ENUM);
    "sort" => method(sort; DONT_ENUM);
    "sortOn" => method(sort_on; DONT_ENUM);
};

const OBJECT_DECLS: &[Declaration] = declare_properties! {
    "CASEINSENSITIVE" => int(SortFlags::CASE_INSENSITIVE.bits(); DONT_ENUM | DONT_DELETE | READ_ONLY);
    "DESCENDING" => int(SortFlags::DESCENDING.bits(); DONT_ENUM | DONT_DELETE | READ_ONLY);
    "UNIQUESORT" => int(SortFlags::UNIQUE_SORT.bits(); DONT_ENUM | DONT_DELETE | READ_ONLY);
    "RETURNINDEXEDARRAY" => int(SortFlags::RETURN_INDEXED_ARRAY.bits(); DONT_ENUM | DONT_DELETE | READ_ONLY);
    "NUMERIC" => int(SortFlags::NUMERIC.bits(); DONT_ENUM | DONT_DELETE | READ_ONLY);
};

pub fn create_array_object<'gc>(
    gc_context: MutationContext<'gc, '_>,
    array_proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let array = FunctionObject::constructor(
        gc_context,
        Executable::Native(constructor),
        Executable::Native(constructor),
        Some(fn_proto),
        array_proto,
    );
    let object = array.as_script_object().unwrap();

    // TODO: These were added in Flash Player 7, but are available even to SWFv6 and lower
    // when run in Flash Player 7. Make these conditional if we add a parameter to control
    // target Flash Player version.
    define_properties_on(OBJECT_DECLS, gc_context, object, fn_proto);
    array
}

/// Implements `Array` constructor and function
pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let [Value::Number(length)] = *args {
        let length = if length.is_finite() && length >= i32::MIN.into() && length <= i32::MAX.into()
        {
            length as i32
        } else {
            i32::MIN
        };
        let array = ArrayObject::empty(activation);
        array.set_length(activation, length)?;
        Ok(array.into())
    } else {
        Ok(ArrayObject::new(
            activation.context.gc_context,
            activation.context.avm1.prototypes().array,
            args.iter().cloned(),
        )
        .into())
    }
}

pub fn push<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let old_length = this.length(activation)?;
    for (i, &arg) in args.iter().enumerate() {
        this.set_element(activation, old_length + i as i32, arg)?;
    }

    let new_length = old_length + args.len() as i32;
    this.set_length(activation, new_length)?;
    Ok(new_length.into())
}

pub fn unshift<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let old_length = this.length(activation)?;
    let new_length = old_length + args.len() as i32;
    for i in 0..old_length {
        let from = old_length - i - 1;
        let to = new_length - i - 1;
        if this.has_element(activation, from) {
            let element = this.get_element(activation, from);
            this.set_element(activation, to, element)?;
        } else {
            this.delete_element(activation, to);
        }
    }

    for (i, &arg) in args.iter().enumerate() {
        this.set_element(activation, i as i32, arg)?;
    }

    if this.as_array_object().is_some() {
        this.set_length(activation, new_length)?;
    }

    Ok(new_length.into())
}

pub fn shift<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let length = this.length(activation)?;
    if length == 0 {
        return Ok(Value::Undefined);
    }

    let first = this.get_element(activation, 0);

    for i in 1..length {
        if this.has_element(activation, i) {
            let element = this.get_element(activation, i);
            this.set_element(activation, i - 1, element)?;
        } else {
            this.delete_element(activation, i - 1);
        }
    }

    this.delete_element(activation, length - 1);

    if this.as_array_object().is_some() {
        this.set_length(activation, length - 1)?;
    }

    Ok(first)
}

pub fn pop<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let length = this.length(activation)?;
    if length == 0 {
        return Ok(Value::Undefined);
    }

    let last = this.get_element(activation, length - 1);

    this.delete_element(activation, length - 1);

    if this.as_array_object().is_some() {
        this.set_length(activation, length - 1)?;
    }

    Ok(last)
}

pub fn reverse<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let length = this.length(activation)?;
    for lower_index in 0..length / 2 {
        let has_lower = this.has_element(activation, lower_index);
        let lower_value = if has_lower {
            this.get_element(activation, lower_index)
        } else {
            Value::Undefined
        };

        let upper_index = length - lower_index - 1;
        let has_upper = this.has_element(activation, upper_index);
        let upper_value = if has_upper {
            this.get_element(activation, upper_index)
        } else {
            Value::Undefined
        };

        match (has_lower, has_upper) {
            (true, true) => {
                this.set_element(activation, lower_index, upper_value)?;
                this.set_element(activation, upper_index, lower_value)?;
            }
            (true, false) => {
                this.delete_element(activation, lower_index);
                this.set_element(activation, upper_index, lower_value)?;
            }
            (false, true) => {
                this.set_element(activation, lower_index, upper_value)?;
                this.delete_element(activation, upper_index);
            }
            (false, false) => {
                this.delete_element(activation, lower_index);
                this.delete_element(activation, upper_index);
            }
        }
    }

    // Some docs incorrectly say reverse returns Void.
    Ok(this.into())
}

pub fn join<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let length = this.length(activation)?;

    let separator = if let Some(v) = args.get(0) {
        v.coerce_to_string(activation)?
    } else {
        ",".into()
    };

    if length <= 0 {
        return Ok("".into());
    }

    let parts: Result<Vec<_>, Error<'gc>> = (0..length)
        .map(|i| {
            let element = this.get_element(activation, i);
            Ok(element.coerce_to_string(activation)?.to_string())
        })
        .collect();

    Ok(AvmString::new(activation.context.gc_context, parts?.join(&separator)).into())
}

/// Handles an index parameter that may be positive (starting from beginning) or negaitve (starting from end).
/// The returned index will be positive and clamped from [0, length].
fn make_index_absolute(index: i32, length: i32) -> i32 {
    if index < 0 {
        (index + length).max(0)
    } else {
        index.min(length)
    }
}

pub fn slice<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let length = this.length(activation)?;

    let start = make_index_absolute(
        args.get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_i32(activation)?,
        length,
    );

    let end = args.get(1).unwrap_or(&Value::Undefined);
    let end = if end == &Value::Undefined {
        length
    } else {
        make_index_absolute(end.coerce_to_i32(activation)?, length)
    };

    Ok(ArrayObject::new(
        activation.context.gc_context,
        activation.context.avm1.prototypes().array,
        (start..end).map(|i| this.get_element(activation, i)),
    )
    .into())
}

pub fn splice<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.is_empty() {
        return Ok(Value::Undefined);
    }

    let length = this.length(activation)?;
    let start = make_index_absolute(args.get(0).unwrap().coerce_to_i32(activation)?, length);
    let delete_count = if let Some(arg) = args.get(1) {
        let delete_count = arg.coerce_to_i32(activation)?;
        if delete_count < 0 {
            return Ok(Value::Undefined);
        }
        delete_count.min(length - start)
    } else {
        length - start
    };

    let result_elements: Vec<_> = (0..delete_count)
        .map(|i| this.get_element(activation, start + i))
        .collect();

    let items = if args.len() > 2 { &args[2..] } else { &[] };
    // TODO: Avoid code duplication.
    if items.len() as i32 > delete_count {
        for i in (start + delete_count..length).rev() {
            if this.has_element(activation, i) {
                let element = this.get_element(activation, i);
                this.set_element(activation, i - delete_count + items.len() as i32, element)?;
            } else {
                this.delete_element(activation, i - delete_count + items.len() as i32);
            }
        }
    } else {
        for i in start + delete_count..length {
            if this.has_element(activation, i) {
                let element = this.get_element(activation, i);
                this.set_element(activation, i - delete_count + items.len() as i32, element)?;
            } else {
                this.delete_element(activation, i - delete_count + items.len() as i32);
            }
        }
    }

    for (i, &item) in items.iter().enumerate() {
        this.set_element(activation, start + i as i32, item)?;
    }
    this.set_length(activation, length - delete_count + items.len() as i32)?;

    Ok(ArrayObject::new(
        activation.context.gc_context,
        activation.context.avm1.prototypes().array,
        result_elements,
    )
    .into())
}

pub fn concat<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let mut elements = vec![];
    for &value in [this.into()].iter().chain(args) {
        let array_object = if let Value::Object(object) = value {
            if object.as_array_object().is_some() {
                Some(object)
            } else {
                None
            }
        } else {
            None
        };

        if let Some(array_object) = array_object {
            let length = array_object.length(activation)?;
            for i in 0..length {
                let element = array_object.get_element(activation, i);
                elements.push(element);
            }
        } else {
            elements.push(value);
        }
    }
    Ok(ArrayObject::new(
        activation.context.gc_context,
        activation.context.avm1.prototypes().array,
        elements,
    )
    .into())
}

pub fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    join(activation, this, &[])
}

fn sort<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // Overloads:
    // 1) a.sort(flags: Number = 0): Sorts with the given flags.
    // 2) a.sort(compare_fn: Object, flags: Number = 0): Sorts using the given compare function and flags.
    use crate::ecma_conversions::f64_to_wrapping_i32;
    let (compare_fn, flags) = match args {
        [Value::Number(_), Value::Number(n), ..] => (None, f64_to_wrapping_i32(*n)),
        [Value::Number(n), ..] => (None, f64_to_wrapping_i32(*n)),
        [Value::Object(compare_fn), Value::Number(n), ..] => {
            (Some(compare_fn), f64_to_wrapping_i32(*n))
        }
        [Value::Object(compare_fn), ..] => (Some(compare_fn), 0),
        [] => (None, 0),
        _ => return Ok(Value::Undefined),
    };
    let flags = SortFlags::from_bits_truncate(flags);

    let string_compare_fn = if flags.contains(SortFlags::CASE_INSENSITIVE) {
        sort_compare_string_ignore_case
    } else {
        sort_compare_string
    };

    let compare_fn: CompareFn<'_, 'gc> = if let Some(f) = compare_fn {
        let this = Value::Undefined.coerce_to_object(activation);
        // this is undefined in the compare function
        Box::new(move |activation, a: &Value<'gc>, b: &Value<'gc>| {
            sort_compare_custom(activation, this, a, b, f)
        })
    } else if flags.contains(SortFlags::NUMERIC) {
        Box::new(sort_compare_numeric(
            flags.contains(SortFlags::CASE_INSENSITIVE),
        ))
    } else {
        Box::new(string_compare_fn)
    };

    sort_with_function(activation, this, compare_fn, flags)
}

fn sort_on<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // a.sortOn(field_name, flags: Number = 0): Sorts with the given flags.
    // a.sortOn(field_names: Array, flags: Number = 0): Sorts with fields in order of precedence with the given flags.
    // a.sortOn(field_names: Array, flags: Array): Sorts with fields in order of precedence with the given flags respectively.
    let fields = match args.get(0) {
        Some(Value::Object(array)) => {
            // Array of field names.
            let length = array.length(activation)?;
            let field_names: Result<Vec<_>, Error<'gc>> = (0..length)
                .map(|i| {
                    array
                        .get_element(activation, i)
                        .coerce_to_string(activation)
                })
                .collect();
            field_names?
        }
        Some(field_name) => {
            // Single field.
            vec![field_name.coerce_to_string(activation)?]
        }
        None => return Ok(Value::Undefined),
    };

    // Bail out if we don't have any fields.
    if fields.is_empty() {
        return Ok(this.into());
    }

    let flags = match args.get(1) {
        Some(Value::Object(array)) => {
            // Array of field names.
            let length = array.length(activation)?;
            if length as usize == fields.len() {
                let flags: Result<Vec<_>, Error<'gc>> = (0..length)
                    .map(|i| {
                        Ok(SortFlags::from_bits_truncate(
                            array.get_element(activation, i).coerce_to_i32(activation)?,
                        ))
                    })
                    .collect();
                flags?
            } else {
                // If the lengths of the flags and fields array do not match, the flags array is ignored.
                std::iter::repeat(SortFlags::empty())
                    .take(fields.len())
                    .collect()
            }
        }
        Some(flags) => {
            // Single field.
            let flags = SortFlags::from_bits_truncate(flags.coerce_to_i32(activation)?);
            std::iter::repeat(flags).take(fields.len()).collect()
        }
        None => std::iter::repeat(SortFlags::empty())
            .take(fields.len())
            .collect(),
    };

    // CASEINSENSITIVE, UNIQUESORT, and RETURNINDEXEDARRAY are taken from the first set of flags in the array.
    let main_flags = flags[0];

    // Generate a list of compare functions to use for each field in the array.
    let field_compare_fns: Vec<CompareFn<'_, 'gc>> = flags
        .into_iter()
        .map(|flags| {
            let string_compare_fn = if flags.contains(SortFlags::CASE_INSENSITIVE) {
                sort_compare_string_ignore_case
            } else {
                sort_compare_string
            };

            if flags.contains(SortFlags::NUMERIC) {
                Box::new(sort_compare_numeric(
                    flags.contains(SortFlags::CASE_INSENSITIVE),
                ))
            } else {
                Box::new(string_compare_fn) as CompareFn<'_, 'gc>
            }
        })
        .collect();

    let compare_fn = sort_compare_fields(fields, field_compare_fns);

    sort_with_function(activation, this, compare_fn, main_flags)
}

fn sort_with_function<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    mut compare_fn: impl FnMut(&mut Activation<'_, 'gc, '_>, &Value<'gc>, &Value<'gc>) -> Ordering,
    flags: SortFlags,
) -> Result<Value<'gc>, Error<'gc>> {
    let length = this.length(activation)?;
    let mut values: Vec<_> = (0..length)
        .map(|i| (i, this.get_element(activation, i)))
        .collect();

    let mut is_unique = true;
    values.sort_unstable_by(|(_, a), (_, b)| {
        let mut ret = compare_fn(activation, a, b);
        if flags.contains(SortFlags::DESCENDING) {
            ret = ret.reverse();
        }
        if ret == Ordering::Equal {
            is_unique = false;
        }
        ret
    });

    if flags.contains(SortFlags::UNIQUE_SORT) && !is_unique {
        // Check for uniqueness. Return 0 if there is a duplicated value.
        return Ok(0.into());
    }

    if flags.contains(SortFlags::RETURN_INDEXED_ARRAY) {
        // Array.RETURNINDEXEDARRAY returns an array containing the sorted indices, and does not modify
        // the original array.
        Ok(ArrayObject::new(
            activation.context.gc_context,
            activation.context.avm1.prototypes().array,
            values.into_iter().map(|(index, _)| index.into()),
        )
        .into())
    } else {
        // Standard sort modifies the original array, and returns it.
        // AS2 reference incorrectly states this returns nothing, but it returns the original array, sorted.
        for (i, (_, value)) in values.into_iter().enumerate() {
            this.set_element(activation, i as i32, value)?;
        }
        this.set_length(activation, length)?;
        Ok(this.into())
    }
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let array = ArrayObject::empty_with_proto(gc_context, Some(proto));
    let object = array.as_script_object().unwrap();
    define_properties_on(PROTO_DECLS, gc_context, object, fn_proto);
    object.into()
}

fn sort_compare_string<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    a: &Value<'gc>,
    b: &Value<'gc>,
) -> Ordering {
    let a_str = a.coerce_to_string(activation);
    let b_str = b.coerce_to_string(activation);
    // TODO: Handle errors.
    if let (Ok(a_str), Ok(b_str)) = (a_str, b_str) {
        a_str.cmp(&b_str)
    } else {
        DEFAULT_ORDERING
    }
}

fn sort_compare_string_ignore_case<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    a: &Value<'gc>,
    b: &Value<'gc>,
) -> Ordering {
    let a_str = a.coerce_to_string(activation);
    let b_str = b.coerce_to_string(activation);
    // TODO: Handle errors.
    if let (Ok(a_str), Ok(b_str)) = (a_str, b_str) {
        crate::string::utils::swf_string_cmp_ignore_case(&a_str, &b_str)
    } else {
        DEFAULT_ORDERING
    }
}

fn sort_compare_numeric<'gc>(
    case_insensitive: bool,
) -> impl FnMut(&mut Activation<'_, 'gc, '_>, &Value<'gc>, &Value<'gc>) -> Ordering {
    move |activation, a, b| {
        if let (Value::Number(a), Value::Number(b)) = (a, b) {
            a.partial_cmp(b).unwrap_or(DEFAULT_ORDERING)
        } else if case_insensitive {
            sort_compare_string_ignore_case(activation, a, b)
        } else {
            sort_compare_string(activation, a, b)
        }
    }
}

fn sort_compare_fields<'a, 'gc: 'a>(
    field_names: Vec<AvmString<'gc>>,
    mut compare_fns: Vec<CompareFn<'a, 'gc>>,
) -> impl 'a + FnMut(&mut Activation<'_, 'gc, '_>, &Value<'gc>, &Value<'gc>) -> Ordering {
    move |activation, a, b| {
        for (field_name, compare_fn) in field_names.iter().zip(compare_fns.iter_mut()) {
            let a_object = a.coerce_to_object(activation);
            let b_object = b.coerce_to_object(activation);
            let a_prop = a_object.get(*field_name, activation).unwrap();
            let b_prop = b_object.get(*field_name, activation).unwrap();

            let result = compare_fn(activation, &a_prop, &b_prop);
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
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    a: &Value<'gc>,
    b: &Value<'gc>,
    compare_fn: &Object<'gc>,
) -> Ordering {
    // TODO: Handle errors.
    let args = [*a, *b];
    let ret = compare_fn
        .call("[Compare]".into(), activation, this, &args)
        .unwrap_or(Value::Undefined);
    match ret {
        Value::Number(n) if n > 0.0 => Ordering::Greater,
        Value::Number(n) if n < 0.0 => Ordering::Less,
        Value::Number(n) if n == 0.0 => Ordering::Equal,
        _ => DEFAULT_ORDERING,
    }
}
