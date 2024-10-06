//! AVM2 values

use crate::avm2::activation::Activation;
use crate::avm2::error;
use crate::avm2::error::type_error;
use crate::avm2::object::{NamespaceObject, Object, PrimitiveObject, TObject};
use crate::avm2::script::TranslationUnit;
use crate::avm2::Error;
use crate::avm2::Multiname;
use crate::avm2::Namespace;
use crate::ecma_conversions::{f64_to_wrapping_i32, f64_to_wrapping_u32};
use crate::string::{AvmAtom, AvmString, WStr};
use gc_arena::Collect;
use num_bigint::BigInt;
use num_traits::{ToPrimitive, Zero};
use std::mem::size_of;
use swf::avm2::types::{DefaultValue as AbcDefaultValue, Index};

use super::class::Class;
use super::e4x::E4XNode;

/// Indicate what kind of primitive coercion would be preferred when coercing
/// objects.
#[derive(Eq, PartialEq)]
pub enum Hint {
    /// Prefer string coercion (e.g. call `toString` preferentially over
    /// `valueOf`)
    String,

    /// Prefer numerical coercion (e.g. call `valueOf` preferentially over
    /// `toString`)
    Number,
}

/// An AVM2 value.
#[derive(Clone, Copy, Collect, Debug)]
#[collect(no_drop)]
pub enum Value<'gc> {
    Undefined,
    Null,
    Bool(bool),
    Number(f64),
    // note: this value should never reach +/- 1<<28; this is currently not enforced (TODO).
    // Ruffle currently won't break if you break that invariant,
    // but some FP compatibility edge cases depend on it, so we should do this at some point.
    Integer(i32),
    String(AvmString<'gc>),
    Object(Object<'gc>),
}

// This type is used very frequently, so make sure it doesn't unexpectedly grow.
#[cfg(target_family = "wasm")]
const _: () = assert!(size_of::<Value<'_>>() == 16);

#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<Value<'_>>() == 24);

impl<'gc> From<AvmString<'gc>> for Value<'gc> {
    fn from(string: AvmString<'gc>) -> Self {
        Value::String(string)
    }
}

impl<'gc> From<AvmAtom<'gc>> for Value<'gc> {
    fn from(atom: AvmAtom<'gc>) -> Self {
        Value::String(atom.into())
    }
}

impl<'gc> From<&'static str> for Value<'gc> {
    fn from(string: &'static str) -> Self {
        Value::String(string.into())
    }
}

impl<'gc> From<bool> for Value<'gc> {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

impl<'gc, T> From<T> for Value<'gc>
where
    Object<'gc>: From<T>,
{
    fn from(value: T) -> Self {
        Value::Object(Object::from(value))
    }
}

impl<'gc> From<f64> for Value<'gc> {
    fn from(value: f64) -> Self {
        Value::Number(value)
    }
}

impl<'gc> From<f32> for Value<'gc> {
    fn from(value: f32) -> Self {
        Value::Number(f64::from(value))
    }
}

impl<'gc> From<u8> for Value<'gc> {
    fn from(value: u8) -> Self {
        Value::Integer(i32::from(value))
    }
}

impl<'gc> From<i8> for Value<'gc> {
    fn from(value: i8) -> Self {
        Value::Integer(i32::from(value))
    }
}

impl<'gc> From<i16> for Value<'gc> {
    fn from(value: i16) -> Self {
        Value::Integer(i32::from(value))
    }
}

impl<'gc> From<u16> for Value<'gc> {
    fn from(value: u16) -> Self {
        Value::Integer(i32::from(value))
    }
}

impl<'gc> From<i32> for Value<'gc> {
    fn from(value: i32) -> Self {
        if value >= (1 << 28) || value < -(1 << 28) {
            Value::Number(value as f64)
        } else {
            Value::Integer(value)
        }
    }
}

impl<'gc> From<u32> for Value<'gc> {
    fn from(value: u32) -> Self {
        if value >= (1 << 28) {
            Value::Number(value as f64)
        } else {
            Value::Integer(value as i32)
        }
    }
}

impl<'gc> From<usize> for Value<'gc> {
    fn from(value: usize) -> Self {
        Value::Number(value as f64)
    }
}

impl PartialEq for Value<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Undefined, Value::Undefined) => true,
            (Value::Null, Value::Null) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Number(a), Value::Integer(b)) => *a == *b as f64,
            (Value::Integer(a), Value::Number(b)) => *a as f64 == *b,
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Object(a), Value::Object(b)) => Object::ptr_eq(*a, *b),
            _ => false,
        }
    }
}

/// Strips leading whitespace.
fn skip_spaces(s: &mut &WStr) {
    *s = s.trim_start_matches(|c| {
        matches!(
            c,
            0x20 | 0x09 | 0x0d | 0x0a | 0x0c | 0x0b | 0x2000
                ..=0x200b | 0x2028 | 0x2029 | 0x205f | 0x3000
        )
    });
}

/// Consumes an optional sign character.
/// Returns whether a minus sign was consumed.
fn parse_sign(s: &mut &WStr) -> bool {
    if let Some(after_sign) = s.strip_prefix(b'-') {
        *s = after_sign;
        true
    } else if let Some(after_sign) = s.strip_prefix(b'+') {
        *s = after_sign;
        false
    } else {
        false
    }
}

/// Converts a `WStr` to an integer (as an `f64`).
///
/// This function might fail for some invalid inputs, by returning `f64::NAN`.
///
/// `radix` is only valid in the range `2..=36`, plus the special `0` value, which means the
/// radix is inferred from the string; hexadecimal if it starts with a `0x` prefix (case
/// insensitive), or decimal otherwise.
/// `strict` tells whether to fail on trailing garbage, or ignore it.
pub fn string_to_int(mut s: &WStr, mut radix: i32, strict: bool) -> f64 {
    // Allow leading whitespace.
    skip_spaces(&mut s);

    let is_negative = parse_sign(&mut s);

    if radix == 16 || radix == 0 {
        if let Some(after_0x) = s
            .strip_prefix(WStr::from_units(b"0x"))
            .or_else(|| s.strip_prefix(WStr::from_units(b"0X")))
        {
            // Consume hexadecimal prefix.
            s = after_0x;

            // Explicit hexadecimal.
            radix = 16;
        } else if radix == 0 {
            // Default to decimal.
            radix = 10;
        }
    }

    // Fail on invalid radix or blank string.
    if !(2..=36).contains(&radix) || s.is_empty() {
        return f64::NAN;
    }

    // Actual number parsing.
    let mut result = 0.0;
    let start = s;
    s = s.trim_start_matches(|c| {
        match u8::try_from(c)
            .ok()
            .and_then(|c| char::from(c).to_digit(radix as u32))
        {
            Some(digit) => {
                result *= f64::from(radix);
                result += f64::from(digit);
                true
            }
            None => false,
        }
    });

    // Fail if we got no digits.
    // TODO: Compare by reference instead?
    if s.len() == start.len() {
        return f64::NAN;
    }

    if strict {
        // Allow trailing whitespace.
        skip_spaces(&mut s);

        // Fail if we got digits, but we're in strict mode and not at end of string.
        if !s.is_empty() {
            return f64::NAN;
        }
    }

    // Apply sign.
    if is_negative {
        result = -result;
    }

    // We should only return integers and +/-Infinity.
    debug_assert!(result.is_infinite() || result.fract() == 0.0);
    result
}

/// Converts a `WStr` to an `f64`.
///
/// This function might fail for some invalid inputs, by returning `None`.
///
/// `strict` typically tells whether to behave like `Number()` or `parseFloat()`:
/// * `strict == true` fails on trailing garbage, but interprets blank strings (which are empty or consist only of whitespace) as zero.
/// * `strict == false` ignores trailing garbage, but fails on blank strings.
pub fn string_to_f64(mut s: &WStr, swf_version: u8, strict: bool) -> Option<f64> {
    fn is_ascii_digit(c: u16) -> bool {
        u8::try_from(c).map_or(false, |c| c.is_ascii_digit())
    }

    fn to_decimal_digit(c: u16) -> Option<u32> {
        u8::try_from(c)
            .ok()
            .and_then(|c| char::from(c).to_digit(10))
    }

    // Allow leading whitespace.
    skip_spaces(&mut s);

    // Handle blank strings as described above.
    if s.is_empty() {
        return if strict { Some(0.0) } else { None };
    }

    // Parse sign.
    let is_negative = parse_sign(&mut s);
    let after_sign = s;

    // Count digits before decimal point.
    s = s.trim_start_matches(is_ascii_digit);
    let mut total_digits = after_sign.len() - s.len();

    // Count digits after decimal point.
    if let Some(after_dot) = s.strip_prefix(b'.') {
        s = after_dot;
        s = s.trim_start_matches(is_ascii_digit);
        total_digits += after_dot.len() - s.len();
    }

    // Handle exponent.
    let mut exponent: i32 = 0;
    if let Some(after_e) = s.strip_prefix(b"eE".as_ref()) {
        s = after_e;

        // Parse exponent sign.
        let exponent_is_negative = parse_sign(&mut s);

        // Fail if string ends with "e-" with no exponent value specified.
        if exponent_is_negative && s.is_empty() {
            return None;
        }

        // Parse exponent itself.
        s = s.trim_start_matches(|c| match to_decimal_digit(c) {
            Some(digit) => {
                exponent = exponent.wrapping_mul(10);
                exponent = exponent.wrapping_add(digit as i32);
                true
            }
            None => false,
        });

        // Apply exponent sign.
        if exponent_is_negative {
            exponent = exponent.wrapping_neg();
        }
    }

    // Allow trailing whitespace.
    skip_spaces(&mut s);

    // If we got no digits, check for Infinity/-Infinity. Otherwise fail.
    if total_digits == 0 {
        if let Some(after_infinity) = s.strip_prefix(WStr::from_units(b"Infinity")) {
            s = after_infinity;

            // Allow end of string or a whitespace. Otherwise fail.
            if !s.is_empty() {
                skip_spaces(&mut s);
                // TODO: Compare by reference instead?
                if s.len() == after_infinity.len() {
                    return None;
                }
            }

            let result = if is_negative {
                f64::NEG_INFINITY
            } else {
                f64::INFINITY
            };
            return Some(result);
        }
        return None;
    }

    // Fail if we got digits, but we're in strict mode and not at end of string or at a null character.
    if strict && !s.is_empty() && !s.starts_with(b'\0') {
        return None;
    }

    // Bug compatibility: https://bugzilla.mozilla.org/show_bug.cgi?id=513018
    let s = if swf_version >= 11 {
        &after_sign[..after_sign.len() - s.len()]
    } else {
        after_sign
    };

    // Finally, calculate the result.
    let mut result = if total_digits > 15 {
        // With more than 15 digits, avmplus uses integer arithmetic to avoid rounding errors.
        let mut result: BigInt = Zero::zero();
        let mut decimal_digits = -1;
        for c in s {
            if let Some(digit) = to_decimal_digit(c) {
                if decimal_digits != -1 {
                    decimal_digits += 1;
                }

                result *= 10;
                result += i64::from(digit);
            } else if c == b'.' as u16 {
                decimal_digits = 0;
            } else {
                break;
            }
        }

        if decimal_digits > 0 {
            exponent -= decimal_digits;
        }

        if exponent > 0 {
            result *= i64::pow(10, exponent as u32);
        }

        result.to_f64().unwrap_or(f64::NAN)
    } else {
        let mut result = 0.0;
        let mut decimal_digits = -1;
        for c in s {
            if let Some(digit) = to_decimal_digit(c) {
                if decimal_digits != -1 {
                    decimal_digits += 1;
                }

                result *= 10.0;
                result += digit as f64;
            } else if c == b'.' as u16 {
                decimal_digits = 0;
            } else {
                break;
            }
        }

        if decimal_digits > 0 {
            exponent -= decimal_digits;
        }

        if exponent > 0 {
            result *= f64::powi(10.0, exponent);
        }

        result
    };

    if exponent < 0 {
        if exponent < -307 {
            let diff = exponent + 307;
            result /= f64::powi(10.0, -diff);
            exponent = -307;
        }
        result /= f64::powi(10.0, -exponent);
    }

    // Apply sign.
    if is_negative {
        result = -result;
    }

    // We shouldn't return `NaN` after a successful parsing.
    debug_assert!(!result.is_nan());
    Some(result)
}

#[allow(clippy::needless_lifetimes)]
pub fn abc_int<'gc>(
    translation_unit: TranslationUnit<'gc>,
    index: Index<i32>,
) -> Result<i32, Error<'gc>> {
    if index.0 == 0 {
        return Ok(0);
    }

    translation_unit
        .abc()
        .constant_pool
        .ints
        .get(index.0 as usize - 1)
        .cloned()
        .ok_or_else(|| format!("Unknown int constant {}", index.0).into())
}

#[allow(clippy::needless_lifetimes)]
pub fn abc_uint<'gc>(
    translation_unit: TranslationUnit<'gc>,
    index: Index<u32>,
) -> Result<u32, Error<'gc>> {
    if index.0 == 0 {
        return Ok(0);
    }

    translation_unit
        .abc()
        .constant_pool
        .uints
        .get(index.0 as usize - 1)
        .cloned()
        .ok_or_else(|| format!("Unknown uint constant {}", index.0).into())
}

#[allow(clippy::needless_lifetimes)]
pub fn abc_double<'gc>(
    translation_unit: TranslationUnit<'gc>,
    index: Index<f64>,
) -> Result<f64, Error<'gc>> {
    if index.0 == 0 {
        return Ok(f64::NAN);
    }

    translation_unit
        .abc()
        .constant_pool
        .doubles
        .get(index.0 as usize - 1)
        .cloned()
        .ok_or_else(|| format!("Unknown double constant {}", index.0).into())
}

/// Retrieve a default value as an AVM2 `Value`.
pub fn abc_default_value<'gc>(
    translation_unit: TranslationUnit<'gc>,
    default: &AbcDefaultValue,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    match default {
        AbcDefaultValue::Int(i) => abc_int(translation_unit, *i).map(|v| v.into()),
        AbcDefaultValue::Uint(u) => abc_uint(translation_unit, *u).map(|v| v.into()),
        AbcDefaultValue::Double(d) => abc_double(translation_unit, *d).map(|v| v.into()),
        AbcDefaultValue::String(s) => translation_unit
            .pool_string(s.0, activation.strings())
            .map(Into::into),
        AbcDefaultValue::True => Ok(true.into()),
        AbcDefaultValue::False => Ok(false.into()),
        AbcDefaultValue::Null => Ok(Value::Null),
        AbcDefaultValue::Undefined => Ok(Value::Undefined),
        AbcDefaultValue::Namespace(ns)
        | AbcDefaultValue::Package(ns)
        | AbcDefaultValue::PackageInternal(ns)
        | AbcDefaultValue::Protected(ns)
        | AbcDefaultValue::Explicit(ns)
        | AbcDefaultValue::StaticProtected(ns)
        | AbcDefaultValue::Private(ns) => {
            let ns = translation_unit.pool_namespace(activation, *ns)?;
            NamespaceObject::from_namespace(activation, ns).map(Into::into)
        }
    }
}

impl<'gc> Value<'gc> {
    pub fn as_namespace(&self) -> Result<Namespace<'gc>, Error<'gc>> {
        match self {
            Value::Object(ns) => ns
                .as_namespace()
                .ok_or_else(|| "Expected Namespace, found Object".into()),
            _ => Err(format!("Expected Namespace, found {self:?}").into()),
        }
    }

    /// Get the numerical portion of the value, if it exists.
    ///
    /// This function performs no numerical coercion, nor are any methods called.
    /// If the value is not numeric, this function will panic.
    pub fn as_f64(&self) -> f64 {
        match self {
            Value::Number(num) => *num,
            Value::Integer(num) => *num as f64,
            _ => panic!("Expected Number or Integer"),
        }
    }

    /// Like `as_number`, but for `i32`
    pub fn as_i32(&self) -> i32 {
        match self {
            Value::Number(num) => f64_to_wrapping_i32(*num),
            Value::Integer(num) => *num,
            _ => panic!("Expected Number or Integer"),
        }
    }

    /// Like `as_number`, but for `u32`
    pub fn as_u32(&self) -> u32 {
        match self {
            Value::Number(num) => f64_to_wrapping_u32(*num),
            Value::Integer(num) => *num as u32,
            _ => panic!("Expected Number or Integer"),
        }
    }

    /// Yields `true` if the given value is an unboxed primitive value.
    ///
    /// Note: Boxed primitive values are not considered primitive - it is
    /// expected that their `toString`/`valueOf` handlers have already had a
    /// chance to unbox the primitive contained within.
    pub fn is_primitive(&self) -> bool {
        !matches!(self, Value::Object(_))
    }

    /// Coerce the value to a boolean.
    ///
    /// Boolean coercion happens according to the rules specified in the ES4
    /// draft proposals, which appear to be identical to ECMA-262 Edition 3.
    pub fn coerce_to_boolean(&self) -> bool {
        match self {
            Value::Undefined | Value::Null => false,
            Value::Bool(b) => *b,
            Value::Number(f) => !f.is_nan() && *f != 0.0,
            Value::Integer(i) => *i != 0,
            Value::String(s) => !s.is_empty(),
            Value::Object(o) => {
                if let Some(prim) = o.as_primitive() {
                    prim.coerce_to_boolean()
                } else {
                    true
                }
            }
        }
    }

    /// Coerce the value to a primitive.
    ///
    /// This function is guaranteed to return either a primitive value, or a
    /// `TypeError`.
    ///
    /// The `Hint` parameter selects if the coercion prefers `toString` or
    /// `valueOf`. If the preferred function is not available, its opposite
    /// will be called. If neither function successfully generates a primitive,
    /// a `TypeError` will be raised.
    ///
    /// Primitive conversions occur according to ECMA-262 3rd Edition's
    /// ToPrimitive algorithm which appears to match AVM2.
    pub fn coerce_to_primitive(
        &self,
        hint: Option<Hint>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        let hint = hint.unwrap_or_else(|| match self {
            Value::Object(o) => o.default_hint(),
            _ => Hint::Number,
        });

        match self {
            Value::Object(Object::PrimitiveObject(o)) => o.value_of(activation.strings()),
            Value::Object(o) if hint == Hint::String => {
                let object = *o;

                let prim = object.call_public_property("toString", &[], activation)?;
                if prim.is_primitive() {
                    return Ok(prim);
                }

                let prim = object.call_public_property("valueOf", &[], activation)?;
                if prim.is_primitive() {
                    return Ok(prim);
                }

                Err(Error::AvmError(type_error(
                    activation,
                    &format!(
                        "Error #1050: Cannot convert {} to primitive.",
                        o.instance_of_class_name(activation.context.gc_context)
                    ),
                    1050,
                )?))
            }
            Value::Object(o) if hint == Hint::Number => {
                let object = *o;

                let prim = object.call_public_property("valueOf", &[], activation)?;
                if prim.is_primitive() {
                    return Ok(prim);
                }

                let prim = object.call_public_property("toString", &[], activation)?;
                if prim.is_primitive() {
                    return Ok(prim);
                }

                Err(Error::AvmError(type_error(
                    activation,
                    &format!(
                        "Error #1050: Cannot convert {} to primitive.",
                        o.instance_of_class_name(activation.context.gc_context)
                    ),
                    1050,
                )?))
            }
            _ => Ok(*self),
        }
    }

    /// Coerce the value to a floating-point number.
    ///
    /// This function returns the resulting floating-point directly; or a
    /// TypeError if the value is an `Object` that cannot be converted to a
    /// primitive value.
    ///
    /// Numerical conversions occur according to ECMA-262 3rd Edition's
    /// ToNumber algorithm which appears to match AVM2.
    pub fn coerce_to_number(
        &self,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<f64, Error<'gc>> {
        Ok(match self {
            Value::Undefined => f64::NAN,
            Value::Null => 0.0,
            Value::Bool(true) => 1.0,
            Value::Bool(false) => 0.0,
            Value::Number(n) => *n,
            Value::Integer(i) => *i as f64,
            Value::String(s) => {
                let swf_version = activation.context.swf.version();
                string_to_f64(s, swf_version, true).unwrap_or_else(|| string_to_int(s, 0, true))
            }
            Value::Object(_) => self
                .coerce_to_primitive(Some(Hint::Number), activation)?
                .coerce_to_number(activation)?,
        })
    }

    /// Coerce the value to a 32-bit unsigned integer.
    ///
    /// This function returns the resulting u32 directly; or a TypeError if the
    /// value is an `Object` that cannot be converted to a primitive value.
    ///
    /// Numerical conversions occur according to ECMA-262 3rd Edition's
    /// ToUint32 algorithm which appears to match AVM2.
    pub fn coerce_to_u32(&self, activation: &mut Activation<'_, 'gc>) -> Result<u32, Error<'gc>> {
        Ok(match self {
            Value::Integer(i) => *i as u32,
            Value::Number(n) => f64_to_wrapping_u32(*n),
            Value::Bool(b) => *b as u32,
            Value::Undefined | Value::Null => 0,
            Value::String(_) | Value::Object(_) => {
                f64_to_wrapping_u32(self.coerce_to_number(activation)?)
            }
        })
    }

    /// Coerce the value to a 32-bit signed integer.
    ///
    /// This function returns the resulting i32 directly; or a TypeError if the
    /// value is an `Object` that cannot be converted to a primitive value.
    ///
    /// Numerical conversions occur according to ECMA-262 3rd Edition's
    /// ToInt32 algorithm which appears to match AVM2.
    pub fn coerce_to_i32(&self, activation: &mut Activation<'_, 'gc>) -> Result<i32, Error<'gc>> {
        Ok(match self {
            Value::Integer(i) => *i,
            Value::Number(n) => f64_to_wrapping_i32(*n),
            Value::Bool(b) => *b as i32,
            Value::Undefined | Value::Null => 0,
            Value::String(_) | Value::Object(_) => {
                f64_to_wrapping_i32(self.coerce_to_number(activation)?)
            }
        })
    }

    /// Minimum number of digits after which numbers are formatted as
    /// exponential strings.
    const MIN_DIGITS: f64 = -6.0;

    /// Maximum number of digits before numbers are formatted as exponential
    /// strings.
    const MAX_DIGITS: f64 = 21.0;

    /// Maximum number of significant digits renderable within coerced numbers.
    ///
    /// Any precision beyond this point will be discarded and replaced with
    /// zeroes (for whole parts) or not rendered (for decimal parts).
    const MAX_PRECISION: f64 = 15.0;

    /// Coerce the value to a String.
    ///
    /// This function returns the resulting String directly; or a TypeError if
    /// the value is an `Object` that cannot be converted to a primitive value.
    ///
    /// String conversions generally occur according to ECMA-262 3rd Edition's
    /// ToString algorithm. The conversion of numbers to strings appears to be
    /// somewhat underspecified; there are several formatting modes which
    /// change at specific digit count cutoffs, but the spec allows
    /// implementations to limit how much precision is displayed on coerced
    /// numbers, even if that precision would result in rounding the whole part
    /// of the number. (This is confusingly expressed in ECMA-262.)
    ///
    /// TODO: The cutoffs change based on SWF/ABC version. Targeting FP10.3 in
    /// Animate CC 2020 significantly reduces them (towards zero).
    pub fn coerce_to_string<'a>(
        &'a self,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<AvmString<'gc>, Error<'gc>> {
        Ok(match self {
            Value::Undefined => "undefined".into(),
            Value::Null => "null".into(),
            Value::Bool(true) => "true".into(),
            Value::Bool(false) => "false".into(),
            Value::Number(n) if n.is_nan() => "NaN".into(),
            Value::Number(n) if *n == 0.0 => "0".into(),
            Value::Number(n) if *n < 0.0 => AvmString::new_utf8(
                activation.context.gc_context,
                format!("-{}", Value::Number(-n).coerce_to_string(activation)?),
            ),
            Value::Number(n) if n.is_infinite() => "Infinity".into(),
            Value::Number(n) => {
                let digits = n.log10().floor();

                // TODO: This needs to limit precision in the resulting decimal
                // output, not in binary.
                let precision = (n * 10.0_f64.powf(Self::MAX_PRECISION - digits)).floor()
                    / 10.0_f64.powf(Self::MAX_PRECISION - digits);

                if digits < Self::MIN_DIGITS || digits >= Self::MAX_DIGITS {
                    AvmString::new_utf8(
                        activation.context.gc_context,
                        format!(
                            "{}e{}{}",
                            precision / 10.0_f64.powf(digits),
                            if digits < 0.0 { "-" } else { "+" },
                            digits.abs()
                        ),
                    )
                } else {
                    AvmString::new_utf8(activation.context.gc_context, n.to_string())
                }
            }
            Value::Integer(i) => {
                if *i >= 0 && *i < 10 {
                    activation.strings().make_char('0' as u16 + *i as u16)
                } else {
                    AvmString::new_utf8(activation.context.gc_context, i.to_string())
                }
            }
            Value::String(s) => *s,
            Value::Object(_) => self
                .coerce_to_primitive(Some(Hint::String), activation)?
                .coerce_to_string(activation)?,
        })
    }

    /// Coerce the value to a literal value / debug string.
    ///
    /// This matches the string formatting that appears to be in use in "debug"
    /// contexts, where strings themselves also get quoted. Such contexts would
    /// include things like `valueOf`/`toString` on classes that expose their
    /// properties as part of the string.
    pub fn coerce_to_debug_string<'a>(
        &'a self,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<AvmString<'gc>, Error<'gc>> {
        Ok(match self {
            Value::String(s) => {
                AvmString::new_utf8(activation.context.gc_context, format!("\"{s}\""))
            }
            Value::Object(_) => self
                .coerce_to_primitive(Some(Hint::String), activation)?
                .coerce_to_debug_string(activation)?,
            _ => self.coerce_to_string(activation)?,
        })
    }

    /// Coerce the value to an Object.
    ///
    /// TODO: Once `PrimitiveObject` is removed, this method will be able
    /// to be removed too, since all that this will do then is a null/undefined check.
    pub fn coerce_to_object(
        &self,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        match self {
            Value::Undefined => return Err("TypeError: undefined is not an Object".into()),
            Value::Null => return Err("TypeError: null is not an Object".into()),
            Value::Object(o) => return Ok(*o),
            _ => {}
        };

        PrimitiveObject::from_primitive(*self, activation)
    }

    /// Coerce the value to an object, and throw a TypeError relating to object
    /// receivers being null or undefined otherwise.
    /// Note: The error may contain a non-spec info about the way in which it was to be used.
    pub fn coerce_to_object_or_typeerror(
        &self,
        activation: &mut Activation<'_, 'gc>,
        name: Option<&Multiname<'gc>>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        if matches!(self, Value::Null | Value::Undefined) {
            return Err(error::make_null_or_undefined_error(activation, *self, name));
        }
        self.coerce_to_object(activation)
    }

    #[inline(always)]
    pub fn null_check(
        &self,
        activation: &mut Activation<'_, 'gc>,
        name: Option<&Multiname<'gc>>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        if matches!(self, Value::Null | Value::Undefined) {
            return Err(error::make_null_or_undefined_error(activation, *self, name));
        }

        Ok(*self)
    }

    pub fn as_object(&self) -> Option<Object<'gc>> {
        match self {
            Value::Object(o) => Some(*o),
            _ => None,
        }
    }

    /// Unwrap the value's object, if present, and otherwise report an error
    /// if the value is not a callable object (class or function).
    ///
    /// This is also suitable for constructors (with the exception of
    /// `DispatchObject`, which user code shouldn't be able to access).
    ///
    /// The `name` parameter allows inclusion of the name used to look up the
    /// callable in the resulting error, if provided.
    ///
    /// The `receiver` parameter allows inclusion of the type of the receiver
    /// in the error message, if provided.
    pub fn as_callable(
        &self,
        activation: &mut Activation<'_, 'gc>,
        name: Option<&Multiname<'gc>>,
        receiver: Option<Value<'gc>>,
        as_constructor: bool,
    ) -> Result<Object<'gc>, Error<'gc>> {
        match self.as_object() {
            Some(o) if o.as_class_object().is_some() || o.as_executable().is_some() => Ok(o),
            _ => {
                // Undefined function
                let name = if let Some(name) = name {
                    name.to_qualified_name(activation.context.gc_context)
                } else {
                    "value".into()
                };
                let error = if as_constructor {
                    if activation.context.swf.version() < 11 {
                        type_error(
                            activation,
                            &format!("Error #1115: {} is not a constructor.", name),
                            1115,
                        )
                    } else {
                        type_error(
                            activation,
                            "Error #1007: Instantiation attempted on a non-constructor.",
                            1007,
                        )
                    }
                } else if let Some(Value::Object(receiver)) = receiver {
                    type_error(
                        activation,
                        &format!(
                            "Error #1006: {} is not a function of class {}.",
                            name,
                            receiver.instance_of_class_name(activation.context.gc_context)
                        ),
                        1006,
                    )
                } else {
                    type_error(
                        activation,
                        &format!("Error #1006: {} is not a function.", name),
                        1006,
                    )
                };
                Err(Error::AvmError(error?))
            }
        }
    }

    /// Coerce the value to another value by type name.
    ///
    /// This function implements a handful of coercion rules that appear to be
    /// in use when parameters are typechecked. `op_coerce` appears to use
    /// these as well. If `class` is the class corresponding to a primitive
    /// type, then this function will coerce the given value to that type.
    ///
    /// If the type is not coercible to the given type, an error is thrown.
    pub fn coerce_to_type(
        &self,
        activation: &mut Activation<'_, 'gc>,
        class: Class<'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        if class == activation.avm2().class_defs().int {
            return Ok(self.coerce_to_i32(activation)?.into());
        }

        if class == activation.avm2().class_defs().uint {
            return Ok(self.coerce_to_u32(activation)?.into());
        }

        if class == activation.avm2().class_defs().number {
            return Ok(self.coerce_to_number(activation)?.into());
        }

        if class == activation.avm2().class_defs().boolean {
            return Ok(self.coerce_to_boolean().into());
        }

        if matches!(self, Value::Undefined) || matches!(self, Value::Null) {
            if class == activation.avm2().class_defs().void {
                return Ok(Value::Undefined);
            }
            return Ok(Value::Null);
        }

        if class == activation.avm2().class_defs().string {
            return Ok(self.coerce_to_string(activation)?.into());
        }

        if class == activation.avm2().class_defs().object {
            return Ok(*self);
        }

        if let Some(object) = self.as_object() {
            if object.is_of_type(class) {
                return Ok(*self);
            }
        }

        let name = class
            .name()
            .to_qualified_name_err_message(activation.context.gc_context);

        let debug_str = match self {
            Value::Object(obj) if obj.as_primitive().is_none() => {
                // Flash prints the class name (ignoring the toString() impl on the object),
                // followed by something that looks like an address (it varies between executions).
                // For now, we just set the "address" to all zeroes, on the off chance that some
                // application is trying to parse the error message.
                format!(
                    "{}@00000000000",
                    obj.instance_of_class_name(activation.context.gc_context)
                )
            }
            _ => self.coerce_to_debug_string(activation)?.to_string(),
        };

        Err(Error::AvmError(type_error(
            activation,
            &format!("Error #1034: Type Coercion failed: cannot convert {debug_str} to {name}.",),
            1034,
        )?))
    }

    /// Determine if this value is any kind of number.
    pub fn is_number(&self) -> bool {
        match self {
            Value::Number(_) => true,
            Value::Integer(_) => true,
            Value::Object(o) => o.as_primitive().map_or(false, |p| p.is_number()),
            _ => false,
        }
    }

    /// Determine if this value is a number representable as a u32 without loss
    /// of precision.
    #[allow(clippy::float_cmp)]
    pub fn is_u32(&self) -> bool {
        match self {
            Value::Number(n) => *n == (*n as u32 as f64),
            Value::Integer(i) => *i >= 0,
            Value::Object(o) => o.as_primitive().map_or(false, |p| p.is_u32()),
            _ => false,
        }
    }

    /// Determine if this value is a number representable as an i32 without
    /// loss of precision.
    #[allow(clippy::float_cmp)]
    pub fn is_i32(&self) -> bool {
        match self {
            Value::Number(n) => *n == (*n as i32 as f64),
            Value::Integer(_) => true,
            Value::Object(o) => o.as_primitive().map_or(false, |p| p.is_i32()),
            _ => false,
        }
    }

    /// Determine if this value is of a given type.
    ///
    /// This implements a particularly unusual rule: primitive numeric values
    /// considered instances of all numeric types that can represent them. For
    /// example, 5 is simultaneously an instance of `int`, `uint`, and
    /// `Number`.
    pub fn is_of_type(&self, activation: &mut Activation<'_, 'gc>, type_class: Class<'gc>) -> bool {
        if type_class == activation.avm2().class_defs().number {
            return self.is_number();
        }
        if type_class == activation.avm2().class_defs().uint {
            return self.is_u32();
        }
        if type_class == activation.avm2().class_defs().int {
            return self.is_i32();
        }

        if type_class == activation.avm2().class_defs().void {
            return matches!(self, Value::Undefined);
        }

        if type_class == activation.avm2().class_defs().boolean {
            return matches!(self, Value::Bool(_));
        }

        if type_class == activation.avm2().class_defs().string {
            return matches!(self, Value::String(_));
        }

        if type_class == activation.avm2().class_defs().object {
            return !matches!(self, Value::Undefined | Value::Null);
        }

        if let Some(o) = self.as_object() {
            o.is_of_type(type_class)
        } else {
            false
        }
    }

    /// Get the class that this value is of, supporting primitives.
    /// This function will panic if passed Value::Null or Value::Undefined to;
    /// make sure to handle them with `null_check` or similar beforehand.
    pub fn instance_class(&self, activation: &mut Activation<'_, 'gc>) -> Class<'gc> {
        let class_defs = activation.avm2().class_defs();

        match self {
            Value::Bool(_) => class_defs.boolean,
            Value::Number(_) | Value::Integer(_) => class_defs.number,
            Value::String(_) => class_defs.string,
            Value::Object(obj) => obj.instance_class(),

            Value::Undefined | Value::Null => {
                unreachable!("Should not have Undefined or Null in `instance_class`")
            }
        }
    }

    pub fn instance_of_class_name(&self, activation: &mut Activation<'_, 'gc>) -> AvmString<'gc> {
        self.instance_class(activation)
            .name()
            .to_qualified_name(activation.gc())
    }

    /// Implements the strict-equality `===` check for AVM2.
    pub fn strict_eq(&self, other: &Value<'gc>) -> bool {
        if self == other {
            true
        } else {
            // TODO - this should apply to (Array/Vector).indexOf, and possibility more places as well
            if let Some(xml1) = self.as_object().and_then(|obj| obj.as_xml_object()) {
                if let Some(xml2) = other.as_object().and_then(|obj| obj.as_xml_object()) {
                    return E4XNode::ptr_eq(xml1.node(), xml2.node());
                }
            }
            false
        }
    }

    /// Determine if two values are abstractly equal to each other.
    ///
    /// This abstract equality algorithm is intended to match ECMA-262 3rd
    /// edition, section 11.9.3. Inequality is the direct opposite of equality,
    /// and this function always returns a boolean.
    pub fn abstract_eq(
        &self,
        other: &Value<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<bool, Error<'gc>> {
        // ECMA-357 extends the abstract equality algorithm with steps
        // for XML and XMLList types. Because they are objects in Ruffle we
        // have to be a bit more complicated and factor out the code into
        // a separate method.
        if let Value::Object(obj) = self {
            if let Some(xml_list_obj) = obj.as_xml_list_object() {
                return xml_list_obj.equals(other, activation);
            }

            if let Some(xml_obj) = obj.as_xml_object() {
                return xml_obj.abstract_eq(other, activation);
            }

            if let Some(self_qname) = obj.as_qname_object() {
                if let Value::Object(Object::QNameObject(other_qname)) = other {
                    return Ok(self_qname.uri(activation.strings())
                        == other_qname.uri(activation.strings())
                        && self_qname.local_name() == other_qname.local_name());
                }
            }

            if let Some(self_ns) = obj.as_namespace_object() {
                if let Value::Object(Object::NamespaceObject(other_ns)) = other {
                    return Ok(self_ns.namespace().as_uri(activation.strings())
                        == other_ns.namespace().as_uri(activation.strings()));
                }
            }
        }

        if let Value::Object(obj) = other {
            if let Some(xml_list_obj) = obj.as_xml_list_object() {
                return xml_list_obj.equals(self, activation);
            }

            if let Some(xml_obj) = obj.as_xml_object() {
                return xml_obj.abstract_eq(self, activation);
            }
        }

        match (self, other) {
            (Value::Undefined, Value::Undefined) => Ok(true),
            (Value::Null, Value::Null) => Ok(true),
            (Value::Integer(a), Value::Integer(b)) => Ok(a == b),
            (Value::Number(_) | Value::Integer(_), Value::Number(_) | Value::Integer(_)) => {
                let a = self.coerce_to_number(activation)?;
                let b = other.coerce_to_number(activation)?;

                if a.is_nan() || b.is_nan() {
                    return Ok(false);
                }

                if a == b {
                    return Ok(true);
                }

                if a.abs() == 0.0 && b.abs() == 0.0 {
                    return Ok(true);
                }

                Ok(false)
            }
            (Value::String(a), Value::String(b)) => Ok(a == b),
            (Value::Bool(a), Value::Bool(b)) => Ok(a == b),
            (Value::Object(a), Value::Object(b)) => Ok(Object::ptr_eq(*a, *b)),
            (Value::Undefined, Value::Null) => Ok(true),
            (Value::Null, Value::Undefined) => Ok(true),
            (Value::Number(_) | Value::Integer(_), Value::String(_)) => {
                let number_other = Value::from(other.coerce_to_number(activation)?);

                self.abstract_eq(&number_other, activation)
            }
            (Value::String(_), Value::Number(_) | Value::Integer(_)) => {
                let number_self = Value::from(self.coerce_to_number(activation)?);

                number_self.abstract_eq(other, activation)
            }
            (Value::Bool(_), _) => {
                let number_self = Value::from(self.coerce_to_number(activation)?);

                number_self.abstract_eq(other, activation)
            }
            (_, Value::Bool(_)) => {
                let number_other = Value::from(other.coerce_to_number(activation)?);

                self.abstract_eq(&number_other, activation)
            }
            (Value::String(_) | Value::Number(_) | Value::Integer(_), Value::Object(_)) => {
                //TODO: Should this be `Hint::Number`, `Hint::String`, or no-hint?
                let primitive_other = other.coerce_to_primitive(Some(Hint::Number), activation)?;

                self.abstract_eq(&primitive_other, activation)
            }
            (Value::Object(_), Value::String(_) | Value::Number(_) | Value::Integer(_)) => {
                //TODO: Should this be `Hint::Number`, `Hint::String`, or no-hint?
                let primitive_self = self.coerce_to_primitive(Some(Hint::Number), activation)?;

                primitive_self.abstract_eq(other, activation)
            }
            _ => Ok(false),
        }
    }

    /// Determine if this value is abstractly less than the other.
    ///
    /// This abstract relational comparison algorithm is intended to match
    /// ECMA-262 3rd edition, section 11.8.5. It returns `true`, `false`, *or*
    /// `undefined` (to signal NaN), the latter of which we represent as `None`.
    #[allow(clippy::float_cmp)]
    pub fn abstract_lt(
        &self,
        other: &Value<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Option<bool>, Error<'gc>> {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Some(a < b)),
            _ => {
                let prim_self = self.coerce_to_primitive(Some(Hint::Number), activation)?;
                let prim_other = other.coerce_to_primitive(Some(Hint::Number), activation)?;

                if let (Value::String(s), Value::String(o)) = (&prim_self, &prim_other) {
                    return Ok(Some(s.to_string().bytes().lt(o.to_string().bytes())));
                }

                let num_self = prim_self.coerce_to_number(activation)?;
                let num_other = prim_other.coerce_to_number(activation)?;

                if num_self.is_nan() || num_other.is_nan() {
                    return Ok(None);
                }

                Ok(Some(num_self < num_other))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_to_f64() {
        assert_eq!(
            string_to_f64(WStr::from_units(b"350000000000000000000"), 0, true),
            Some(3.5e20)
        );
    }
}
