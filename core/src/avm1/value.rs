use crate::avm1::object::Object;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Error, UpdateContext};
use gc_arena::GcCell;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum Value<'gc> {
    Undefined,
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Object(GcCell<'gc, Object<'gc>>),
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

impl<'gc> From<GcCell<'gc, Object<'gc>>> for Value<'gc> {
    fn from(object: GcCell<'gc, Object<'gc>>) -> Self {
        Value::Object(object)
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
                Value::Object(other_value) => value.as_ptr() == other_value.as_ptr(),
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

    pub fn as_number(&self) -> f64 {
        // ECMA-262 2nd edtion s. 9.3 ToNumber
        use std::f64::NAN;
        match self {
            Value::Undefined => NAN,
            Value::Null => NAN,
            Value::Bool(false) => 0.0,
            Value::Bool(true) => 1.0,
            Value::Number(v) => *v,
            Value::String(v) => match v.as_str() {
                v if v.starts_with("0x") => {
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
                "" => 0.0,
                _ => v.parse().unwrap_or(NAN),
            },
            Value::Object(_object) => {
                log::error!("Unimplemented: Object ToNumber");
                0.0
            }
        }
    }

    pub fn from_bool_v1(value: bool, swf_version: u8) -> Value<'gc> {
        // SWF version 4 did not have true bools and will push bools as 0 or 1.
        // e.g. SWF19 p. 72:
        // "If the numbers are equal, true is pushed to the stack for SWF 5 and later. For SWF 4, 1 is pushed to the stack."
        if swf_version >= 5 {
            Value::Bool(value)
        } else {
            Value::Number(if value { 1.0 } else { 0.0 })
        }
    }

    pub fn into_string(self) -> String {
        match self {
            Value::Undefined => "undefined".to_string(),
            Value::Null => "null".to_string(),
            Value::Bool(v) => v.to_string(),
            Value::Number(v) => v.to_string(), // TODO(Herschel): Rounding for int?
            Value::String(v) => v,
            Value::Object(object) => object.read().as_string(),
        }
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
                Value::Object(object) => object.read().type_of(),
            }
            .to_string(),
        )
    }

    pub fn as_i32(&self) -> Result<i32, Error> {
        self.as_f64().map(|n| n as i32)
    }

    pub fn as_u32(&self) -> Result<u32, Error> {
        self.as_f64().map(|n| n as u32)
    }

    pub fn as_i64(&self) -> Result<i64, Error> {
        self.as_f64().map(|n| n as i64)
    }

    pub fn as_f64(&self) -> Result<f64, Error> {
        match *self {
            Value::Number(v) => Ok(v),
            _ => Err(format!("Expected Number, found {:?}", self).into()),
        }
    }

    pub fn as_string(&self) -> Result<&String, Error> {
        match self {
            Value::String(s) => Ok(s),
            _ => Err(format!("Expected String, found {:?}", self).into()),
        }
    }

    pub fn as_object(&self) -> Result<GcCell<'gc, Object<'gc>>, Error> {
        if let Value::Object(object) = self {
            Ok(object.to_owned())
        } else {
            Err(format!("Expected Object, found {:?}", self).into())
        }
    }

    pub fn call(
        &self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: GcCell<'gc, Object<'gc>>,
        args: &[Value<'gc>],
    ) -> Result<ReturnValue<'gc>, Error> {
        if let Value::Object(object) = self {
            object.read().call(avm, context, this, args)
        } else {
            Err(format!("Expected function, found {:?}", self).into())
        }
    }
}
