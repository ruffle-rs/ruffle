//! AVM2 values

use crate::avm2::names::Namespace;
use crate::avm2::object::Object;
use crate::avm2::Error;
use gc_arena::Collect;
use std::f64::NAN;
use swf::avm2::types::{AbcFile, DefaultValue as AbcDefaultValue, Index};

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
    String(String),
    Namespace(Namespace),
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

impl<'gc> From<Namespace> for Value<'gc> {
    fn from(value: Namespace) -> Self {
        Value::Namespace(value)
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
            Value::Namespace(ns) => match other {
                Value::Namespace(other_ns) => ns == other_ns,
                _ => false,
            },
        }
    }
}

pub fn abc_int(file: &AbcFile, index: Index<i32>) -> Result<i32, Error> {
    if index.0 == 0 {
        return Ok(0);
    }

    file.constant_pool
        .ints
        .get(index.0 as usize - 1)
        .cloned()
        .ok_or_else(|| format!("Unknown int constant {}", index.0).into())
}

pub fn abc_uint(file: &AbcFile, index: Index<u32>) -> Result<u32, Error> {
    if index.0 == 0 {
        return Ok(0);
    }

    file.constant_pool
        .uints
        .get(index.0 as usize - 1)
        .cloned()
        .ok_or_else(|| format!("Unknown uint constant {}", index.0).into())
}

pub fn abc_double(file: &AbcFile, index: Index<f64>) -> Result<f64, Error> {
    if index.0 == 0 {
        return Ok(NAN);
    }

    file.constant_pool
        .doubles
        .get(index.0 as usize - 1)
        .cloned()
        .ok_or_else(|| format!("Unknown double constant {}", index.0).into())
}

/// Retrieve a string from an ABC constant pool, yielding `""` if the string
/// is the zero string.
pub fn abc_string(file: &AbcFile, index: Index<String>) -> Result<String, Error> {
    if index.0 == 0 {
        return Ok("".to_string());
    }

    file.constant_pool
        .strings
        .get(index.0 as usize - 1)
        .cloned()
        .ok_or_else(|| format!("Unknown string constant {}", index.0).into())
}

/// Retrieve a string from an ABC constant pool, yielding `None` if the string
/// is the zero string.
///
/// This function still yields `Err` for out-of-bounds string constants, which
/// should cause the runtime to halt. `None` indicates that the zero string is
/// in use, which callers are free to interpret as necessary (although this
/// usually means "any name").
pub fn abc_string_option(file: &AbcFile, index: Index<String>) -> Result<Option<String>, Error> {
    if index.0 == 0 {
        return Ok(None);
    }

    file.constant_pool
        .strings
        .get(index.0 as usize - 1)
        .cloned()
        .map(Some)
        .ok_or_else(|| format!("Unknown string constant {}", index.0).into())
}

/// Retrieve a default value as an AVM2 `Value`.
pub fn abc_default_value<'gc>(
    file: &AbcFile,
    default: &AbcDefaultValue,
) -> Result<Value<'gc>, Error> {
    match default {
        AbcDefaultValue::Int(i) => abc_int(file, *i).map(|v| v.into()),
        AbcDefaultValue::Uint(u) => abc_uint(file, *u).map(|v| v.into()),
        AbcDefaultValue::Double(d) => abc_double(file, *d).map(|v| v.into()),
        AbcDefaultValue::String(s) => abc_string(file, s.clone()).map(|v| v.into()),
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
            Namespace::from_abc_namespace(file, ns.clone()).map(|v| v.into())
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
    pub fn as_string(&self) -> Result<&String, Error> {
        match self {
            Value::String(s) => Ok(s),
            _ => Err(format!("Expected String, found {:?}", self).into()),
        }
    }

    /// Coerce a value into a string.
    pub fn coerce_string(self) -> String {
        match self {
            Value::String(s) => s,
            Value::Bool(true) => "true".to_string(),
            Value::Bool(false) => "false".to_string(),
            _ => "".to_string(),
        }
    }

    pub fn as_number(&self) -> Result<f64, Error> {
        match self {
            Value::Number(f) => Ok(*f),
            _ => Err(format!("Expected Number, found {:?}", self).into()),
        }
    }

    pub fn as_bool(&self) -> Result<bool, Error> {
        if let Value::Bool(b) = self {
            Ok(*b)
        } else {
            Err(format!("Expected Boolean, found {:?}", self).into())
        }
    }

    pub fn as_namespace(&self) -> Result<&Namespace, Error> {
        match self {
            Value::Namespace(ns) => Ok(ns),
            _ => Err(format!("Expected Namespace, found {:?}", self).into()),
        }
    }
}
