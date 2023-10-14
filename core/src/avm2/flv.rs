use crate::avm2::{Activation, TObject as _, Value as Avm2Value};
use crate::string::AvmString;
use flv_rs::{Value as FlvValue, Variable as FlvVariable};

fn avm2_object_from_flv_variables<'gc>(
    activation: &mut Activation<'_, 'gc>,
    variables: Vec<FlvVariable>,
) -> Avm2Value<'gc> {
    let info_object = activation
        .context
        .avm2
        .classes()
        .object
        .construct(activation, &[])
        .expect("Object construction should succeed");

    for value in variables {
        let property_name = value.name;

        info_object
            .set_public_property(
                AvmString::new_utf8_bytes(activation.context.gc_context, property_name),
                value.data.to_avm2_value(activation),
                activation,
            )
            .expect("valid set");
    }

    info_object.into()
}

pub trait FlvValueAvm2Ext<'gc> {
    fn to_avm2_value(self, activation: &mut Activation<'_, 'gc>) -> Avm2Value<'gc>;
}

impl<'gc> FlvValueAvm2Ext<'gc> for FlvValue<'_> {
    fn to_avm2_value(self, activation: &mut Activation<'_, 'gc>) -> Avm2Value<'gc> {
        match self {
            FlvValue::EcmaArray(values) => avm2_object_from_flv_variables(activation, values),
            FlvValue::String(string_data) | FlvValue::LongString(string_data) => {
                AvmString::new_utf8_bytes(activation.context.gc_context, string_data).into()
            }
            FlvValue::Number(value) => value.into(),
            FlvValue::Boolean(value) => value.into(),
            FlvValue::Null => Avm2Value::Null,
            FlvValue::Undefined => Avm2Value::Undefined,
            _ => {
                unimplemented!(
                    "FLV data to AVM2 data conversion unimplemented for {:?}",
                    self
                );
            }
        }
    }
}
