//! AVM2 values

use crate::avm2::activation::Activation;
use crate::avm2::names::Namespace;
use crate::avm2::names::QName;
use crate::avm2::object::{Object, TObject};
use crate::avm2::script::TranslationUnit;
use crate::avm2::string::AvmString;
use crate::avm2::Error;
use gc_arena::{Collect, MutationContext};
use std::f64::NAN;
use swf::avm2::types::{DefaultValue as AbcDefaultValue, Index};

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
///
/// TODO: AVM2 also needs Scope, Namespace, and XML values.
#[derive(Clone, Collect, Debug)]
#[collect(no_drop)]
pub enum Value<'gc> {
    Undefined,
    Null,
    Bool(bool),
    Number(f64),
    String(AvmString<'gc>),
    Namespace(Namespace<'gc>),
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

impl<'gc> From<Namespace<'gc>> for Value<'gc> {
    fn from(value: Namespace<'gc>) -> Self {
        Value::Namespace(value)
    }
}

impl PartialEq for Value<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Undefined, Value::Undefined) => true,
            (Value::Null, Value::Null) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Object(a), Value::Object(b)) => Object::ptr_eq(*a, *b),
            (Value::Namespace(a), Value::Namespace(b)) => a == b,
            _ => false,
        }
    }
}

pub fn abc_int(translation_unit: TranslationUnit<'_>, index: Index<i32>) -> Result<i32, Error> {
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

pub fn abc_uint(translation_unit: TranslationUnit<'_>, index: Index<u32>) -> Result<u32, Error> {
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

pub fn abc_double(translation_unit: TranslationUnit<'_>, index: Index<f64>) -> Result<f64, Error> {
    if index.0 == 0 {
        return Ok(NAN);
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
    mc: MutationContext<'gc, '_>,
) -> Result<Value<'gc>, Error> {
    match default {
        AbcDefaultValue::Int(i) => abc_int(translation_unit, *i).map(|v| v.into()),
        AbcDefaultValue::Uint(u) => abc_uint(translation_unit, *u).map(|v| v.into()),
        AbcDefaultValue::Double(d) => abc_double(translation_unit, *d).map(|v| v.into()),
        AbcDefaultValue::String(s) => translation_unit.pool_string(s.0, mc).map(|v| v.into()),
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
            Namespace::from_abc_namespace(translation_unit, ns.clone(), mc).map(|v| v.into())
        }
    }
}

impl<'gc> Value<'gc> {
    pub fn as_object(&self) -> Result<Object<'gc>, Error> {
        if let Value::Object(object) = self {
            Ok(*object)
        } else {
            Err(format!("Expected Object, found {:?}", self).into())
        }
    }

    /// Demand a string value, erroring out if one is not found.
    ///
    /// TODO: This should be replaced with `coerce_string` where possible.
    pub fn as_string(&self) -> Result<AvmString<'gc>, Error> {
        match self {
            Value::String(s) => Ok(*s),
            _ => Err(format!("Expected String, found {:?}", self).into()),
        }
    }

    /// Coerce a value into a string.
    pub fn coerce_string(self, mc: MutationContext<'gc, '_>) -> AvmString<'gc> {
        match self {
            Value::String(s) => s,
            Value::Number(n) if n == (f64::INFINITY) => "Infinity".into(),
            Value::Number(n) if n == (f64::INFINITY * -1.0) => "-Infinity".into(),
            Value::Number(n) => AvmString::new(mc, format!("{}", n)),
            Value::Bool(true) => "true".into(),
            Value::Bool(false) => "false".into(),
            _ => "".into(),
        }
    }

    pub fn as_number(&self) -> Result<f64, Error> {
        match self {
            Value::Number(f) => Ok(*f),
            _ => Err(format!("Expected Number, found {:?}", self).into()),
        }
    }

    pub fn as_namespace(&self) -> Result<&Namespace<'gc>, Error> {
        match self {
            Value::Namespace(ns) => Ok(ns),
            _ => Err(format!("Expected Namespace, found {:?}", self).into()),
        }
    }

    /// Yields `true` if the given value is a primitive value.
    ///
    /// Note: Boxed primitive values are not considered primitive - it is
    /// expected that their `toString`/`valueOf` handlers have already had a
    /// chance to unbox the primitive contained within.
    pub fn is_primitive(&self) -> bool {
        match self {
            Value::Object(_) | Value::Namespace(_) => false,
            _ => true,
        }
    }

    /// Coerce the value to a boolean.
    ///
    /// Boolean coercion happens according to the rules specified in the ES4
    /// draft proposals, which appear to be identical to ECMA-262 Edition 3.
    pub fn coerce_to_boolean(&self) -> bool {
        match self {
            Value::Undefined | Value::Null => false,
            Value::Bool(b) => *b,
            Value::Number(f) => !f.is_nan() && f.abs() != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Namespace(_) => true,
            Value::Object(_) => true,
        }
    }

    /// Coerce the value to a primitive.
    ///
    /// This function is guaranteed to return either a primitive value, or a
    /// `TypeError`.
    ///
    /// The `Hint` parameter selects if the coercion prefers `toString` or
    /// `valueOf`. If the preferred function is not available, it's opposite
    /// will be called. If neither function successfully generates a primitive,
    /// a `TypeError` will be raised.
    ///
    /// Primitive conversions occur according to ECMA-262 3rd Edition's
    /// ToPrimitive algorithm which appears to match AVM2.
    pub fn coerce_to_primitive(
        &self,
        hint: Hint,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Value<'gc>, Error> {
        match self {
            Value::Object(o) if hint == Hint::String => {
                let mut prim = self.clone();
                let mut object = *o;

                if let Value::Object(f) =
                    object.get_property(*o, &QName::dynamic_name("toString"), activation)?
                {
                    prim = f.call(Some(*o), &[], activation, None)?;
                }

                if prim.is_primitive() {
                    return Ok(prim);
                }

                if let Value::Object(f) =
                    object.get_property(*o, &QName::dynamic_name("valueOf"), activation)?
                {
                    prim = f.call(Some(*o), &[], activation, None)?;
                }

                if prim.is_primitive() {
                    return Ok(prim);
                }

                Err("TypeError: cannot convert object to string".into())
            }
            Value::Object(o) if hint == Hint::Number => {
                let mut prim = self.clone();
                let mut object = *o;

                if let Value::Object(f) =
                    object.get_property(*o, &QName::dynamic_name("valueOf"), activation)?
                {
                    prim = f.call(Some(*o), &[], activation, None)?;
                }

                if prim.is_primitive() {
                    return Ok(prim);
                }

                if let Value::Object(f) =
                    object.get_property(*o, &QName::dynamic_name("toString"), activation)?
                {
                    prim = f.call(Some(*o), &[], activation, None)?;
                }

                if prim.is_primitive() {
                    return Ok(prim);
                }

                Err("TypeError: cannot convert object to number".into())
            }
            _ => Ok(self.clone()),
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
    pub fn coerce_to_number(&self, activation: &mut Activation<'_, 'gc, '_>) -> Result<f64, Error> {
        Ok(match self {
            Value::Undefined => f64::NAN,
            Value::Null => 0.0,
            Value::Bool(true) => 1.0,
            Value::Bool(false) => 0.0,
            Value::Number(n) => *n,
            Value::String(s) => {
                let strim = s.trim();
                if strim.is_empty() {
                    0.0
                } else if strim.starts_with("0x") || strim.starts_with("0X") {
                    let mut n: f64 = 0.0;
                    for c in strim[2..].chars() {
                        n *= 16.0;
                        n += match c {
                            '0' => 0.0,
                            '1' => 1.0,
                            '2' => 2.0,
                            '3' => 3.0,
                            '4' => 4.0,
                            '5' => 5.0,
                            '6' => 6.0,
                            '7' => 7.0,
                            '8' => 8.0,
                            '9' => 9.0,
                            'a' | 'A' => 10.0,
                            'b' | 'B' => 11.0,
                            'c' | 'C' => 12.0,
                            'd' | 'D' => 13.0,
                            'e' | 'E' => 14.0,
                            'f' | 'F' => 15.0,
                            _ => return Ok(f64::NAN),
                        };
                    }

                    n
                } else {
                    let (sign, digits) = if strim.starts_with('+') {
                        (1.0, &strim[1..])
                    } else if strim.starts_with('-') {
                        (-1.0, &strim[1..])
                    } else {
                        (1.0, strim)
                    };

                    if digits == "Infinity" {
                        return Ok(sign * f64::INFINITY);
                    }

                    //TODO: This is slightly more permissive than ES3 spec, as
                    //Rust documentation claims it will accept "inf" as f64
                    //infinity.
                    sign * digits.parse().unwrap_or(f64::NAN)
                }
            }
            Value::Namespace(ns) => Value::String(AvmString::new(
                activation.context.gc_context,
                ns.as_uri().to_string(),
            ))
            .coerce_to_number(activation)?,
            Value::Object(_) => self
                .coerce_to_primitive(Hint::Number, activation)?
                .coerce_to_number(activation)?,
        })
    }
}
