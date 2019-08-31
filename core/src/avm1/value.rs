use crate::avm1::object::Object;
use crate::avm1::Error;
use gc_arena::{GcCell, MutationContext};

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

unsafe impl<'gc> gc_arena::Collect for Value<'gc> {
    fn trace(&self, cc: gc_arena::CollectionContext) {
        if let Value::Object(object) = self {
            object.trace(cc);
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

    pub fn into_number(self) -> f64 {
        // ECMA-262 2nd edtion s. 9.3 ToNumber
        use std::f64::NAN;
        match self {
            Value::Undefined => NAN,
            Value::Null => NAN,
            Value::Bool(false) => 0.0,
            Value::Bool(true) => 1.0,
            Value::Number(v) => v,
            Value::String(v) => v.parse().unwrap_or(NAN), // TODO(Herschel): Handle Infinity/etc.?
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
            Value::Object(_) => "[Object object]".to_string(), // TODO: object.toString()
        }
    }

    pub fn as_bool(&self) -> bool {
        match *self {
            Value::Bool(v) => v,
            Value::Number(v) => v != 0.0,
            // TODO(Herschel): Value::String(v) => ??
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

    pub fn as_object(&self) -> Result<&GcCell<'gc, Object<'gc>>, Error> {
        if let Value::Object(object) = self {
            Ok(object)
        } else {
            Err(format!("Expected Object, found {:?}", self).into())
        }
    }

    pub fn call(
        &self,
        gc_context: MutationContext<'gc, '_>,
        this: GcCell<'gc, Object<'gc>>,
        args: &[Value<'gc>],
    ) -> Result<Value<'gc>, Error> {
        if let Value::Object(object) = self {
            Ok(object.read().call(gc_context, this, args))
        } else {
            Err(format!("Expected function, found {:?}", self).into())
        }
    }
}
