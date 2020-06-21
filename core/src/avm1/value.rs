use crate::avm1::error::Error;
use crate::avm1::value_object::ValueObject;
use crate::avm1::{Avm1, Object, TObject, UpdateContext};
use std::borrow::Cow;
use std::f64::NAN;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Value<'gc> {
    Undefined,
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Object(Object<'gc>),
}

impl<'gc> From<String> for Value<'gc> {
    fn from(string: String) -> Self {
        Value::String(string)
    }
}

impl<'gc> From<&str> for Value<'gc> {
    fn from(string: &str) -> Self {
        Value::String(string.to_owned())
    }
}

impl<'gc> From<Cow<'_, str>> for Value<'gc> {
    fn from(string: Cow<str>) -> Self {
        Value::String(string.to_string())
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

unsafe impl<'gc> gc_arena::Collect for Value<'gc> {
    fn trace(&self, cc: gc_arena::CollectionContext) {
        if let Value::Object(object) = self {
            object.trace(cc);
        }
    }
}

impl PartialEq for Value<'_> {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Value::Undefined => match other {
                Value::Undefined => true,
                _ => false,
            },
            Value::Null => match other {
                Value::Null => true,
                _ => false,
            },
            Value::Bool(value) => match other {
                Value::Bool(other_value) => value == other_value,
                _ => false,
            },
            Value::Number(value) => match other {
                Value::Number(other_value) => {
                    (value == other_value) || (value.is_nan() && other_value.is_nan())
                }
                _ => false,
            },
            Value::String(value) => match other {
                Value::String(other_value) => value == other_value,
                _ => false,
            },
            Value::Object(value) => match other {
                Value::Object(other_value) => Object::ptr_eq(*value, *other_value),
                _ => false,
            },
        }
    }
}

impl<'gc> Value<'gc> {
    pub fn into_number_v1(self) -> f64 {
        match self {
            Value::Bool(true) => 1.0,
            Value::Number(v) => v,
            Value::String(v) => v.parse().unwrap_or(0.0),
            _ => 0.0,
        }
    }

    /// ECMA-262 2nd edtion s. 9.3 ToNumber (after calling `to_primitive_num`)
    ///
    /// Flash diverges from spec in a number of ways. These ways are, as far as
    /// we are aware, version-gated:
    ///
    /// * In SWF6 and lower, `undefined` is coerced to `0.0` (like `false`)
    /// rather than `NaN` as required by spec.
    /// * In SWF5 and lower, hexadecimal is unsupported.
    fn primitive_as_number(
        &self,
        avm: &mut Avm1<'gc>,
        _context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> f64 {
        match self {
            Value::Undefined if avm.current_swf_version() < 7 => 0.0,
            Value::Null if avm.current_swf_version() < 7 => 0.0,
            Value::Undefined => NAN,
            Value::Null => NAN,
            Value::Bool(false) => 0.0,
            Value::Bool(true) => 1.0,
            Value::Number(v) => *v,
            Value::String(v) => match v.as_str() {
                v if avm.current_swf_version() >= 6 && v.starts_with("0x") => {
                    let mut n: u32 = 0;
                    for c in v[2..].bytes() {
                        n = n.wrapping_shl(4);
                        n |= match c {
                            b'0' => 0,
                            b'1' => 1,
                            b'2' => 2,
                            b'3' => 3,
                            b'4' => 4,
                            b'5' => 5,
                            b'6' => 6,
                            b'7' => 7,
                            b'8' => 8,
                            b'9' => 9,
                            b'a' | b'A' => 10,
                            b'b' | b'B' => 11,
                            b'c' | b'C' => 12,
                            b'd' | b'D' => 13,
                            b'e' | b'E' => 14,
                            b'f' | b'F' => 15,
                            _ => return NAN,
                        }
                    }
                    f64::from(n as i32)
                }
                v if avm.current_swf_version() >= 6
                    && (v.starts_with('0') || v.starts_with("+0") || v.starts_with("-0"))
                    && v[1..].bytes().all(|c| c >= b'0' && c <= b'7') =>
                {
                    let trimmed = v.trim_start_matches(|c| c == '+' || c == '-');
                    let mut n: u32 = 0;
                    for c in trimmed.bytes() {
                        n = n.wrapping_shl(3);
                        n |= (c - b'0') as u32;
                    }
                    if v.starts_with('-') {
                        n = n.wrapping_neg();
                    }
                    f64::from(n as i32)
                }
                "" => NAN,
                _ => v
                    .trim_start_matches(|c| c == '\t' || c == '\n' || c == '\r' || c == ' ')
                    .parse()
                    .unwrap_or(NAN),
            },
            Value::Object(_) => NAN,
        }
    }

    /// ECMA-262 2nd edition s. 9.3 ToNumber
    pub fn coerce_to_f64(
        &self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<f64, Error> {
        Ok(match self {
            Value::Object(_) => self
                .to_primitive_num(avm, context)?
                .primitive_as_number(avm, context),
            val => val.primitive_as_number(avm, context),
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
        &self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<Value<'gc>, Error> {
        Ok(match self {
            Value::Object(object) => object.call_method("valueOf", &[], avm, context)?,
            val => val.to_owned(),
        })
    }

    /// ECMA-262 2nd edition s. 11.8.5 Abstract relational comparison algorithm
    #[allow(clippy::float_cmp)]
    pub fn abstract_lt(
        &self,
        other: Value<'gc>,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<Value<'gc>, Error> {
        let prim_self = self.to_primitive_num(avm, context)?;
        let prim_other = other.to_primitive_num(avm, context)?;

        if let (Value::String(a), Value::String(b)) = (&prim_self, &prim_other) {
            return Ok(a.to_string().bytes().lt(b.to_string().bytes()).into());
        }

        let num_self = prim_self.primitive_as_number(avm, context);
        let num_other = prim_other.primitive_as_number(avm, context);

        if num_self.is_nan() || num_other.is_nan() {
            return Ok(Value::Undefined);
        }

        if num_self == num_other
            || num_self == 0.0 && num_other == -0.0
            || num_self == -0.0 && num_other == 0.0
            || num_self.is_infinite() && num_self.is_sign_positive()
            || num_other.is_infinite() && num_other.is_sign_negative()
        {
            return Ok(false.into());
        }

        if num_self.is_infinite() && num_self.is_sign_negative()
            || num_other.is_infinite() && num_other.is_sign_positive()
        {
            return Ok(true.into());
        }

        Ok((num_self < num_other).into())
    }

    /// ECMA-262 2nd edition s. 11.9.3 Abstract equality comparison algorithm
    #[allow(clippy::unknown_clippy_lints, clippy::unnested_or_patterns)]
    pub fn abstract_eq(
        &self,
        other: Value<'gc>,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        coerced: bool,
    ) -> Result<Value<'gc>, Error> {
        match (self, &other) {
            (Value::Undefined, Value::Undefined) => Ok(true.into()),
            (Value::Null, Value::Null) => Ok(true.into()),
            (Value::Number(a), Value::Number(b)) => {
                if !coerced && a.is_nan() && b.is_nan() {
                    return Ok(true.into());
                }

                if a == b {
                    return Ok(true.into());
                }

                if *a == 0.0 && *b == -0.0 || *a == -0.0 && *b == 0.0 {
                    return Ok(true.into());
                }

                Ok(false.into())
            }
            (Value::String(a), Value::String(b)) => Ok((a == b).into()),
            (Value::Bool(a), Value::Bool(b)) => Ok((a == b).into()),
            (Value::Object(a), Value::Object(b)) => Ok(Object::ptr_eq(*a, *b).into()),
            (Value::Object(a), Value::Null) | (Value::Object(a), Value::Undefined) => {
                Ok(Object::ptr_eq(*a, avm.global_object_cell()).into())
            }
            (Value::Null, Value::Object(b)) | (Value::Undefined, Value::Object(b)) => {
                Ok(Object::ptr_eq(*b, avm.global_object_cell()).into())
            }
            (Value::Undefined, Value::Null) => Ok(true.into()),
            (Value::Null, Value::Undefined) => Ok(true.into()),
            (Value::Number(_), Value::String(_)) => Ok(self.abstract_eq(
                Value::Number(other.coerce_to_f64(avm, context)?),
                avm,
                context,
                true,
            )?),
            (Value::String(_), Value::Number(_)) => {
                Ok(Value::Number(self.coerce_to_f64(avm, context)?)
                    .abstract_eq(other, avm, context, true)?)
            }
            (Value::Bool(_), _) => Ok(Value::Number(self.coerce_to_f64(avm, context)?)
                .abstract_eq(other, avm, context, true)?),
            (_, Value::Bool(_)) => Ok(self.abstract_eq(
                Value::Number(other.coerce_to_f64(avm, context)?),
                avm,
                context,
                true,
            )?),
            (Value::String(_), Value::Object(_)) => {
                let non_obj_other = other.to_primitive_num(avm, context)?;
                if let Value::Object(_) = non_obj_other {
                    return Ok(false.into());
                }

                Ok(self.abstract_eq(non_obj_other, avm, context, true)?)
            }
            (Value::Number(_), Value::Object(_)) => {
                let non_obj_other = other.to_primitive_num(avm, context)?;
                if let Value::Object(_) = non_obj_other {
                    return Ok(false.into());
                }

                Ok(self.abstract_eq(non_obj_other, avm, context, true)?)
            }
            (Value::Object(_), Value::String(_)) => {
                let non_obj_self = self.to_primitive_num(avm, context)?;
                if let Value::Object(_) = non_obj_self {
                    return Ok(false.into());
                }

                Ok(non_obj_self.abstract_eq(other, avm, context, true)?)
            }
            (Value::Object(_), Value::Number(_)) => {
                let non_obj_self = self.to_primitive_num(avm, context)?;
                if let Value::Object(_) = non_obj_self {
                    return Ok(false.into());
                }

                Ok(non_obj_self.abstract_eq(other, avm, context, true)?)
            }
            _ => Ok(false.into()),
        }
    }

    /// Converts a bool value into the appropriate value for the platform.
    /// This should be used when pushing a bool onto the stack.
    /// This handles SWFv4 pushing a Number, 0 or 1.
    pub fn from_bool(value: bool, swf_version: u8) -> Value<'gc> {
        // SWF version 4 did not have true bools and will push bools as 0 or 1.
        // e.g. SWF19 p. 72:
        // "If the numbers are equal, true is pushed to the stack for SWF 5 and later. For SWF 4, 1 is pushed to the stack."
        if swf_version >= 5 {
            Value::Bool(value)
        } else {
            Value::Number(if value { 1.0 } else { 0.0 })
        }
    }

    /// Coerce a number to an `u16` following the ECMAScript specifications for `ToUInt16`.
    /// The value will be wrapped modulo 2^16.
    /// This will call `valueOf` and do any conversions that are necessary.
    #[allow(dead_code)]
    pub fn coerce_to_u16(
        &self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<u16, Error> {
        self.coerce_to_f64(avm, context).map(f64_to_wrapping_u16)
    }

    /// Coerce a number to an `i16` following the wrapping behavior ECMAScript specifications.
    /// The value will be wrapped in the range [-2^15, 2^15).
    /// This will call `valueOf` and do any conversions that are necessary.
    #[allow(dead_code)]
    pub fn coerce_to_i16(
        &self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<i16, Error> {
        self.coerce_to_f64(avm, context).map(f64_to_wrapping_i16)
    }

    /// Coerce a number to an `i32` following the ECMAScript specifications for `ToInt32`.
    /// The value will be wrapped modulo 2^32.
    /// This will call `valueOf` and do any conversions that are necessary.
    /// If you are writing AVM code that accepts an integer, you probably want to use this.
    #[allow(dead_code)]
    pub fn coerce_to_i32(
        &self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<i32, Error> {
        self.coerce_to_f64(avm, context).map(f64_to_wrapping_i32)
    }

    /// Coerce a number to an `u32` following the ECMAScript specifications for `ToUInt32`.
    /// The value will be wrapped in the range [-2^31, 2^31).
    /// This will call `valueOf` and do any conversions that are necessary.
    #[allow(dead_code)]
    pub fn coerce_to_u32(
        &self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<u32, Error> {
        self.coerce_to_f64(avm, context).map(f64_to_wrapping_u32)
    }

    /// Coerce a value to a string.
    pub fn coerce_to_string<'a>(
        &'a self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<Cow<'a, str>, Error> {
        Ok(match self {
            Value::Object(object) => match object.call_method("toString", &[], avm, context)? {
                Value::String(s) => Cow::Owned(s),
                _ => Cow::Borrowed("[type Object]"),
            },
            Value::Undefined => {
                if avm.current_swf_version() >= 7 {
                    Cow::Borrowed("undefined")
                } else {
                    Cow::Borrowed("")
                }
            }
            Value::Null => Cow::Borrowed("null"),
            Value::Bool(true) => Cow::Borrowed("true"),
            Value::Bool(false) => Cow::Borrowed("false"),
            Value::Number(v) => Cow::Owned(f64_to_string(*v)),
            Value::String(v) => Cow::Borrowed(v),
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

    pub fn type_of(&self) -> Value<'gc> {
        Value::String(
            match self {
                Value::Undefined => "undefined",
                Value::Null => "null",
                Value::Number(_) => "number",
                Value::Bool(_) => "boolean",
                Value::String(_) => "string",
                Value::Object(object) => object.type_of(),
            }
            .to_string(),
        )
    }

    pub fn coerce_to_object(
        &self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Object<'gc> {
        ValueObject::boxed(avm, context, self.to_owned())
    }

    pub fn call(
        &self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
        base_proto: Option<Object<'gc>>,
        args: &[Value<'gc>],
    ) -> Result<Value<'gc>, Error> {
        if let Value::Object(object) = self {
            object.call(avm, context, this, base_proto, args)
        } else {
            Ok(Value::Undefined)
        }
    }
}

/// Converts an `f64` to a String with (hopefully) the same output as Flash.
/// For example, NAN returns `"NaN"`, and infinity returns `"Infinity"`.
pub fn f64_to_string(n: f64) -> String {
    if n.is_nan() {
        "NaN".to_string()
    } else if n == std::f64::INFINITY {
        "Infinity".to_string()
    } else if n == std::f64::NEG_INFINITY {
        "-Infinity".to_string()
    } else if n != 0.0 && (n.abs() >= 1e15 || n.abs() < 1e-5) {
        // Exponential notation.
        // Cheating a bit here; Flash always put a sign in front of the exponent, e.g. 1e+15.
        // Can't do this with rust format params, so shove it in there manually.
        let mut s = format!("{:e}", n);
        if let Some(i) = s.find('e') {
            if s.as_bytes().get(i + 1) != Some(&b'-') {
                s.insert(i + 1, '+');
            }
        }
        s
    } else {
        // Normal number.
        n.to_string()
    }
}

/// Converts an `f64` to an `u16` with ECMAScript `ToUInt16` wrapping behavior.
/// The value will be wrapped modulo 2^16.
pub fn f64_to_wrapping_u16(n: f64) -> u16 {
    if !n.is_finite() {
        0
    } else {
        n.trunc().rem_euclid(65536.0) as u16
    }
}

/// Converts an `f64` to an `i16` with ECMAScript wrapping behavior.
/// The value will be wrapped in the range [-2^15, 2^15).
pub fn f64_to_wrapping_i16(n: f64) -> i16 {
    f64_to_wrapping_u16(n) as i16
}

/// Converts an `f64` to an `u32` with ECMAScript `ToUInt32` wrapping behavior.
/// The value will be wrapped modulo 2^32.
#[allow(clippy::unreadable_literal)]
pub fn f64_to_wrapping_u32(n: f64) -> u32 {
    if !n.is_finite() {
        0
    } else {
        n.trunc().rem_euclid(4294967296.0) as u32
    }
}

/// Converts an `f64` to an `i32` with ECMAScript `ToInt32` wrapping behavior.
/// The value will be wrapped in the range [-2^31, 2^31).
pub fn f64_to_wrapping_i32(n: f64) -> i32 {
    f64_to_wrapping_u32(n) as i32
}

#[cfg(test)]
mod test {
    use crate::avm1::error::Error;
    use crate::avm1::function::{Executable, FunctionObject};
    use crate::avm1::globals::create_globals;
    use crate::avm1::object::{Object, TObject};
    use crate::avm1::return_value::ReturnValue;
    use crate::avm1::script_object::ScriptObject;
    use crate::avm1::test_utils::with_avm;
    use crate::avm1::{Avm1, Value};
    use crate::context::UpdateContext;
    use enumset::EnumSet;
    use std::f64::{INFINITY, NAN, NEG_INFINITY};

    #[test]
    fn to_primitive_num() {
        with_avm(6, |avm, context, _this| {
            let true_value = Value::Bool(true);
            let undefined = Value::Undefined;
            let false_value = Value::Bool(false);
            let null = Value::Null;

            assert_eq!(
                true_value.to_primitive_num(avm, context).unwrap(),
                true_value
            );
            assert_eq!(undefined.to_primitive_num(avm, context).unwrap(), undefined);
            assert_eq!(
                false_value.to_primitive_num(avm, context).unwrap(),
                false_value
            );
            assert_eq!(null.to_primitive_num(avm, context).unwrap(), null);

            let (protos, global, _) = create_globals(context.gc_context);
            let vglobal = Value::Object(global);

            assert_eq!(vglobal.to_primitive_num(avm, context).unwrap(), undefined);

            fn value_of_impl<'gc>(
                _: &mut Avm1<'gc>,
                _: &mut UpdateContext<'_, 'gc, '_>,
                _: Object<'gc>,
                _: &[Value<'gc>],
            ) -> Result<ReturnValue<'gc>, Error> {
                Ok(5.0.into())
            }

            let valueof = FunctionObject::function(
                context.gc_context,
                Executable::Native(value_of_impl),
                Some(protos.function),
                None,
            );

            let o = ScriptObject::object_cell(context.gc_context, Some(protos.object));
            o.define_value(
                context.gc_context,
                "valueOf",
                valueof.into(),
                EnumSet::empty(),
            );

            assert_eq!(
                Value::Object(o).to_primitive_num(avm, context).unwrap(),
                Value::Number(5.0)
            );
        });
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn to_number_swf7() {
        with_avm(7, |avm, context, _this| {
            let t = Value::Bool(true);
            let u = Value::Undefined;
            let f = Value::Bool(false);
            let n = Value::Null;

            assert_eq!(t.coerce_to_f64(avm, context).unwrap(), 1.0);
            assert!(u.coerce_to_f64(avm, context).unwrap().is_nan());
            assert_eq!(f.coerce_to_f64(avm, context).unwrap(), 0.0);
            assert!(n.coerce_to_f64(avm, context).unwrap().is_nan());

            let bo = Value::Object(ScriptObject::bare_object(context.gc_context).into());

            assert!(bo.coerce_to_f64(avm, context).unwrap().is_nan());
        });
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn to_number_swf6() {
        with_avm(6, |avm, context, _this| {
            let t = Value::Bool(true);
            let u = Value::Undefined;
            let f = Value::Bool(false);
            let n = Value::Null;

            assert_eq!(t.coerce_to_f64(avm, context).unwrap(), 1.0);
            assert_eq!(u.coerce_to_f64(avm, context).unwrap(), 0.0);
            assert_eq!(f.coerce_to_f64(avm, context).unwrap(), 0.0);
            assert_eq!(n.coerce_to_f64(avm, context).unwrap(), 0.0);

            let bo = Value::Object(ScriptObject::bare_object(context.gc_context).into());

            assert_eq!(bo.coerce_to_f64(avm, context).unwrap(), 0.0);
        });
    }

    #[test]
    fn abstract_lt_num() {
        with_avm(8, |avm, context, _this| {
            let a = Value::Number(1.0);
            let b = Value::Number(2.0);

            assert_eq!(a.abstract_lt(b, avm, context).unwrap(), Value::Bool(true));

            let nan = Value::Number(NAN);
            assert_eq!(a.abstract_lt(nan, avm, context).unwrap(), Value::Undefined);

            let inf = Value::Number(INFINITY);
            assert_eq!(a.abstract_lt(inf, avm, context).unwrap(), Value::Bool(true));

            let neg_inf = Value::Number(NEG_INFINITY);
            assert_eq!(
                a.abstract_lt(neg_inf, avm, context).unwrap(),
                Value::Bool(false)
            );

            let zero = Value::Number(0.0);
            assert_eq!(
                a.abstract_lt(zero, avm, context).unwrap(),
                Value::Bool(false)
            );
        });
    }

    #[test]
    fn abstract_gt_num() {
        with_avm(8, |avm, context, _this| {
            let a = Value::Number(1.0);
            let b = Value::Number(2.0);

            assert_eq!(
                b.abstract_lt(a.clone(), avm, context).unwrap(),
                Value::Bool(false)
            );

            let nan = Value::Number(NAN);
            assert_eq!(
                nan.abstract_lt(a.clone(), avm, context).unwrap(),
                Value::Undefined
            );

            let inf = Value::Number(INFINITY);
            assert_eq!(
                inf.abstract_lt(a.clone(), avm, context).unwrap(),
                Value::Bool(false)
            );

            let neg_inf = Value::Number(NEG_INFINITY);
            assert_eq!(
                neg_inf.abstract_lt(a.clone(), avm, context).unwrap(),
                Value::Bool(true)
            );

            let zero = Value::Number(0.0);
            assert_eq!(
                zero.abstract_lt(a, avm, context).unwrap(),
                Value::Bool(true)
            );
        });
    }

    #[test]
    fn abstract_lt_str() {
        with_avm(8, |avm, context, _this| {
            let a = Value::String("a".to_owned());
            let b = Value::String("b".to_owned());

            assert_eq!(a.abstract_lt(b, avm, context).unwrap(), Value::Bool(true))
        })
    }

    #[test]
    fn abstract_gt_str() {
        with_avm(8, |avm, context, _this| {
            let a = Value::String("a".to_owned());
            let b = Value::String("b".to_owned());

            assert_eq!(b.abstract_lt(a, avm, context).unwrap(), Value::Bool(false))
        })
    }

    #[test]
    #[allow(clippy::unreadable_literal)]

    fn wrapping_u16() {
        use super::f64_to_wrapping_u16;
        assert_eq!(f64_to_wrapping_u16(0.0), 0);
        assert_eq!(f64_to_wrapping_u16(1.0), 1);
        assert_eq!(f64_to_wrapping_u16(-1.0), 65535);
        assert_eq!(f64_to_wrapping_u16(123.1), 123);
        assert_eq!(f64_to_wrapping_u16(66535.9), 999);
        assert_eq!(f64_to_wrapping_u16(-9980.7), 55556);
        assert_eq!(f64_to_wrapping_u16(-196608.0), 0);
        assert_eq!(f64_to_wrapping_u16(std::f64::NAN), 0);
        assert_eq!(f64_to_wrapping_u16(std::f64::INFINITY), 0);
        assert_eq!(f64_to_wrapping_u16(std::f64::NEG_INFINITY), 0);
    }

    #[test]
    #[allow(clippy::unreadable_literal)]

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
        assert_eq!(f64_to_wrapping_i16(std::f64::NAN), 0);
        assert_eq!(f64_to_wrapping_i16(std::f64::INFINITY), 0);
        assert_eq!(f64_to_wrapping_i16(std::f64::NEG_INFINITY), 0);
    }

    #[test]
    #[allow(clippy::unreadable_literal)]
    fn wrapping_u32() {
        use super::f64_to_wrapping_u32;
        assert_eq!(f64_to_wrapping_u32(0.0), 0);
        assert_eq!(f64_to_wrapping_u32(1.0), 1);
        assert_eq!(f64_to_wrapping_u32(-1.0), 4294967295);
        assert_eq!(f64_to_wrapping_u32(123.1), 123);
        assert_eq!(f64_to_wrapping_u32(4294968295.9), 999);
        assert_eq!(f64_to_wrapping_u32(-4289411740.3), 5555556);
        assert_eq!(f64_to_wrapping_u32(-12884901888.0), 0);
        assert_eq!(f64_to_wrapping_u32(std::f64::NAN), 0);
        assert_eq!(f64_to_wrapping_u32(std::f64::INFINITY), 0);
        assert_eq!(f64_to_wrapping_u32(std::f64::NEG_INFINITY), 0);
    }

    #[test]
    #[allow(clippy::unreadable_literal)]
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
        assert_eq!(f64_to_wrapping_i32(std::f64::NAN), 0);
        assert_eq!(f64_to_wrapping_i32(std::f64::INFINITY), 0);
        assert_eq!(f64_to_wrapping_i32(std::f64::NEG_INFINITY), 0);
    }

    #[test]
    fn f64_to_string() {
        use super::f64_to_string;
        assert_eq!(f64_to_string(0.0), "0");
        assert_eq!(f64_to_string(-0.0), "0");
        assert_eq!(f64_to_string(1.0), "1");
        assert_eq!(f64_to_string(1.4), "1.4");
        assert_eq!(f64_to_string(-990.123), "-990.123");
        assert_eq!(f64_to_string(std::f64::NAN), "NaN");
        assert_eq!(f64_to_string(std::f64::INFINITY), "Infinity");
        assert_eq!(f64_to_string(std::f64::NEG_INFINITY), "-Infinity");
        assert_eq!(f64_to_string(9.9999e14), "999990000000000");
        assert_eq!(f64_to_string(-9.9999e14), "-999990000000000");
        assert_eq!(f64_to_string(1e15), "1e+15");
        assert_eq!(f64_to_string(-1e15), "-1e+15");
        assert_eq!(f64_to_string(1e-5), "0.00001");
        assert_eq!(f64_to_string(-1e-5), "-0.00001");
        assert_eq!(f64_to_string(0.999e-5), "9.99e-6");
        assert_eq!(f64_to_string(-0.999e-5), "-9.99e-6");
    }
}
