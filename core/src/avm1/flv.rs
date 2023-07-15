use crate::avm1::{Activation, ScriptObject, TObject as _, Value as Avm1Value};
use crate::string::AvmString;
use flv_rs::{Value as FlvValue, Variable as FlvVariable};

fn avm1_object_from_flv_variables<'gc>(
    activation: &mut Activation<'_, 'gc>,
    variables: Vec<FlvVariable>,
) -> Avm1Value<'gc> {
    let object_proto = activation.context.avm1.prototypes().object;
    let info_object = ScriptObject::new(activation.context.gc_context, Some(object_proto));

    for value in variables {
        let property_name = value.name;

        info_object
            .set(
                AvmString::new_utf8_bytes(activation.context.gc_context, property_name),
                value.data.to_avm1_value(activation),
                activation,
            )
            .expect("valid set");
    }

    info_object.into()
}

pub trait FlvValueAvm1Ext<'gc> {
    fn to_avm1_value(self, activation: &mut Activation<'_, 'gc>) -> Avm1Value<'gc>;
}

impl<'gc> FlvValueAvm1Ext<'gc> for FlvValue<'_> {
    fn to_avm1_value(self, activation: &mut Activation<'_, 'gc>) -> Avm1Value<'gc> {
        match self {
            FlvValue::EcmaArray(values) => avm1_object_from_flv_variables(activation, values),
            FlvValue::String(string_data) | FlvValue::LongString(string_data) => {
                AvmString::new_utf8_bytes(activation.context.gc_context, string_data).into()
            }
            FlvValue::Number(value) => value.into(),
            FlvValue::Boolean(value) => value.into(),
            FlvValue::Null => Avm1Value::Null,
            FlvValue::Undefined => Avm1Value::Undefined,
            _ => {
                unimplemented!(
                    "FLV data to AVM1 data conversion unimplemented for {:?}",
                    self
                );
            }
        }
    }
}
