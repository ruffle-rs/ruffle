use crate::avm1::Value as Avm1Value;
use crate::context::UpdateContext;
use crate::string::AvmString;
use serde::{Deserialize, Serialize};

/// A value that can be recieved as part of a debug command
/// This is separate from the AVM* values as it cannot hold a Gc ptr and must be serializable
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum DValue {
    String(String),
    Int(i32),
    Number(f64),
    Bool(bool),
    Null,
    Undefined,
    Object { kind: String },
}

impl DValue {
    pub(crate) fn as_avm1<'gc>(&self, context: &mut UpdateContext<'_, 'gc>) -> Avm1Value<'gc> {
        match self {
            // Objects can only be sent for now, not recieved
            Self::Null | Self::Object { .. } => Avm1Value::Null,
            Self::Undefined => Avm1Value::Undefined,
            Self::Int(v) => Avm1Value::Number(*v as f64),
            Self::Number(v) => Avm1Value::Number(*v),
            Self::Bool(b) => Avm1Value::Bool(*b),
            Self::String(s) => Avm1Value::String(AvmString::new_utf8(context.gc_context, s)),
        }
    }
}

impl<'gc> From<Avm1Value<'gc>> for DValue {
    fn from(value: Avm1Value<'gc>) -> Self {
        match value {
            Avm1Value::Undefined => Self::Undefined,
            Avm1Value::Null => Self::Null,
            Avm1Value::Bool(b) => Self::Bool(b),
            Avm1Value::Number(n) => {
                if let Ok(i) = n.to_string().parse::<i32>() {
                    Self::Int(i)
                } else {
                    Self::Number(n)
                }
            }
            //TODO: send wstrs
            Avm1Value::String(s) => Self::String(s.to_utf8_lossy().to_string()),

            Avm1Value::Object(o) => Self::Object {
                kind: format!("{:?}", o),
            },
        }
    }
}
