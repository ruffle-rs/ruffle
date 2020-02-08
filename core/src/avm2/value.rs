//! AVM2 values

use crate::avm2::object::Object;
use crate::avm2::Error;
use gc_arena::Collect;

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
    pub fn as_object(&self) -> Result<Object<'gc>, Error> {
        if let Value::Object(object) = self {
            Ok(*object)
        } else {
            Err(format!("Expected Object, found {:?}", self).into())
        }
    }
}
