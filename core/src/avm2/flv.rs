use crate::avm2::array::ArrayStorage;
use crate::avm2::object::{ArrayObject, DateObject, ScriptObject, TObject as _};
use crate::avm2::Value as Avm2Value;
use crate::context::UpdateContext;
use crate::string::AvmString;

use chrono::DateTime;
use flv_rs::{Value as FlvValue, Variable as FlvVariable};

fn avm2_object_from_flv_variables<'gc>(
    context: &mut UpdateContext<'gc>,
    variables: Vec<FlvVariable>,
) -> Avm2Value<'gc> {
    let info_object = ScriptObject::new_object(context);

    for value in variables {
        let property_name = value.name;

        info_object.set_dynamic_property(
            AvmString::new_utf8_bytes(context.gc(), property_name),
            value.data.to_avm2_value(context),
            context.gc(),
        );
    }

    info_object.into()
}

fn avm2_array_from_flv_values<'gc>(
    context: &mut UpdateContext<'gc>,
    values: Vec<FlvValue>,
) -> Avm2Value<'gc> {
    let storage = ArrayStorage::from_storage(
        values
            .iter()
            .map(|v| Some(v.clone().to_avm2_value(context)))
            .collect::<Vec<Option<Avm2Value<'gc>>>>(),
    );

    ArrayObject::from_storage(context, storage).into()
}

fn avm2_date_from_flv_date<'gc>(
    context: &mut UpdateContext<'gc>,
    unix_time: f64,
    _local_offset: i16,
) -> Avm2Value<'gc> {
    let date_time = DateTime::from_timestamp(unix_time as i64, 0).expect("invalid timestamp");

    DateObject::from_date_time(context, date_time).into()
}

pub trait FlvValueAvm2Ext<'gc> {
    fn to_avm2_value(self, context: &mut UpdateContext<'gc>) -> Avm2Value<'gc>;
}

impl<'gc> FlvValueAvm2Ext<'gc> for FlvValue<'_> {
    fn to_avm2_value(self, context: &mut UpdateContext<'gc>) -> Avm2Value<'gc> {
        match self {
            FlvValue::Object(variables) | FlvValue::EcmaArray(variables) => {
                avm2_object_from_flv_variables(context, variables)
            }
            FlvValue::StrictArray(values) => avm2_array_from_flv_values(context, values),
            FlvValue::String(string_data) | FlvValue::LongString(string_data) => {
                AvmString::new_utf8_bytes(context.gc(), string_data).into()
            }
            FlvValue::Date {
                unix_time,
                local_offset,
            } => avm2_date_from_flv_date(context, unix_time, local_offset),
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
