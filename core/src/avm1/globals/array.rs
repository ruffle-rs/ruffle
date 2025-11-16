//! Array class

use crate::avm1::activation::Activation;
use crate::avm1::clamp::Clamp;
use crate::avm1::error::Error;
use crate::avm1::parameters::{ParametersExt, UndefinedAs};
use crate::avm1::property_decl::{DeclContext, Declaration, SystemClass};
use crate::avm1::{Attribute, NativeObject, Object, Value};
use crate::ecma_conversions::f64_to_wrapping_i32;
use crate::string::{AvmString, StringContext};
use bitflags::bitflags;
use gc_arena::Mutation;
use ruffle_macros::istr;
use std::cmp::Ordering;

bitflags! {
    /// Options used by `Array.sort` and `Array.sortOn`.
    #[derive(Clone, Copy)]
    struct SortOptions: i32 {
        const CASE_INSENSITIVE     = 1 << 0;
        const DESCENDING           = 1 << 1;
        const UNIQUE_SORT          = 1 << 2;
        const RETURN_INDEXED_ARRAY = 1 << 3;
        const NUMERIC              = 1 << 4;
    }
}

/// Default ordering to return if comparison is invalid.
const DEFAULT_ORDERING: Ordering = Ordering::Equal;

/// Compare function used by `Array.sort` and `Array.sortOn`.
type CompareFn<'a, 'gc> = Box<
    dyn 'a
        + Fn(
            &mut Activation<'_, 'gc>,
            &Value<'gc>,
            &Value<'gc>,
            SortOptions,
        ) -> Result<Ordering, Error<'gc>>,
>;

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "push" => method(push; DONT_ENUM | DONT_DELETE);
    "unshift" => method(unshift; DONT_ENUM | DONT_DELETE);
    "shift" => method(shift; DONT_ENUM | DONT_DELETE);
    "pop" => method(pop; DONT_ENUM | DONT_DELETE);
    "reverse" => method(reverse; DONT_ENUM | DONT_DELETE);
    "join" => method(join; DONT_ENUM | DONT_DELETE);
    "slice" => method(slice; DONT_ENUM | DONT_DELETE);
    "splice" => method(splice; DONT_ENUM | DONT_DELETE);
    "concat" => method(concat; DONT_ENUM | DONT_DELETE);
    "toString" => method(to_string; DONT_ENUM | DONT_DELETE);
    "sort" => method(sort; DONT_ENUM | DONT_DELETE);
    "sortOn" => method(sort_on; DONT_ENUM | DONT_DELETE);
};

const OBJECT_DECLS: &[Declaration] = declare_properties! {
    "CASEINSENSITIVE" => int(SortOptions::CASE_INSENSITIVE.bits());
    "DESCENDING" => int(SortOptions::DESCENDING.bits());
    "UNIQUESORT" => int(SortOptions::UNIQUE_SORT.bits());
    "RETURNINDEXEDARRAY" => int(SortOptions::RETURN_INDEXED_ARRAY.bits());
    "NUMERIC" => int(SortOptions::NUMERIC.bits());
};

pub fn create_class<'gc>(
    context: &mut DeclContext<'_, 'gc>,
    super_proto: Object<'gc>,
) -> SystemClass<'gc> {
    let proto = ArrayBuilder::new_with_proto(context.strings, super_proto).with([]);
    let class = context.native_class_with_proto(constructor, Some(array), proto);
    context.define_properties_on(proto, PROTO_DECLS);

    // TODO: These were added in Flash Player 7, but are available even to SWFv6 and lower
    // when run in Flash Player 7. Make these conditional if we add a parameter to control
    // target Flash Player version.
    context.define_properties_on(class.constr, OBJECT_DECLS);

    class
}

/// Intermediate builder for constructing `ArrayObject`,
/// used to work around borrow-checker issues.
pub struct ArrayBuilder<'gc> {
    mc: &'gc Mutation<'gc>,
    length_prop: AvmString<'gc>,
    proto_prop: AvmString<'gc>,
    proto: Object<'gc>,
}

impl<'gc> ArrayBuilder<'gc> {
    pub fn empty(activation: &Activation<'_, 'gc>) -> Object<'gc> {
        Self::new(activation).with([])
    }

    pub fn new(activation: &Activation<'_, 'gc>) -> Self {
        let proto = activation.prototypes().array;
        Self::new_with_proto(&activation.context.strings, proto)
    }

    pub fn new_with_proto(context: &StringContext<'gc>, proto: Object<'gc>) -> Self {
        Self {
            mc: context.gc(),
            length_prop: istr!(context, "length"),
            proto_prop: istr!(context, "__proto__"),
            proto,
        }
    }

    fn init_with(self, this: Object<'gc>, elements: impl IntoIterator<Item = Value<'gc>>) {
        let mut length: i32 = 0;
        for value in elements.into_iter() {
            let length_str = AvmString::new_utf8(self.mc, length.to_string());
            this.define_value(self.mc, length_str, value, Attribute::empty());
            length += 1;
        }
        this.define_value(
            self.mc,
            self.length_prop,
            length.into(),
            Attribute::DONT_ENUM | Attribute::DONT_DELETE,
        );

        this.set_native(self.mc, NativeObject::Array(()));
    }

    pub fn with(self, elements: impl IntoIterator<Item = Value<'gc>>) -> Object<'gc> {
        let obj = Object::new_without_proto(self.mc);
        obj.define_value(
            self.mc,
            self.proto_prop,
            self.proto.into(),
            Attribute::DONT_ENUM | Attribute::DONT_DELETE,
        );

        self.init_with(obj, elements);
        obj
    }
}

/// Implements `Array` constructor
fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let builder = ArrayBuilder::new(activation);
    if let [Value::Number(length)] = *args {
        builder.init_with(this, []);
        this.set_length(activation, length.clamp_to_i32())?;
    } else {
        builder.init_with(this, args.iter().cloned());
    }

    Ok(this.into())
}

/// Implements `Array` function
pub fn array<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let [Value::Number(length)] = *args {
        let array = ArrayBuilder::empty(activation);
        array.set_length(activation, length.clamp_to_i32())?;
        Ok(array.into())
    } else {
        Ok(ArrayBuilder::new(activation)
            .with(args.iter().cloned())
            .into())
    }
}

pub fn push<'gc>(
    activation: &mut Activation<'_, 'gc>,
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
    activation: &mut Activation<'_, 'gc>,
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

    if let NativeObject::Array(_) = this.native() {
        this.set_length(activation, new_length)?;
    }

    Ok(new_length.into())
}

pub fn shift<'gc>(
    activation: &mut Activation<'_, 'gc>,
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

    if let NativeObject::Array(_) = this.native() {
        this.set_length(activation, length - 1)?;
    }

    Ok(first)
}

pub fn pop<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let length = this.length(activation)?;
    if length == 0 {
        return Ok(Value::Undefined);
    }

    let last = this.get_element(activation, length - 1);

    this.delete_element(activation, length - 1);

    if let NativeObject::Array(_) = this.native() {
        this.set_length(activation, length - 1)?;
    }

    Ok(last)
}

pub fn reverse<'gc>(
    activation: &mut Activation<'_, 'gc>,
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
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let length = this.length(activation)?;

    let separator = args
        .try_get_string(activation, 0, UndefinedAs::Some)?
        .unwrap_or(istr!(","));

    if length <= 0 {
        return Ok(istr!("").into());
    }

    let parts = (0..length)
        .map(|i| {
            let element = this.get_element(activation, i);
            element.coerce_to_string(activation)
        })
        .collect::<Result<Vec<_>, _>>()?;

    let joined = crate::string::join(&parts, &separator);
    Ok(AvmString::new(activation.gc(), joined).into())
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
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let length = this.length(activation)?;

    let start = make_index_absolute(args.get_i32(activation, 0)?, length);

    let index = args.try_get_i32(activation, 1, UndefinedAs::None)?;
    let end = if let Some(index) = index {
        make_index_absolute(index, length)
    } else {
        length
    };

    Ok(ArrayBuilder::new(activation)
        .with((start..end).map(|i| this.get_element(activation, i)))
        .into())
}

pub fn splice<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let Some(start) = args.try_get_i32(activation, 0, UndefinedAs::None)? else {
        return Ok(Value::Undefined);
    };

    let length = this.length(activation)?;
    let start = make_index_absolute(start, length);

    let delete_count = if args.len() > 1 {
        if let Some(delete_count) = args.try_get_i32(activation, 1, UndefinedAs::None)? {
            delete_count.min(length - start)
        } else {
            return Ok(Value::Undefined);
        }
    } else {
        length - start
    };
    if delete_count < 0 {
        return Ok(Value::Undefined);
    }

    let result = ArrayBuilder::new(activation)
        .with((0..delete_count).map(|i| this.get_element(activation, start + i)));

    let items = if args.len() > 2 { &args[2..] } else { &[] };
    if items.len() as i32 > delete_count {
        for i in (start + delete_count..length).rev() {
            splice_internal(activation, this, delete_count, items, i)?;
        }
    } else {
        for i in start + delete_count..length {
            splice_internal(activation, this, delete_count, items, i)?;
        }
    }

    fn splice_internal<'gc>(
        activation: &mut Activation<'_, 'gc>,
        this: Object<'gc>,
        delete_count: i32,
        items: &[Value<'gc>],
        i: i32,
    ) -> Result<Value<'gc>, Error<'gc>> {
        if this.has_element(activation, i) {
            let element = this.get_element(activation, i);
            this.set_element(activation, i - delete_count + items.len() as i32, element)?;
        } else {
            this.delete_element(activation, i - delete_count + items.len() as i32);
        }
        Ok(Value::Undefined)
    }

    for (i, &item) in items.iter().enumerate() {
        this.set_element(activation, start + i as i32, item)?;
    }
    this.set_length(activation, length - delete_count + items.len() as i32)?;

    Ok(result.into())
}

pub fn concat<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this_value = this.into();

    let mut elements = vec![];

    std::iter::once(&this_value)
        .chain(args)
        .enumerate()
        .try_for_each(|(index, value)| -> Result<(), Error<'gc>> {
            if let Value::Object(object) = value {
                match (object.native(), index) {
                    // When Array.prototype.concat is called directly with the first argument being an object,
                    // such as in the avm1/from_shumway/array test, the this value (i.e. index 0 in the iterator) will be NativeObject::None instead of NativeObject::Array
                    (NativeObject::None, 0) | (NativeObject::Array(()), _) => {
                        let length = object.length(activation)?;

                        let object_elements =
                            (0..length).map(|index| object.get_element(activation, index));

                        elements.extend(object_elements);

                        return Ok(());
                    }
                    _ => {}
                }
            }

            elements.push(*value);

            Ok(())
        })?;

    Ok(ArrayBuilder::new(activation).with(elements).into())
}

pub fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    join(activation, this, &[])
}

fn sort<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // Overloads:
    // 1) a.sort(options: Number = 0): Sort with the given options.
    // 2) a.sort(compare_fn: Object, options: Number = 0): Sort using the given compare function and options.

    let (compare_fn, options) = match args {
        [Value::Object(f), Value::Number(n), ..] => (
            Some(f),
            SortOptions::from_bits_truncate(f64_to_wrapping_i32(*n)),
        ),
        [Value::Object(f), ..] => (Some(f), SortOptions::empty()),
        [Value::Number(_), Value::Number(n), ..] | [Value::Number(n), ..] => (
            None,
            SortOptions::from_bits_truncate(f64_to_wrapping_i32(*n)),
        ),
        [_, ..] => return Ok(Value::Undefined),
        [] => (None, SortOptions::empty()),
    };

    let compare_fn = if let Some(compare_fn) = compare_fn {
        sort_compare_custom(compare_fn)
    } else {
        Box::new(sort_compare)
    };
    sort_internal(activation, this, compare_fn, options, false)
}

fn sort_on<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // Overloads:
    // 1) a.sortOn(field_name: String, options: Number = 0): Sort by a field and with the given options.
    // 2) a.sortOn(field_names: Array, options: Array): Sort by fields in order of precedence and with the given options respectively.

    let fields = match args.get(0) {
        Some(Value::Object(field_names_array)) => {
            if !matches!(field_names_array.native(), NativeObject::Array(_)) {
                // Non-Array fields.
                // Fallback to standard sort.
                return sort_internal(
                    activation,
                    this,
                    Box::new(sort_compare),
                    SortOptions::empty(),
                    true,
                );
            }

            // Array of field names.
            let length = field_names_array.length(activation)?;

            // Bail-out if we don't have any fields.
            if length <= 0 {
                return Ok(this.into());
            }

            let fields: Result<Vec<_>, Error<'gc>> = (0..length)
                .map(|i| {
                    field_names_array
                        .get_element(activation, i)
                        .coerce_to_string(activation)
                        .map(|field_name| (field_name, SortOptions::empty()))
                })
                .collect();
            let mut fields = fields?;

            match args.get(1) {
                Some(Value::Object(options_array))
                    if matches!(options_array.native(), NativeObject::Array(_))
                        && options_array.length(activation)? == length =>
                {
                    // Array of options.
                    for (i, (_field_name, options)) in fields.iter_mut().enumerate() {
                        *options = SortOptions::from_bits_truncate(
                            options_array
                                .get_element(activation, i as i32)
                                .coerce_to_i32(activation)?,
                        );
                    }
                }
                Some(options) if options.is_primitive() => {
                    // Single options.
                    let options =
                        SortOptions::from_bits_truncate(options.coerce_to_i32(activation)?);
                    fields.iter_mut().for_each(|(_, o)| *o = options);
                }
                _ => {
                    // Non-Array options or mismatching lengths.
                }
            }

            fields
        }
        Some(field_name) => {
            // Single field.
            let field_name = field_name.coerce_to_string(activation)?;

            let options = match args.get(1) {
                Some(Value::Number(n)) => SortOptions::from_bits_truncate(f64_to_wrapping_i32(*n)),
                _ => SortOptions::empty(),
            };

            vec![(field_name, options)]
        }
        None => return Ok(Value::Undefined),
    };

    let (_, main_options) = fields[0];
    let compare_fn = sort_on_compare(&fields);
    sort_internal(activation, this, compare_fn, main_options, true)
}

/// Compare between two values, with specified sort options.
fn sort_compare<'gc>(
    activation: &mut Activation<'_, 'gc>,
    a: &Value<'gc>,
    b: &Value<'gc>,
    options: SortOptions,
) -> Result<Ordering, Error<'gc>> {
    let result = match [a, b] {
        [Value::Number(a), Value::Number(b)] if options.contains(SortOptions::NUMERIC) => {
            a.partial_cmp(b).unwrap_or(DEFAULT_ORDERING)
        }
        _ => {
            let a = a.coerce_to_string(activation)?;
            let b = b.coerce_to_string(activation)?;
            if options.contains(SortOptions::CASE_INSENSITIVE) {
                a.cmp_ignore_case(&b)
            } else {
                a.cmp(&b)
            }
        }
    };

    if options.contains(SortOptions::DESCENDING) {
        Ok(result.reverse())
    } else {
        Ok(result)
    }
}

/// Create a compare function based on a user-provided custom AS function.
fn sort_compare_custom<'a, 'gc>(compare_fn: &'a Object<'gc>) -> CompareFn<'a, 'gc> {
    Box::new(move |activation, a, b, _options| {
        let this = Value::Undefined;
        let args = [*a, *b];
        let result = compare_fn.call("[Compare]", activation, this, &args)?;
        let result = result.coerce_to_f64(activation)?;
        Ok(result.partial_cmp(&0.0).unwrap_or(DEFAULT_ORDERING))
    })
}

/// Create a compare function based on field names and options.
fn sort_on_compare<'a, 'gc>(fields: &'a [(AvmString<'gc>, SortOptions)]) -> CompareFn<'a, 'gc> {
    Box::new(move |activation, a, b, main_options| {
        if let [Value::Object(a), Value::Object(b)] = [a, b] {
            for (field_name, options) in fields {
                let a_prop = a
                    .get_local_stored(*field_name, activation)
                    .unwrap_or(Value::Undefined);
                let b_prop = b
                    .get_local_stored(*field_name, activation)
                    .unwrap_or(Value::Undefined);

                let result = sort_compare(activation, &a_prop, &b_prop, *options)?;
                if result.is_ne() {
                    return Ok(result);
                }
            }

            // Got through all fields; must be equal.
            Ok(Ordering::Equal)
        } else {
            // Fallback to standard comparison.
            sort_compare(activation, a, b, main_options)
        }
    })
}

/// Common code for both `Array.sort` and `Array.sortOn`.
fn sort_internal<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    compare_fn: CompareFn<'_, 'gc>,
    mut options: SortOptions,
    is_sort_on: bool,
) -> Result<Value<'gc>, Error<'gc>> {
    let length = this.length(activation)?;
    let mut elements: Vec<_> = (0..length)
        .map(|i| (i, this.get_element(activation, i)))
        .collect();

    // Remove DESCENDING for `Array.sort`, as it should be handled by the `.reverse()` below.
    let descending = options.contains(SortOptions::DESCENDING);
    if !is_sort_on {
        options.remove(SortOptions::DESCENDING);
    }

    qsort(activation, &mut elements, &compare_fn, options)?;

    if !is_sort_on && descending {
        elements.reverse();
    }

    if options.contains(SortOptions::UNIQUE_SORT) {
        // Check for uniqueness. Return 0 if there is a duplicated value.
        let compare_fn = if is_sort_on {
            compare_fn
        } else {
            Box::new(sort_compare)
        };
        for pair in elements.windows(2) {
            let (_, a) = pair[0];
            let (_, b) = pair[1];
            if compare_fn(activation, &a, &b, options)?.is_eq() {
                return Ok(0.into());
            }
        }
    }

    if options.contains(SortOptions::RETURN_INDEXED_ARRAY) {
        // Array.RETURNINDEXEDARRAY returns an array containing the sorted indices, and does not modify
        // the original array.
        Ok(ArrayBuilder::new(activation)
            .with(elements.into_iter().map(|(index, _)| index.into()))
            .into())
    } else {
        // Standard sort modifies the original array, and returns it.
        // AS2 reference incorrectly states this returns nothing, but it returns the original array, sorted.
        for (i, (_, value)) in elements.into_iter().enumerate() {
            this.set_element(activation, i as i32, value)?;
        }
        Ok(this.into())
    }
}

/// Sort elements using the quicksort algorithm, mimicking Flash's behavior.
fn qsort<'gc>(
    activation: &mut Activation<'_, 'gc>,
    elements: &mut [(i32, Value<'gc>)],
    compare_fn: &CompareFn<'_, 'gc>,
    options: SortOptions,
) -> Result<(), Error<'gc>> {
    if elements.len() < 2 {
        // One or no elements - nothing to do.
        return Ok(());
    }

    // Stack for storing inclusive subarray boundaries (start and end).
    let mut stack: Vec<(usize, usize)> = Vec::new();

    stack.push((0, elements.len() - 1));

    while let Some((low, high)) = stack.pop() {
        if low >= high {
            continue;
        }

        // Flash always chooses the leftmost element as the pivot.
        let pivot = elements[low].1;

        let mut left = low + 1;
        let mut right = high;

        loop {
            // Find an element greater than the pivot from the left.
            while left < right {
                let (_, item) = &elements[left];
                if compare_fn(activation, &pivot, item, options)?.is_le() {
                    break;
                }
                left += 1;
            }

            // Find an element lower than the pivot from the right.
            while right > low {
                let (_, item) = &elements[right];
                if compare_fn(activation, &pivot, item, options)?.is_gt() {
                    break;
                }
                right -= 1;
            }

            // When left and right cross, then no element greater than
            // the pivot comes before an element lower than the pivot.
            if left >= right {
                break;
            }

            // Otherwise, swap left and right, and keep going.
            elements.swap(left, right);
        }

        // Move the pivot element to its position between the partitions.
        elements.swap(low, right);

        // Push subarrays onto the stack for further sorting.
        stack.push((right + 1, high));
        if right > 0 {
            stack.push((low, right - 1));
        }
    }

    Ok(())
}
