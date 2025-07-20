use either::Either;
use ruffle_render::pixel_bender::{PixelBenderType, PixelBenderTypeOpcode};

use crate::{
    avm2::{
        error::{make_error_2004, Error2004Type},
        Activation, ArrayObject, ArrayStorage, Error, Value,
    },
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
        fn next_val<'gc>(
            activation: &mut Activation<'_, 'gc>,
            vals: &mut impl Iterator<Item = Value<'gc>>,
        ) -> Result<f64, Error<'gc>> {
            let Some(val) = vals.next() else {
                return Ok(0.0);
            };

            val.try_as_f64()
                .ok_or_else(|| make_error_2004(activation, Error2004Type::ArgumentError))
        }

        let array_storage = match value {
            Value::Object(ref o) => o.as_array_storage(),
            Value::Null | Value::Undefined => None,
            _ => unreachable!("value should be an array"),
        };

        let mut vals = if let Some(ref array) = array_storage {
            Either::Left(array.iter().map(|v| v.unwrap_or(Value::Integer(0))))
        } else {
            Either::Right(std::iter::empty())
        };

        match kind {
            PixelBenderTypeOpcode::TFloat => Ok(PixelBenderType::TFloat(
                //
                next_val(activation, &mut vals)? as f32,
            )),
            PixelBenderTypeOpcode::TFloat2 => Ok(PixelBenderType::TFloat2(
                next_val(activation, &mut vals)? as f32,
                next_val(activation, &mut vals)? as f32,
            )),
            PixelBenderTypeOpcode::TFloat3 => Ok(PixelBenderType::TFloat3(
                next_val(activation, &mut vals)? as f32,
                next_val(activation, &mut vals)? as f32,
                next_val(activation, &mut vals)? as f32,
            )),
            PixelBenderTypeOpcode::TFloat4 => Ok(PixelBenderType::TFloat4(
                next_val(activation, &mut vals)? as f32,
                next_val(activation, &mut vals)? as f32,
                next_val(activation, &mut vals)? as f32,
                next_val(activation, &mut vals)? as f32,
            )),
            PixelBenderTypeOpcode::TFloat2x2 => Ok(PixelBenderType::TFloat2x2(
                // TODO use core::array::try_from_fn when it's stable
                (0..4)
                    .map(|_| next_val(activation, &mut vals).map(|v| v as f32))
                    .collect::<Result<Vec<f32>, Error<'gc>>>()?
                    .try_into()
                    .unwrap(),
            )),
            PixelBenderTypeOpcode::TFloat3x3 => Ok(PixelBenderType::TFloat3x3(
                // TODO use core::array::try_from_fn when it's stable
                (0..9)
                    .map(|_| next_val(activation, &mut vals).map(|v| v as f32))
                    .collect::<Result<Vec<f32>, Error<'gc>>>()?
                    .try_into()
                    .unwrap(),
            )),
            PixelBenderTypeOpcode::TFloat4x4 => Ok(PixelBenderType::TFloat4x4(
                // TODO use core::array::try_from_fn when it's stable
                (0..16)
                    .map(|_| next_val(activation, &mut vals).map(|v| v as f32))
                    .collect::<Result<Vec<f32>, Error<'gc>>>()?
                    .try_into()
                    .unwrap(),
            )),
            PixelBenderTypeOpcode::TInt => Ok(PixelBenderType::TInt(
                //
                next_val(activation, &mut vals)? as i16,
            )),
            PixelBenderTypeOpcode::TInt2 => Ok(PixelBenderType::TInt2(
                next_val(activation, &mut vals)? as i16,
                next_val(activation, &mut vals)? as i16,
            )),
            PixelBenderTypeOpcode::TInt3 => Ok(PixelBenderType::TInt3(
                next_val(activation, &mut vals)? as i16,
                next_val(activation, &mut vals)? as i16,
                next_val(activation, &mut vals)? as i16,
            )),
            PixelBenderTypeOpcode::TInt4 => Ok(PixelBenderType::TInt4(
                next_val(activation, &mut vals)? as i16,
                next_val(activation, &mut vals)? as i16,
                next_val(activation, &mut vals)? as i16,
                next_val(activation, &mut vals)? as i16,
            )),
            PixelBenderTypeOpcode::TString => Ok(PixelBenderType::TString(
                vals.next()
                    .and_then(|v| v.coerce_to_string(activation).ok())
                    .map(|s| s.to_string())
                    .unwrap_or_default(),
            )),
            PixelBenderTypeOpcode::TBool => Ok(PixelBenderType::TBool(
                next_val(activation, &mut vals)? as i16 != 0,
            )),
            PixelBenderTypeOpcode::TBool2 => Ok(PixelBenderType::TBool2(
                next_val(activation, &mut vals)? as i16 != 0,
                next_val(activation, &mut vals)? as i16 != 0,
            )),
            PixelBenderTypeOpcode::TBool3 => Ok(PixelBenderType::TBool3(
                next_val(activation, &mut vals)? as i16 != 0,
                next_val(activation, &mut vals)? as i16 != 0,
                next_val(activation, &mut vals)? as i16 != 0,
            )),
            PixelBenderTypeOpcode::TBool4 => Ok(PixelBenderType::TBool4(
                next_val(activation, &mut vals)? as i16 != 0,
                next_val(activation, &mut vals)? as i16 != 0,
                next_val(activation, &mut vals)? as i16 != 0,
                next_val(activation, &mut vals)? as i16 != 0,
            )),
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
            PixelBenderType::TBool(b) => vec![(*b as i16).into()],
            PixelBenderType::TBool2(b1, b2) => vec![(*b1 as i16).into(), (*b2 as i16).into()],
            PixelBenderType::TBool3(b1, b2, b3) => vec![
                (*b1 as i16).into(),
                (*b2 as i16).into(),
                (*b3 as i16).into(),
            ],
            PixelBenderType::TBool4(b1, b2, b3, b4) => {
                vec![
                    (*b1 as i16).into(),
                    (*b2 as i16).into(),
                    (*b3 as i16).into(),
                    (*b4 as i16).into(),
                ]
            }
        };
        let storage = ArrayStorage::from_args(&vals);
        Ok(ArrayObject::from_storage(activation, storage).into())
    }
}
