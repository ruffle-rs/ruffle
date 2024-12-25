use ruffle_render::pixel_bender::{PixelBenderType, PixelBenderTypeOpcode};

use crate::{
    avm2::{Activation, ArrayObject, ArrayStorage, Error, TObject, Value},
    ecma_conversions::f64_to_wrapping_i32,
    string::AvmString,
};

pub trait PixelBenderTypeExt {
    fn from_avm2_value<'gc>(
        activation: &mut Activation<'_, 'gc>,
        value: Value<'gc>,
        kind: &PixelBenderTypeOpcode,
    ) -> Result<Self, Error<'gc>>
    where
        Self: Sized;

    fn as_avm2_value<'gc>(
        &self,
        activation: &mut Activation<'_, 'gc>,
        tint_as_int: bool,
    ) -> Result<Value<'gc>, Error<'gc>>;
}

impl PixelBenderTypeExt for PixelBenderType {
    fn from_avm2_value<'gc>(
        activation: &mut Activation<'_, 'gc>,
        value: Value<'gc>,
        kind: &PixelBenderTypeOpcode,
    ) -> Result<Self, Error<'gc>>
    where
        Self: Sized,
    {
        let is_float = matches!(
            kind,
            PixelBenderTypeOpcode::TFloat
                | PixelBenderTypeOpcode::TFloat2
                | PixelBenderTypeOpcode::TFloat3
                | PixelBenderTypeOpcode::TFloat4
                | PixelBenderTypeOpcode::TFloat2x2
                | PixelBenderTypeOpcode::TFloat3x3
                | PixelBenderTypeOpcode::TFloat4x4
        );

        match value {
            Value::String(s) => Ok(PixelBenderType::TString(s.to_string())),
            Value::Number(n) => Ok(PixelBenderType::TFloat(n as f32)),
            Value::Integer(i) => Ok(PixelBenderType::TInt(i as i16)),
            Value::Object(o) => {
                if let Some(array) = o.as_array_storage() {
                    if is_float {
                        let mut vals = array.iter().map(|val| {
                            val.expect("Array with hole")
                                .coerce_to_number(activation)
                                .unwrap() as f32
                        });
                        match kind {
                            PixelBenderTypeOpcode::TFloat => {
                                Ok(PixelBenderType::TFloat(vals.next().unwrap()))
                            }
                            PixelBenderTypeOpcode::TFloat2 => Ok(PixelBenderType::TFloat2(
                                vals.next().unwrap(),
                                vals.next().unwrap(),
                            )),
                            PixelBenderTypeOpcode::TFloat3 => Ok(PixelBenderType::TFloat3(
                                vals.next().unwrap(),
                                vals.next().unwrap(),
                                vals.next().unwrap(),
                            )),
                            PixelBenderTypeOpcode::TFloat4 => Ok(PixelBenderType::TFloat4(
                                vals.next().unwrap(),
                                vals.next().unwrap(),
                                vals.next().unwrap(),
                                vals.next().unwrap(),
                            )),
                            PixelBenderTypeOpcode::TFloat2x2 => Ok(PixelBenderType::TFloat2x2(
                                vals.collect::<Vec<_>>().try_into().unwrap(),
                            )),
                            PixelBenderTypeOpcode::TFloat3x3 => Ok(PixelBenderType::TFloat3x3(
                                vals.collect::<Vec<_>>().try_into().unwrap(),
                            )),
                            PixelBenderTypeOpcode::TFloat4x4 => Ok(PixelBenderType::TFloat4x4(
                                vals.collect::<Vec<_>>().try_into().unwrap(),
                            )),
                            _ => unreachable!("Unexpected float kind {kind:?}"),
                        }
                    } else {
                        let mut vals = array.iter().map(|val| {
                            val.expect("Array with hole")
                                .coerce_to_i32(activation)
                                .unwrap() as i16
                        });
                        match kind {
                            PixelBenderTypeOpcode::TInt => {
                                Ok(PixelBenderType::TInt(vals.next().unwrap()))
                            }
                            PixelBenderTypeOpcode::TInt2 => Ok(PixelBenderType::TInt2(
                                vals.next().unwrap(),
                                vals.next().unwrap(),
                            )),
                            PixelBenderTypeOpcode::TInt3 => Ok(PixelBenderType::TInt3(
                                vals.next().unwrap(),
                                vals.next().unwrap(),
                                vals.next().unwrap(),
                            )),
                            PixelBenderTypeOpcode::TInt4 => Ok(PixelBenderType::TInt4(
                                vals.next().unwrap(),
                                vals.next().unwrap(),
                                vals.next().unwrap(),
                                vals.next().unwrap(),
                            )),
                            _ => unreachable!("Unexpected int kind {kind:?}"),
                        }
                    }
                } else {
                    panic!("Unexpected object {o:?}")
                }
            }
            _ => panic!("Unexpected value {value:?}"),
        }
    }
    fn as_avm2_value<'gc>(
        &self,
        activation: &mut Activation<'_, 'gc>,
        tint_as_int: bool,
    ) -> Result<Value<'gc>, Error<'gc>> {
        // Flash appears to use a uint/int if the float has no fractional part
        let cv = |f: &f32| -> Value<'gc> {
            if f.fract() == 0.0 {
                f64_to_wrapping_i32(*f as f64).into()
            } else {
                (*f).into()
            }
        };
        let vals: Vec<Value<'gc>> = match self {
            PixelBenderType::TString(string) => {
                return Ok(AvmString::new_utf8(activation.gc(), string).into());
            }
            PixelBenderType::TInt(i) => {
                if tint_as_int {
                    return Ok((*i).into());
                } else {
                    vec![(*i).into()]
                }
            }
            PixelBenderType::TFloat(f) => vec![cv(f)],
            PixelBenderType::TFloat2(f1, f2) => vec![cv(f1), cv(f2)],
            PixelBenderType::TFloat3(f1, f2, f3) => vec![cv(f1), cv(f2), cv(f3)],
            PixelBenderType::TFloat4(f1, f2, f3, f4) => vec![cv(f1), cv(f2), cv(f3), cv(f4)],
            PixelBenderType::TFloat2x2(floats) => floats.iter().map(cv).collect(),
            PixelBenderType::TFloat3x3(floats) => floats.iter().map(cv).collect(),
            PixelBenderType::TFloat4x4(floats) => floats.iter().map(cv).collect(),
            PixelBenderType::TInt2(i1, i2) => vec![(*i1).into(), (*i2).into()],
            PixelBenderType::TInt3(i1, i2, i3) => vec![(*i1).into(), (*i2).into(), (*i3).into()],
            PixelBenderType::TInt4(i1, i2, i3, i4) => {
                vec![(*i1).into(), (*i2).into(), (*i3).into(), (*i4).into()]
            }
        };
        let storage = ArrayStorage::from_args(&vals);
        Ok(ArrayObject::from_storage(activation, storage).into())
    }
}
