use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::object::value_object::ValueObject;
use crate::avm1::{Object, TObject};
use crate::display_object::TDisplayObject;
use crate::ecma_conversions::{
    f64_to_wrapping_i16, f64_to_wrapping_i32, f64_to_wrapping_u16, f64_to_wrapping_u32,
    f64_to_wrapping_u8,
};
use crate::string::{AvmString, Integer, WStr};
use gc_arena::Collect;
use std::{borrow::Cow, io::Write, num::Wrapping};

#[derive(Debug, Clone, Copy, Collect)]
#[collect(no_drop)]
#[allow(dead_code)]
pub enum Value<'gc> {
    Undefined,
    Null,
    Bool(bool),
    Number(f64),
    String(AvmString<'gc>),
    Object(Object<'gc>),
}

impl<'gc> From<AvmString<'gc>> for Value<'gc> {
    fn from(string: AvmString<'gc>) -> Self {
        Value::String(string)
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

impl<'gc> From<i8> for Value<'gc> {
    fn from(value: i8) -> Self {
        Value::Number(f64::from(value))
    }
}

impl<'gc> From<u8> for Value<'gc> {
    fn from(value: u8) -> Self {
        Value::Number(f64::from(value))
    }
}

impl<'gc> From<i16> for Value<'gc> {
    fn from(value: i16) -> Self {
        Value::Number(f64::from(value))
    }
}

impl<'gc> From<u16> for Value<'gc> {
    fn from(value: u16) -> Self {
        Value::Number(f64::from(value))
    }
}

impl<'gc> From<i32> for Value<'gc> {
    fn from(value: i32) -> Self {
        Value::Number(f64::from(value))
    }
}

impl<'gc> From<u32> for Value<'gc> {
    fn from(value: u32) -> Self {
        Value::Number(f64::from(value))
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
            (Value::Number(a), Value::Number(b)) => (a == b) || (a.is_nan() && b.is_nan()),
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Object(a), Value::Object(b)) => Object::ptr_eq(*a, *b),
            _ => false,
        }
    }
}

impl<'gc> Value<'gc> {
    /// Yields `true` if the given value is a primitive value.
    ///
    /// Note: Boxed primitive values are not considered primitive - it is
    /// expected that their `toString`/`valueOf` handlers have already had a
    /// chance to unbox the primitive contained within.
    pub fn is_primitive(&self) -> bool {
        !matches!(self, Value::Object(_))
    }

    /// ECMA-262 2nd edtion s. 9.3 ToNumber (after calling `to_primitive_num`)
    ///
    /// Flash diverges from spec in a number of ways. These ways are, as far as
    /// we are aware, version-gated:
    ///
    /// * In SWF6 and lower, `undefined` is coerced to `0.0` (like `false`)
    /// rather than `NaN` as required by spec.
    /// * In SWF5 and lower, hexadecimal is unsupported.
    fn primitive_as_number(&self, activation: &mut Activation<'_, 'gc, '_>) -> f64 {
        let v = match self {
            Value::Undefined if activation.swf_version() < 7 => return 0.0,
            Value::Null if activation.swf_version() < 7 => return 0.0,
            Value::Object(_) if activation.swf_version() < 5 => return 0.0,
            Value::String(v) if activation.swf_version() < 5 => {
                use crate::avm1::globals::parse_float_impl;
                let result = parse_float_impl(v.trim_start(), true);
                if result.is_nan() {
                    return 0.0;
                }
                return result;
            }
            Value::Undefined => return f64::NAN,
            Value::Null => return f64::NAN,
            Value::Bool(false) => return 0.0,
            Value::Bool(true) => return 1.0,
            Value::Number(v) => return *v,
            Value::Object(_) => return f64::NAN,
            Value::String(v) => v,
        };

        if v.is_empty() {
            return f64::NAN;
        }

        if activation.swf_version() >= 6 {
            if let Some(v) = v.strip_prefix(WStr::from_units(b"0x")) {
                // Flash allows the '-' sign here.
                return match Wrapping::<i32>::from_wstr_radix(v, 16) {
                    Ok(n) => f64::from(n.0 as i32),
                    Err(_) => f64::NAN,
                };
            } else if v.starts_with(b'0')
                || v.starts_with(WStr::from_units(b"+0"))
                || v.starts_with(WStr::from_units(b"-0"))
            {
                // Flash allows the '-' sign here.
                if let Ok(n) = Wrapping::<i32>::from_wstr_radix(v, 8) {
                    return f64::from(n.0);
                }
            }
        }

        // Rust parses "inf" and "+inf" into Infinity, but Flash doesn't.
        // (as of nightly 4/13, Rust also accepts "infinity")
        // Check if the string starts with 'i' (ignoring any leading +/-).
        if v.strip_prefix(&b"+-"[..])
            .unwrap_or(v)
            .starts_with(&b"iI"[..])
        {
            f64::NAN
        } else {
            v.trim_start_matches(&b"\t\n\r "[..])
                .parse()
                .unwrap_or(f64::NAN)
        }
    }

    /// ECMA-262 2nd edition s. 9.3 ToNumber
    pub fn coerce_to_f64(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<f64, Error<'gc>> {
        Ok(match self {
            Value::Object(_) => self
                .to_primitive_num(activation)?
                .primitive_as_number(activation),
            val => val.primitive_as_number(activation),
        })
    }

    /// ECMA-262 2nd edition s. 9.1 ToPrimitive (hint: Number)
    ///
    /// Flash diverges from spec in a number of ways. These ways are, as far as
    /// we are aware, version-gated:
    ///
    /// * `toString` is never called when `ToPrimitive` is invoked with number
    ///   hint.
    /// * Objects with uncallable `valueOf` implementations are coerced to
    ///   `undefined`. This is not a special-cased behavior: All values are
    ///   callable in `AVM1`. Values that are not callable objects instead
    ///   return `undefined` rather than yielding a runtime error.
    pub fn to_primitive_num(
        self,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        Ok(match self {
            Value::Object(object) if object.as_display_object().is_none() => {
                object.call_method("valueOf".into(), &[], activation)?
            }
            val => val.to_owned(),
        })
    }

    /// Attempts to coerce a value to a primitive type.
    /// This will coerce to Number except for Date objects, which get coerced to String.
    /// If `valueOf` or `toString` do not return a primitive, the original object is returned.
    /// Used by the Add2 action when concatenating to a String, such as `"a" + {}`.
    ///
    /// Loosely based on `ToPrimitive` with no type hint in the ECMAScript spec s.9.1.
    /// Differences from ECMA spec:
    /// * This is only used by the `Add2` action in AVM1, not the `Equals2` action.
    /// * `valueOf`/`toString` are called even if they aren't functions, resulting in `undefined`.
    /// * Date objects will not fall back to `valueOf` if `toString` does not return a primitive.
    /// * Date objects coerce to Number in SWFv5.
    pub fn to_primitive(
        self,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        let result = match self {
            Value::Object(object) => {
                let val = if activation.swf_version() > 5 && object.as_date_object().is_some() {
                    // In SWFv6 and higher, Date objects call `toString`.
                    object.call_method("toString".into(), &[], activation)?
                } else {
                    // Other objects call `valueOf`.
                    object.call_method("valueOf".into(), &[], activation)?
                };

                if val.is_primitive() {
                    val
                } else {
                    // If the above coercion yields an object, the coercion failed, fall back to the object itself.
                    self
                }
            }
            _ => self,
        };
        Ok(result)
    }

    /// ECMA-262 2nd edition s. 11.8.5 Abstract relational comparison algorithm
    pub fn abstract_lt(
        &self,
        other: Value<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        // If either parameter's `valueOf` results in a non-movieclip object, immediately return false.
        // This is the common case for objects because `Object.prototype.valueOf` returns the same object.
        // For example, `{} < {}` is false.
        let prim_self = self.to_primitive_num(activation)?;
        if matches!(prim_self, Value::Object(o) if o.as_display_object().is_none()) {
            return Ok(false.into());
        }
        let prim_other = other.to_primitive_num(activation)?;
        if matches!(prim_other, Value::Object(o) if o.as_display_object().is_none()) {
            return Ok(false.into());
        }

        let result = match (prim_self, prim_other) {
            (Value::String(a), Value::String(b)) => {
                let a = a.to_string();
                let b = b.to_string();
                a.bytes().lt(b.bytes()).into()
            }
            (a, b) => {
                // Coerce to number and compare, with any NaN resulting in undefined.
                let a = a.primitive_as_number(activation);
                let b = b.primitive_as_number(activation);
                a.partial_cmp(&b).map_or(Value::Undefined, |o| {
                    Value::Bool(o == std::cmp::Ordering::Less)
                })
            }
        };
        Ok(result)
    }

    /// ECMA-262 2nd edition s. 11.9.3 Abstract equality comparison algorithm
    pub fn abstract_eq(
        self,
        other: Value<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<bool, Error<'gc>> {
        let (a, b) = if activation.swf_version() > 5 {
            (other, self)
        } else {
            // SWFv5 always calls `valueOf` even in Object-Object comparisons.
            // Object.prototype.valueOf returns `this`, which will do pointer comparison below.
            // In Object-primitive comparisons, `valueOf` will be called a second time below.
            (
                other.to_primitive_num(activation)?,
                self.to_primitive_num(activation)?,
            )
        };
        let result = match (a, b) {
            (Value::Undefined | Value::Null, Value::Undefined | Value::Null) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Object(a), Value::Object(b)) => Object::ptr_eq(a, b),
            (Value::Number(a), Value::Number(b)) => {
                // PLAYER-SPECIFIC: NaN == NaN returns true in Flash Player 7+ AVM1, but returns false in Flash Player 6 and lower.
                // We choose to return true.
                a == b || (a.is_nan() && b.is_nan())
            }

            // Bool-to-value-comparison: Coerce bool to 0/1 and compare.
            (Value::Bool(bool), val) | (val, Value::Bool(bool)) => {
                val.abstract_eq(Value::Number(bool as i64 as f64), activation)?
            }

            // Number-to-value comparison: Coerce value to f64 and compare.
            // Note that "NaN" == NaN returns false.
            (Value::Number(num), string @ Value::String(_))
            | (string @ Value::String(_), Value::Number(num)) => {
                num == string.primitive_as_number(activation)
            }

            // Object-to-value comparison: Call `obj.valueOf` and compare.
            (obj @ Value::Object(_), val) | (val, obj @ Value::Object(_)) => {
                let obj_val = obj.to_primitive_num(activation)?;
                obj_val.is_primitive() && val.abstract_eq(obj_val, activation)?
            }

            _ => false,
        };
        Ok(result)
    }

    /// Converts a bool value into the appropriate value for the platform.
    /// This should be used when pushing a bool onto the stack.
    /// This handles SWFv4 pushing a Number, 0 or 1.
    pub fn from_bool(value: bool, swf_version: u8) -> Value<'gc> {
        // SWF version 4 did not have true bools and will push bools as 0 or 1.
        // e.g. SWF19 p. 72:
        // "If the numbers are equal, true is pushed to the stack for SWF 5 and later. For SWF 4, 1 is pushed to the stack."
        if swf_version >= 5 {
            value.into()
        } else {
            (value as i32).into()
        }
    }

    pub fn coerce_to_u8(&self, activation: &mut Activation<'_, 'gc, '_>) -> Result<u8, Error<'gc>> {
        self.coerce_to_f64(activation).map(f64_to_wrapping_u8)
    }

    /// Coerce a number to an `u16` following the ECMAScript specifications for `ToUInt16`.
    /// The value will be wrapped modulo 2^16.
    /// This will call `valueOf` and do any conversions that are necessary.
    #[allow(dead_code)]
    pub fn coerce_to_u16(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<u16, Error<'gc>> {
        self.coerce_to_f64(activation).map(f64_to_wrapping_u16)
    }

    /// Coerce a number to an `i16` following the wrapping behavior ECMAScript specifications.
    /// The value will be wrapped in the range [-2^15, 2^15).
    /// This will call `valueOf` and do any conversions that are necessary.
    #[allow(dead_code)]
    pub fn coerce_to_i16(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<i16, Error<'gc>> {
        self.coerce_to_f64(activation).map(f64_to_wrapping_i16)
    }

    /// Coerce a number to an `i32` following the ECMAScript specifications for `ToInt32`.
    /// The value will be wrapped modulo 2^32.
    /// This will call `valueOf` and do any conversions that are necessary.
    /// If you are writing AVM code that accepts an integer, you probably want to use this.
    #[allow(dead_code)]
    pub fn coerce_to_i32(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<i32, Error<'gc>> {
        self.coerce_to_f64(activation).map(f64_to_wrapping_i32)
    }

    /// Coerce a number to an `u32` following the ECMAScript specifications for `ToUInt32`.
    /// The value will be wrapped in the range [-2^31, 2^31).
    /// This will call `valueOf` and do any conversions that are necessary.
    #[allow(dead_code)]
    pub fn coerce_to_u32(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<u32, Error<'gc>> {
        self.coerce_to_f64(activation).map(f64_to_wrapping_u32)
    }

    /// Coerce a value to a string.
    pub fn coerce_to_string(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<AvmString<'gc>, Error<'gc>> {
        Ok(match self {
            Value::Undefined if activation.swf_version() < 7 => "".into(),
            Value::Bool(true) if activation.swf_version() < 5 => "1".into(),
            Value::Bool(false) if activation.swf_version() < 5 => "0".into(),
            Value::Object(object) => {
                match object.call_method("toString".into(), &[], activation)? {
                    Value::String(s) => s,
                    _ => {
                        if object.as_executable().is_some() {
                            "[type Function]".into()
                        } else {
                            "[type Object]".into()
                        }
                    }
                }
            }
            Value::Undefined => "undefined".into(),
            Value::Null => "null".into(),
            Value::Bool(true) => "true".into(),
            Value::Bool(false) => "false".into(),
            Value::Number(v) => match f64_to_string(*v) {
                Cow::Borrowed(s) => s.into(),
                Cow::Owned(s) => AvmString::new_utf8(activation.context.gc_context, s),
            },
            Value::String(v) => v.to_owned(),
        })
    }

    pub fn as_bool(&self, swf_version: u8) -> bool {
        match self {
            Value::Bool(v) => *v,
            Value::Number(v) => !v.is_nan() && *v != 0.0,
            Value::String(v) => {
                if swf_version >= 7 {
                    !v.is_empty()
                } else {
                    let num = v.parse().unwrap_or(0.0);
                    num != 0.0
                }
            }
            Value::Object(_) => true,
            _ => false,
        }
    }

    pub fn type_of(&self) -> &'static str {
        match self {
            Value::Undefined => "undefined",
            Value::Null => "null",
            Value::Number(_) => "number",
            Value::Bool(_) => "boolean",
            Value::String(_) => "string",
            Value::Object(object) if object.as_executable().is_some() => "function",
            // MovieClips have a special typeof "movieclip", while others have the default "object".
            Value::Object(object)
                if object
                    .as_display_object()
                    .and_then(|o| o.as_movie_clip())
                    .is_some() =>
            {
                "movieclip"
            }
            Value::Object(_) => "object",
        }
    }

    pub fn coerce_to_object(&self, activation: &mut Activation<'_, 'gc, '_>) -> Object<'gc> {
        ValueObject::boxed(activation, self.to_owned())
    }
}

/// Converts an `f64` to a String with (hopefully) the same output as Flash AVM1.
/// 15 digits are displayed (not including leading 0s in a decimal <1).
/// Exponential notation is used for numbers <= 1e-5 and >= 1e15.
/// Rounding done with ties rounded away from zero.
/// NAN returns `"NaN"`, and infinity returns `"Infinity"`.
#[allow(clippy::approx_constant)]
fn f64_to_string(mut n: f64) -> Cow<'static, str> {
    if n.is_nan() {
        Cow::Borrowed("NaN")
    } else if n == f64::INFINITY {
        Cow::Borrowed("Infinity")
    } else if n == f64::NEG_INFINITY {
        Cow::Borrowed("-Infinity")
    } else if n == 0.0 {
        Cow::Borrowed("0")
    } else if n >= -2147483648.0 && n <= 2147483647.0 && n.fract() == 0.0 {
        // Fast path for integers.
        (n as i32).to_string().into()
    } else {
        // AVM1 f64 -> String (also trying to reproduce bugs).
        // Flash Player's AVM1 does this in a straightforward way, shifting the float into the
        // range of [0.0, 10.0), repeatedly multiplying by 10 to extract digits, and then finally
        // rounding the result. However, the rounding is buggy, when carrying 9.999 -> 10.
        // For example, -9999999999999999.0 results in "-e+16".
        let mut buf: Vec<u8> = Vec::with_capacity(25);
        let is_negative = if n < 0.0 {
            n = -n;
            buf.push(b'-');
            true
        } else {
            false
        };

        // Extract base-2 exponent from double-precision float (11 bits, biased by 1023).
        const MANTISSA_BITS: u64 = 52;
        const EXPONENT_MASK: u64 = 0x7ff;
        const EXPONENT_BIAS: i32 = 1023;
        let mut exp_base2: i32 =
            ((n.to_bits() >> MANTISSA_BITS) & EXPONENT_MASK) as i32 - EXPONENT_BIAS;

        if exp_base2 == -EXPONENT_BIAS {
            // Subnormal float; scale back into normal range and retry getting the exponent.
            const NORMAL_SCALE: f64 = 1.801439850948198e16; // 2^54
            let n = n * NORMAL_SCALE;
            exp_base2 =
                ((n.to_bits() >> MANTISSA_BITS) & EXPONENT_MASK) as i32 - EXPONENT_BIAS - 54;
        }

        // Convert to base-10 exponent.
        const LOG10_2: f64 = 0.301029995663981; // log_10(2) value (less precise than Rust's f64::LOG10_2).
        let mut exp = f64::round(f64::from(exp_base2) * LOG10_2) as i32;

        // Calculate `value * 10^exp` through repeated multiplication or division.
        fn decimal_shift(mut value: f64, mut exp: i32) -> f64 {
            let mut base: f64 = 10.0;
            // The multiply and division branches are intentionally separate to match Flash's behavior.
            if exp > 0 {
                while exp > 0 {
                    if (exp & 1) != 0 {
                        value *= base;
                    }
                    exp >>= 1;
                    base *= base;
                }
            } else {
                exp = -exp;
                while exp > 0 {
                    if (exp & 1) != 0 {
                        value /= base;
                    }
                    exp >>= 1;
                    base *= base;
                }
            };
            value
        }

        // Shift the decimal value so that it's in the range of [0.0, 10.0).
        let mut mantissa: f64 = decimal_shift(n, -exp);

        // The exponent calculation can be off by 1; try the next exponent if so.
        if mantissa as i32 == 0 {
            exp -= 1;
            mantissa = decimal_shift(n, -exp);
        }
        if mantissa as i32 >= 10 {
            exp += 1;
            mantissa = decimal_shift(n, -exp);
        }

        // Generates the next digit character.
        let mut digit = || {
            let digit: i32 = mantissa as i32;
            debug_assert!(digit >= 0 && digit < 10);
            mantissa -= f64::from(digit);
            mantissa *= 10.0;
            b'0' + digit as u8
        };

        const MAX_DECIMAL_PLACES: i32 = 15;
        match exp {
            15.. => {
                // 1.2345e+15
                // This case fails to push an extra 0 to handle the rounding 9.9999 -> 10, which
                // causes the -9999999999999999.0 -> "-e+16" bug later.
                buf.push(digit());
                buf.push(b'.');
                for _ in 0..MAX_DECIMAL_PLACES - 1 {
                    buf.push(digit());
                }
            }
            0..=14 => {
                // 12345.678901234
                buf.push(b'0');
                for _ in 0..=exp {
                    buf.push(digit());
                }
                buf.push(b'.');
                for _ in 0..MAX_DECIMAL_PLACES - exp - 1 {
                    buf.push(digit());
                }
                exp = 0;
            }
            -5..=-1 => {
                // 0.0012345678901234
                buf.extend_from_slice(b"00.");
                buf.resize(buf.len() + (-exp) as usize - 1, b'0');
                for _ in 0..MAX_DECIMAL_PLACES {
                    buf.push(digit());
                }
                exp = 0;
            }
            _ => {
                // 1.345e-15
                buf.push(b'0');
                let n = digit();
                if n != 0 {
                    buf.push(n);
                }
                buf.push(b'.');
                for _ in 0..MAX_DECIMAL_PLACES - 1 {
                    buf.push(digit());
                }
            }
        };

        // Rounding: Peek at the next generated digit and round accordingly.
        // Ties round away from zero.
        if digit() >= b'5' {
            // Add 1 to the right-most digit, carrying if we hit a 9.
            for c in buf.iter_mut().rev() {
                if *c == b'9' {
                    *c = b'0';
                } else if *c >= b'0' {
                    *c += 1;
                    break;
                }
            }
        }

        // Trim any trailing zeros and decimal point.
        while buf.last() == Some(&b'0') {
            buf.pop();
        }
        if buf.last() == Some(&b'.') {
            buf.pop();
        }

        let mut start = 0;
        if exp != 0 {
            // Write exponent (e+###).

            // Lots of band-aids here to attempt to clean up the rounding above.
            // Negative values are not correctly handled in the Flash Player, causing several bugs.
            // PLAYER-SPECIFIC: I think these checks were added in Flash Player 6.
            // Trim leading zeros.
            let pos = buf.iter().position(|&c| c != b'0').unwrap_or(buf.len());
            if pos != 0 {
                buf.copy_within(pos.., 0);
                buf.truncate(buf.len() - pos);
            }
            if buf.is_empty() {
                // Fix up 9.99999 being rounded to 0.00000 when there is no space for the carried 1.
                // If we have no digits, the value was all 0s that were trimmed, so round to 1.
                buf.push(b'1');
                exp += 1;
            } else {
                // Fix up 100e15 to 1e17.
                let pos = buf.iter().rposition(|&c| c != b'0').unwrap_or_default();
                if pos == 0 {
                    exp += buf.len() as i32 - 1;
                    buf.truncate(1);
                }
            }
            let _ = write!(&mut buf, "e{:+}", exp);
        }

        // One final band-aid to eliminate any leading zeros.
        let i = if is_negative { 1 } else { 0 };
        if buf.get(i) == Some(&b'0') && buf.get(i + 1) != Some(&b'.') {
            if i > 0 {
                buf[i] = buf[i - 1];
            }
            start = 1;
        }

        // SAFETY: Buffer is guaranteed to only contain ASCII digits.
        let s = unsafe { std::str::from_utf8_unchecked(&buf[start..]) };
        s.to_string().into()
    }
}

#[cfg(test)]
#[allow(clippy::unreadable_literal)] // Large numeric literals in tests
mod test {
    use crate::avm1::activation::Activation;
    use crate::avm1::error::Error;
    use crate::avm1::function::{Executable, FunctionObject};
    use crate::avm1::globals::create_globals;
    use crate::avm1::object::script_object::ScriptObject;
    use crate::avm1::object::{Object, TObject};
    use crate::avm1::property::Attribute;
    use crate::avm1::test_utils::with_avm;
    use crate::avm1::Value;
    use crate::string::AvmString;

    #[test]
    fn to_primitive_num() {
        with_avm(6, |activation, _this| -> Result<(), Error> {
            let true_value = Value::Bool(true);
            let undefined = Value::Undefined;
            let false_value = Value::Bool(false);
            let null = Value::Null;

            assert_eq!(true_value.to_primitive_num(activation).unwrap(), true_value);
            assert_eq!(undefined.to_primitive_num(activation).unwrap(), undefined);
            assert_eq!(
                false_value.to_primitive_num(activation).unwrap(),
                false_value
            );
            assert_eq!(null.to_primitive_num(activation).unwrap(), null);

            let (protos, global, _) = create_globals(activation.context.gc_context);
            let vglobal = Value::Object(global);

            assert_eq!(vglobal.to_primitive_num(activation).unwrap(), undefined);

            fn value_of_impl<'gc>(
                _activation: &mut Activation<'_, 'gc, '_>,
                _: Object<'gc>,
                _: &[Value<'gc>],
            ) -> Result<Value<'gc>, Error<'gc>> {
                Ok(5.into())
            }

            let valueof = FunctionObject::function(
                activation.context.gc_context,
                Executable::Native(value_of_impl),
                Some(protos.function),
                protos.function,
            );

            let o = ScriptObject::object_cell(activation.context.gc_context, Some(protos.object));
            o.define_value(
                activation.context.gc_context,
                "valueOf",
                valueof.into(),
                Attribute::empty(),
            );

            assert_eq!(
                Value::Object(o).to_primitive_num(activation).unwrap(),
                5.into()
            );

            Ok(())
        });
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn to_number_swf7() {
        with_avm(7, |activation, _this| -> Result<(), Error> {
            let t = Value::Bool(true);
            let u = Value::Undefined;
            let f = Value::Bool(false);
            let n = Value::Null;

            assert_eq!(t.coerce_to_f64(activation).unwrap(), 1.0);
            assert!(u.coerce_to_f64(activation).unwrap().is_nan());
            assert_eq!(f.coerce_to_f64(activation).unwrap(), 0.0);
            assert!(n.coerce_to_f64(activation).unwrap().is_nan());

            let bo = Value::Object(ScriptObject::bare_object(activation.context.gc_context).into());

            assert!(bo.coerce_to_f64(activation).unwrap().is_nan());

            Ok(())
        });
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn to_number_swf6() {
        with_avm(6, |activation, _this| -> Result<(), Error> {
            let t = Value::Bool(true);
            let u = Value::Undefined;
            let f = Value::Bool(false);
            let n = Value::Null;

            assert_eq!(t.coerce_to_f64(activation).unwrap(), 1.0);
            assert_eq!(u.coerce_to_f64(activation).unwrap(), 0.0);
            assert_eq!(f.coerce_to_f64(activation).unwrap(), 0.0);
            assert_eq!(n.coerce_to_f64(activation).unwrap(), 0.0);

            let bo = Value::Object(ScriptObject::bare_object(activation.context.gc_context).into());

            assert_eq!(bo.coerce_to_f64(activation).unwrap(), 0.0);

            Ok(())
        });
    }

    #[test]
    fn abstract_lt_num() {
        with_avm(8, |activation, _this| -> Result<(), Error> {
            let a = Value::Number(1.0);
            let b = Value::Number(2.0);

            assert_eq!(a.abstract_lt(b, activation).unwrap(), Value::Bool(true));

            let nan = Value::Number(f64::NAN);
            assert_eq!(a.abstract_lt(nan, activation).unwrap(), Value::Undefined);

            let inf = Value::Number(f64::INFINITY);
            assert_eq!(a.abstract_lt(inf, activation).unwrap(), Value::Bool(true));

            let neg_inf = Value::Number(f64::NEG_INFINITY);
            assert_eq!(
                a.abstract_lt(neg_inf, activation).unwrap(),
                Value::Bool(false)
            );

            let zero = Value::Number(0.0);
            assert_eq!(a.abstract_lt(zero, activation).unwrap(), Value::Bool(false));

            Ok(())
        });
    }

    #[test]
    fn abstract_gt_num() {
        with_avm(8, |activation, _this| -> Result<(), Error> {
            let a = Value::Number(1.0);
            let b = Value::Number(2.0);

            assert_eq!(b.abstract_lt(a, activation).unwrap(), Value::Bool(false));

            let nan = Value::Number(f64::NAN);
            assert_eq!(nan.abstract_lt(a, activation).unwrap(), Value::Undefined);

            let inf = Value::Number(f64::INFINITY);
            assert_eq!(inf.abstract_lt(a, activation).unwrap(), Value::Bool(false));

            let neg_inf = Value::Number(f64::NEG_INFINITY);
            assert_eq!(
                neg_inf.abstract_lt(a, activation).unwrap(),
                Value::Bool(true)
            );

            let zero = Value::Number(0.0);
            assert_eq!(zero.abstract_lt(a, activation).unwrap(), Value::Bool(true));

            Ok(())
        });
    }

    #[test]
    fn abstract_lt_str() {
        with_avm(8, |activation, _this| -> Result<(), Error> {
            let a = Value::String(AvmString::new_utf8(activation.context.gc_context, "a"));
            let b = Value::String(AvmString::new_utf8(activation.context.gc_context, "b"));

            assert_eq!(a.abstract_lt(b, activation).unwrap(), Value::Bool(true));

            Ok(())
        })
    }

    #[test]
    fn abstract_gt_str() {
        with_avm(8, |activation, _this| -> Result<(), Error> {
            let a = Value::String(AvmString::new_utf8(activation.context.gc_context, "a"));
            let b = Value::String(AvmString::new_utf8(activation.context.gc_context, "b"));

            assert_eq!(b.abstract_lt(a, activation).unwrap(), Value::Bool(false));

            Ok(())
        })
    }

    #[test]

    fn wrapping_u16() {
        use super::f64_to_wrapping_u16;
        assert_eq!(f64_to_wrapping_u16(0.0), 0);
        assert_eq!(f64_to_wrapping_u16(1.0), 1);
        assert_eq!(f64_to_wrapping_u16(-1.0), 65535);
        assert_eq!(f64_to_wrapping_u16(123.1), 123);
        assert_eq!(f64_to_wrapping_u16(66535.9), 999);
        assert_eq!(f64_to_wrapping_u16(-9980.7), 55556);
        assert_eq!(f64_to_wrapping_u16(-196608.0), 0);
        assert_eq!(f64_to_wrapping_u16(f64::NAN), 0);
        assert_eq!(f64_to_wrapping_u16(f64::INFINITY), 0);
        assert_eq!(f64_to_wrapping_u16(f64::NEG_INFINITY), 0);
    }

    #[test]

    fn wrapping_i16() {
        use super::f64_to_wrapping_i16;
        assert_eq!(f64_to_wrapping_i16(0.0), 0);
        assert_eq!(f64_to_wrapping_i16(1.0), 1);
        assert_eq!(f64_to_wrapping_i16(-1.0), -1);
        assert_eq!(f64_to_wrapping_i16(123.1), 123);
        assert_eq!(f64_to_wrapping_i16(32768.9), -32768);
        assert_eq!(f64_to_wrapping_i16(-32769.9), 32767);
        assert_eq!(f64_to_wrapping_i16(-33268.1), 32268);
        assert_eq!(f64_to_wrapping_i16(-196608.0), 0);
        assert_eq!(f64_to_wrapping_i16(f64::NAN), 0);
        assert_eq!(f64_to_wrapping_i16(f64::INFINITY), 0);
        assert_eq!(f64_to_wrapping_i16(f64::NEG_INFINITY), 0);
    }

    #[test]
    fn wrapping_u32() {
        use super::f64_to_wrapping_u32;
        assert_eq!(f64_to_wrapping_u32(0.0), 0);
        assert_eq!(f64_to_wrapping_u32(1.0), 1);
        assert_eq!(f64_to_wrapping_u32(-1.0), 4294967295);
        assert_eq!(f64_to_wrapping_u32(123.1), 123);
        assert_eq!(f64_to_wrapping_u32(4294968295.9), 999);
        assert_eq!(f64_to_wrapping_u32(-4289411740.3), 5555556);
        assert_eq!(f64_to_wrapping_u32(-12884901888.0), 0);
        assert_eq!(f64_to_wrapping_u32(f64::NAN), 0);
        assert_eq!(f64_to_wrapping_u32(f64::INFINITY), 0);
        assert_eq!(f64_to_wrapping_u32(f64::NEG_INFINITY), 0);
    }

    #[test]
    fn wrapping_i32() {
        use super::f64_to_wrapping_i32;
        assert_eq!(f64_to_wrapping_i32(0.0), 0);
        assert_eq!(f64_to_wrapping_i32(1.0), 1);
        assert_eq!(f64_to_wrapping_i32(-1.0), -1);
        assert_eq!(f64_to_wrapping_i32(123.1), 123);
        assert_eq!(f64_to_wrapping_i32(4294968295.9), 999);
        assert_eq!(f64_to_wrapping_i32(2147484648.3), -2147482648);
        assert_eq!(f64_to_wrapping_i32(-8589934591.2), 1);
        assert_eq!(f64_to_wrapping_i32(4294966896.1), -400);
        assert_eq!(f64_to_wrapping_i32(f64::NAN), 0);
        assert_eq!(f64_to_wrapping_i32(f64::INFINITY), 0);
        assert_eq!(f64_to_wrapping_i32(f64::NEG_INFINITY), 0);
    }

    #[test]
    fn f64_to_string() {
        use super::f64_to_string;
        assert_eq!(f64_to_string(0.0), "0");
        assert_eq!(f64_to_string(-0.0), "0");
        assert_eq!(f64_to_string(1.0), "1");
        assert_eq!(f64_to_string(1.4), "1.4");
        assert_eq!(f64_to_string(-990.123), "-990.123");
        assert_eq!(f64_to_string(f64::NAN), "NaN");
        assert_eq!(f64_to_string(f64::INFINITY), "Infinity");
        assert_eq!(f64_to_string(f64::NEG_INFINITY), "-Infinity");
        assert_eq!(f64_to_string(9.9999e14), "999990000000000");
        assert_eq!(f64_to_string(-9.9999e14), "-999990000000000");
        assert_eq!(f64_to_string(1e15), "1e+15");
        assert_eq!(f64_to_string(-1e15), "-1e+15");
        assert_eq!(f64_to_string(1e-5), "0.00001");
        assert_eq!(f64_to_string(-1e-5), "-0.00001");
        assert_eq!(f64_to_string(0.999e-5), "9.99e-6");
        assert_eq!(f64_to_string(-0.999e-5), "-9.99e-6");
        assert_eq!(f64_to_string(0.19999999999999996), "0.2");
        assert_eq!(f64_to_string(-0.19999999999999996), "-0.2");
        assert_eq!(f64_to_string(100000.12345678912), "100000.123456789");
        assert_eq!(f64_to_string(-100000.12345678912), "-100000.123456789");
        assert_eq!(f64_to_string(0.8000000000000005), "0.800000000000001");
        assert_eq!(f64_to_string(-0.8000000000000005), "-0.800000000000001");
        assert_eq!(f64_to_string(0.8300000000000005), "0.83");
        assert_eq!(f64_to_string(1e-320), "9.99988867182684e-321");
        assert_eq!(f64_to_string(f64::MIN), "-1.79769313486231e+308");
        assert_eq!(f64_to_string(f64::MIN_POSITIVE), "2.2250738585072e-308");
        assert_eq!(f64_to_string(f64::MAX), "1.79769313486231e+308");
        assert_eq!(f64_to_string(5e-324), "4.94065645841247e-324");
        assert_eq!(f64_to_string(9.999999999999999), "10");
        assert_eq!(f64_to_string(-9.999999999999999), "-10");
        assert_eq!(f64_to_string(9999999999999996.0), "1e+16");
        assert_eq!(f64_to_string(-9999999999999996.0), "-e+16"); // wat
        assert_eq!(f64_to_string(0.000009999999999999996), "1e-5");
        assert_eq!(f64_to_string(-0.000009999999999999996), "-10e-6");
        assert_eq!(f64_to_string(0.00009999999999999996), "0.0001");
        assert_eq!(f64_to_string(-0.00009999999999999996), "-0.0001");
    }
}
