use either::Either;
use ruffle_render::pixel_bender::{PixelBenderType, PixelBenderTypeOpcode};

use crate::avm2::error::{make_error_2004, Error2004Type};
use crate::avm2::{Activation, ArrayObject, ArrayStorage, Error, Object, Value};
use crate::context::UpdateContext;
use crate::ecma_conversions::f64_to_wrapping_i32;
use crate::string::AvmString;

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
        context: &mut UpdateContext<'gc>,
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
            Value::Object(Object::ArrayObject(o)) => Some(o.storage()),
            Value::Null => None,
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
            PixelBenderTypeOpcode::TBool => Ok(PixelBenderType::TBool(next_val(
                activation, &mut vals,
            )? as i16)),
            PixelBenderTypeOpcode::TBool2 => Ok(PixelBenderType::TBool2(
                next_val(activation, &mut vals)? as i16,
                next_val(activation, &mut vals)? as i16,
            )),
            PixelBenderTypeOpcode::TBool3 => Ok(PixelBenderType::TBool3(
                next_val(activation, &mut vals)? as i16,
                next_val(activation, &mut vals)? as i16,
                next_val(activation, &mut vals)? as i16,
            )),
            PixelBenderTypeOpcode::TBool4 => Ok(PixelBenderType::TBool4(
                next_val(activation, &mut vals)? as i16,
                next_val(activation, &mut vals)? as i16,
                next_val(activation, &mut vals)? as i16,
                next_val(activation, &mut vals)? as i16,
            )),
        }
    }

    fn as_avm2_value<'gc>(
        &self,
        context: &mut UpdateContext<'gc>,
        tint_as_int: bool,
    ) -> Result<Value<'gc>, Error<'gc>> {
        // Flash appears to use a uint/int if the float has no fractional part
        let cv = |f: f32| -> Value<'gc> {
            if f.fract() == 0.0 {
                f64_to_wrapping_i32(f as f64).into()
            } else {
                f.into()
            }
        };

        let storage = match self {
            PixelBenderType::TString(string) => {
                return Ok(AvmString::new_utf8(context.gc(), string).into());
            }
            &PixelBenderType::TInt(i) => {
                if tint_as_int {
                    return Ok(i.into());
                } else {
                    ArrayStorage::from_iter([i])
                }
            }
            &PixelBenderType::TFloat(f) => ArrayStorage::from_iter([cv(f)]),
            &PixelBenderType::TFloat2(f1, f2) => ArrayStorage::from_iter([cv(f1), cv(f2)]),
            &PixelBenderType::TFloat3(f1, f2, f3) => {
                ArrayStorage::from_iter([cv(f1), cv(f2), cv(f3)])
            }
            &PixelBenderType::TFloat4(f1, f2, f3, f4) => {
                ArrayStorage::from_iter([cv(f1), cv(f2), cv(f3), cv(f4)])
            }
            &PixelBenderType::TFloat2x2(floats) => ArrayStorage::from_iter(floats.map(cv)),
            &PixelBenderType::TFloat3x3(floats) => ArrayStorage::from_iter(floats.map(cv)),
            &PixelBenderType::TFloat4x4(floats) => ArrayStorage::from_iter(floats.map(cv)),
            &PixelBenderType::TInt2(i1, i2) | &PixelBenderType::TBool2(i1, i2) => {
                ArrayStorage::from_iter([i1, i2])
            }
            &PixelBenderType::TInt3(i1, i2, i3) | &PixelBenderType::TBool3(i1, i2, i3) => {
                ArrayStorage::from_iter([i1, i2, i3])
            }
            &PixelBenderType::TInt4(i1, i2, i3, i4) | &PixelBenderType::TBool4(i1, i2, i3, i4) => {
                ArrayStorage::from_iter([i1, i2, i3, i4])
            }
            &PixelBenderType::TBool(b) => ArrayStorage::from_iter([b]),
        };

        Ok(ArrayObject::from_storage(context, storage).into())
    }
}
