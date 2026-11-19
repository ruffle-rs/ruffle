use crate::avm2::error::{Error2004Type, make_error_2004, make_error_2183, make_error_2187};
use crate::avm2::function::FunctionArgs;
use crate::avm2::globals::slots::flash_geom_vector_3d as vector3d_slots;
use crate::avm2::object::{Matrix3DObject, VectorObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::vector::VectorStorage;
use crate::avm2::{Activation, Avm2StrRepresentable as _, Error, Object, TObject as _, Value};
use num_traits::Zero;
use ruffle_macros::Avm2Enum;
use ruffle_render::matrix3d::Matrix3D;

pub use crate::avm2::object::matrix_3d_allocator;

/// A 4x4 matrix, stored in column-major order, like `Matrix3D.rawData`.
type RawData = [f32; 16];

/// A value of `flash.geom.Orientation3D`, as passed to `recompose`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Avm2Enum)]
enum Orientation3D {
    #[avm2_variant("eulerAngles")]
    EulerAngles,
    #[avm2_variant("axisAngle")]
    AxisAngle,
    #[avm2_variant("quaternion")]
    Quaternion,
}

/// Reads the `x`, `y`, `z` and `w` components of a `Vector3D`, or `None` if it's null.
fn read_vector3d(vector: Object<'_>) -> [f64; 4] {
    [
        vector.get_slot(vector3d_slots::X).as_f64(),
        vector.get_slot(vector3d_slots::Y).as_f64(),
        vector.get_slot(vector3d_slots::Z).as_f64(),
        vector.get_slot(vector3d_slots::W).as_f64(),
    ]
}

/// Same as [`read_vector3d`] but accepts a value.
fn read_vector3d_value(value: Option<Value<'_>>) -> Option<[f64; 4]> {
    let object = value?.as_object()?;
    Some(read_vector3d(object))
}

fn dot3(a: &[f32], b: &[f32]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Creates a `Vector3D` with the given components.
fn vector3d_to_object<'gc>(
    activation: &mut Activation<'_, 'gc>,
    components: [f64; 4],
) -> Value<'gc> {
    let [x, y, z, w] = components;

    activation
        .avm2()
        .classes()
        .vector3d
        .construct(activation, &[x.into(), y.into(), z.into(), w.into()])
        .unwrap()
}

pub fn get_raw_data<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_matrix3d_object().unwrap();

    let number = activation.avm2().class_defs().number;
    let storage = VectorStorage::from_values(
        this.matrix_ref()
            .raw_data
            .iter()
            .map(|value| (*value).into())
            .collect(),
        false,
        Some(number),
    );

    Ok(VectorObject::from_vector(storage, activation).into())
}

/// Implements `Matrix3D.rawData`'s setter.
pub fn set_raw_data<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_matrix3d_object().unwrap();
    let Some(value) = args.try_get_object(0) else {
        return Ok(Value::Undefined);
    };
    let value = value.as_vector_storage().unwrap();
    if value.length() != 16 {
        return Ok(Value::Undefined);
    }

    let raw_data: RawData =
        std::array::from_fn(|i| value.get_optional(i).map(|v| v.as_f64() as f32).unwrap());

    this.replace_matrix(Matrix3D { raw_data });

    Ok(Value::Undefined)
}

/// Implements `Matrix3D.identity`.
pub fn identity<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_matrix3d_object().unwrap();

    this.replace_matrix(Matrix3D::IDENTITY);

    Ok(Value::Undefined)
}

/// Implements `Matrix3D.appendTranslation`.
pub fn append_translation<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_matrix3d_object().unwrap();
    let x = args.get_f64(0);
    let y = args.get_f64(1);
    let z = args.get_f64(2);

    let mut matrix = this.matrix_mut();
    matrix.raw_data[12] += x as f32;
    matrix.raw_data[13] += y as f32;
    matrix.raw_data[14] += z as f32;

    Ok(Value::Undefined)
}

/// Implements `Matrix3D.appendScale`.
pub fn append_scale<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_matrix3d_object().unwrap();
    let x = args.get_f64(0);
    let y = args.get_f64(1);
    let z = args.get_f64(2);

    if x.is_zero() || y.is_zero() || z.is_zero() {
        return Err(make_error_2183(activation));
    }

    let scale = Matrix3D::scale(x as f32, y as f32, z as f32);
    let result = scale.multiply(&this.matrix_ref());
    this.replace_matrix(result);

    Ok(Value::Undefined)
}

/// Implements `Matrix3D.append`.
pub fn append<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_matrix3d_object().unwrap();
    let lhs = args
        .get_object(activation, 0, "lhs")?
        .as_matrix3d_object()
        .unwrap();

    let result = lhs.matrix_ref().multiply(&this.matrix_ref());
    this.replace_matrix(result);

    Ok(Value::Undefined)
}

/// Implements `Matrix3D.prepend`.
pub fn prepend<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_matrix3d_object().unwrap();
    let rhs = args
        .get_object(activation, 0, "rhs")?
        .as_matrix3d_object()
        .unwrap();

    let result = this.matrix_ref().multiply(&rhs.matrix_ref());
    this.replace_matrix(result);

    Ok(Value::Undefined)
}

/// Implements `Matrix3D.prependTranslation`.
pub fn prepend_translation<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_matrix3d_object().unwrap();
    let x = args.get_f64(0);
    let y = args.get_f64(1);
    let z = args.get_f64(2);

    let translation = Matrix3D::translate(x as f32, y as f32, z as f32);
    let result = this.matrix_ref().multiply(&translation);
    this.replace_matrix(result);

    Ok(Value::Undefined)
}

/// Implements `Matrix3D.prependScale`.
pub fn prepend_scale<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_matrix3d_object().unwrap();
    let x = args.get_f64(0);
    let y = args.get_f64(1);
    let z = args.get_f64(2);

    if x.is_zero() || y.is_zero() || z.is_zero() {
        return Err(make_error_2183(activation));
    }

    let scale = Matrix3D::scale(x as f32, y as f32, z as f32);
    let result = this.matrix_ref().multiply(&scale);
    this.replace_matrix(result);

    Ok(Value::Undefined)
}

/// Implements `Matrix3D.appendRotation`.
///
/// Based on OpenFL: https://github.com/openfl/openfl/blob/971a4c9e43b5472fd84d73920a2b7c1b3d8d9257/src/openfl/geom/Matrix3D.hx
pub fn append_rotation<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_matrix3d_object().unwrap();
    let degrees = args.get_f64(0);
    let axis = args.get_object(activation, 1, "axis")?;
    let pivot_point = args.try_get_object(2);

    let [tx, ty, tz, _] = if let Some(pivot_point) = pivot_point {
        read_vector3d(pivot_point)
    } else {
        [0.0, 0.0, 0.0, 0.0]
    };

    let radian = degrees * std::f64::consts::PI / 180.0;
    let cos = radian.cos();
    let sin = radian.sin();

    let [mut x, mut y, mut z, _] = read_vector3d(axis);
    let mut x2 = x * x;
    let mut y2 = y * y;
    let mut z2 = z * z;
    let ls = x2 + y2 + z2;

    if ls != 0.0 {
        let l = ls.sqrt();
        x /= l;
        y /= l;
        z /= l;
        x2 /= ls;
        y2 /= ls;
        z2 /= ls;
    }

    let ccos = 1.0 - cos;

    #[rustfmt::skip]
    let m: RawData = [
        x2 + (y2 + z2) * cos,
        x * y * ccos + z * sin,
        x * z * ccos - y * sin,
        0.0,
        x * y * ccos - z * sin,
        y2 + (x2 + z2) * cos,
        y * z * ccos + x * sin,
        0.0,
        x * z * ccos + y * sin,
        y * z * ccos - x * sin,
        z2 + (x2 + y2) * cos,
        0.0,
        (tx * (y2 + z2) - x * (ty * y + tz * z)) * ccos + (ty * z - tz * y) * sin,
        (ty * (x2 + z2) - y * (tx * x + tz * z)) * ccos + (tz * x - tx * z) * sin,
        (tz * (x2 + y2) - z * (tx * x + ty * y)) * ccos + (tx * y - ty * x) * sin,
        1.0,
    ].map(|f| f as f32);

    let result = Matrix3D { raw_data: m }.multiply(&this.matrix_ref());
    this.replace_matrix(result);

    Ok(Value::Undefined)
}

/// Implements `Matrix3D.copyColumnTo`.
pub fn copy_column_to<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_matrix3d_object().unwrap();
    let column = args.get_u32(0);
    let vector3d = args.get_object(activation, 1, "vector3D")?;

    if column > 3 {
        return Err(make_error_2004(activation, Error2004Type::ArgumentError));
    }

    let raw_data = this.matrix_ref().raw_data;
    let base = column as usize * 4;

    vector3d.set_slot(vector3d_slots::X, raw_data[base].into(), activation)?;
    vector3d.set_slot(vector3d_slots::Y, raw_data[base + 1].into(), activation)?;
    vector3d.set_slot(vector3d_slots::Z, raw_data[base + 2].into(), activation)?;
    vector3d.set_slot(vector3d_slots::W, raw_data[base + 3].into(), activation)?;

    Ok(Value::Undefined)
}

/// Implements `Matrix3D.copyColumnFrom`.
pub fn copy_column_from<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_matrix3d_object().unwrap();
    let column = args.get_u32(0);
    let vector3d = args.get_object(activation, 1, "vector3D")?;

    if column > 3 {
        return Err(make_error_2004(activation, Error2004Type::ArgumentError));
    }

    let [x, y, z, w] = read_vector3d(vector3d);

    let mut matrix = this.matrix_mut();
    let base = column as usize * 4;
    matrix.raw_data[base] = x as f32;
    matrix.raw_data[base + 1] = y as f32;
    matrix.raw_data[base + 2] = z as f32;
    matrix.raw_data[base + 3] = w as f32;

    Ok(Value::Undefined)
}

/// Implements `Matrix3D.copyRowTo`.
pub fn copy_row_to<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_matrix3d_object().unwrap();
    let row = args.get_u32(0);
    let vector3d = args.get_object(activation, 1, "vector3D")?;

    if row > 3 {
        return Err(make_error_2004(activation, Error2004Type::ArgumentError));
    }

    let raw_data = this.matrix_ref().raw_data;
    let base = row as usize;

    vector3d.set_slot(vector3d_slots::X, raw_data[base].into(), activation)?;
    vector3d.set_slot(vector3d_slots::Y, raw_data[base + 4].into(), activation)?;
    vector3d.set_slot(vector3d_slots::Z, raw_data[base + 8].into(), activation)?;
    vector3d.set_slot(vector3d_slots::W, raw_data[base + 12].into(), activation)?;

    Ok(Value::Undefined)
}

/// Implements `Matrix3D.copyRowFrom`.
pub fn copy_row_from<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_matrix3d_object().unwrap();
    let row = args.get_u32(0);
    let vector3d = args.get_object(activation, 1, "vector3D")?;

    if row > 3 {
        return Err(make_error_2004(activation, Error2004Type::ArgumentError));
    }

    let [x, y, z, w] = read_vector3d(vector3d);

    let mut matrix = this.matrix_mut();
    let base = row as usize;
    matrix.raw_data[base] = x as f32;
    matrix.raw_data[base + 4] = y as f32;
    matrix.raw_data[base + 8] = z as f32;
    matrix.raw_data[base + 12] = w as f32;

    Ok(Value::Undefined)
}

/// Implements `Matrix3D.copyFrom`.
pub fn copy_from<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_matrix3d_object().unwrap();
    let source = args
        .get_object(activation, 0, "source")?
        .as_matrix3d_object()
        .unwrap();
    this.replace_matrix(source.matrix());
    Ok(Value::Undefined)
}

/// Implements `Matrix3D.copyRawDataFrom`.
pub fn copy_raw_data_from<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_matrix3d_object().unwrap();
    let source = args.get_object(activation, 0, "source")?;
    let source = source.as_vector_storage().unwrap();
    let index = args.get_u32(1) as usize;
    let do_transpose = args.get_bool(2);

    if index + 16 > source.length() {
        return Err(make_error_2004(activation, Error2004Type::ArgumentError));
    }

    let raw_data: RawData =
        std::array::from_fn(|i| source.get_optional(index + i).unwrap().as_f64() as f32);
    let mut matrix = Matrix3D { raw_data };

    if do_transpose {
        matrix.transpose_in_place();
    }

    this.replace_matrix(matrix);

    Ok(Value::Undefined)
}

/// Implements `Matrix3D.copyRawDataTo`.
pub fn copy_raw_data_to<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_matrix3d_object().unwrap();
    let dest = args
        .get_object(activation, 0, "dest")?
        .as_vector_object()
        .unwrap();
    let index = args.get_u32(1) as usize;
    let do_transpose = args.get_bool(2);

    let mut storage = dest.storage_mut(activation.gc());
    if index + 16 > storage.length() {
        storage.resize(index + 16, activation)?;
    }

    let mut matrix = this.matrix();
    if do_transpose {
        matrix.transpose_in_place();
    }

    for (i, value) in matrix.raw_data.into_iter().enumerate() {
        storage.set(index + i, value.into(), activation)?;
    }

    Ok(Value::Undefined)
}

/// Implements `Matrix3D.copyToMatrix3D`.
pub fn copy_to_matrix_3d<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_matrix3d_object().unwrap();
    let dest = args
        .get_object(activation, 0, "dest")?
        .as_matrix3d_object()
        .unwrap();
    dest.replace_matrix(this.matrix());
    Ok(Value::Undefined)
}

/// Implements `Matrix3D.clone`.
pub fn clone<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_matrix3d_object().unwrap();
    Ok(Matrix3DObject::new(activation.context, this.matrix()).into())
}

/// Implements `Matrix3D.position`'s getter.
pub fn get_position<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_matrix3d_object().unwrap();
    let mr = this.matrix_ref().raw_data;

    Ok(vector3d_to_object(
        activation,
        [mr[12] as f64, mr[13] as f64, mr[14] as f64, 0.0],
    ))
}

/// Implements `Matrix3D.position`'s setter.
pub fn set_position<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_matrix3d_object().unwrap();
    let Some(val) = args.try_get_object(0) else {
        return Ok(Value::Undefined);
    };

    let [x, y, z, _] = read_vector3d(val);

    let mut matrix = this.matrix_mut();
    matrix.raw_data[12] = x as f32;
    matrix.raw_data[13] = y as f32;
    matrix.raw_data[14] = z as f32;

    Ok(Value::Undefined)
}

/// Implements `Matrix3D.transpose`.
pub fn transpose<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_matrix3d_object().unwrap();
    this.matrix_mut().transpose_in_place();
    Ok(Value::Undefined)
}

/// Implements `Matrix3D.deltaTransformVector`.
pub fn delta_transform_vector<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_matrix3d_object().unwrap();
    let v = args.get_object(activation, 0, "vector")?;

    let mr = this.matrix_ref().raw_data;
    let [x, y, z, _] = read_vector3d(v);

    // deltaTransformVector() casts the vector to f32 first, losing accuracy.
    let [x, y, z] = [x as f32, y as f32, z as f32];

    Ok(vector3d_to_object(
        activation,
        [
            mr[0] * x + mr[4] * y + mr[8] * z,
            mr[1] * x + mr[5] * y + mr[9] * z,
            mr[2] * x + mr[6] * y + mr[10] * z,
            mr[3] * x + mr[7] * y + mr[11] * z,
        ]
        .map(|f| f as f64),
    ))
}

/// Implements `Matrix3D.transformVector`.
pub fn transform_vector<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_matrix3d_object().unwrap();
    let v = args.get_object(activation, 0, "vector")?;

    let mr = this.matrix_ref().raw_data;
    let [x, y, z, _] = read_vector3d(v);

    // transformVector() casts the vector to f32 first, losing accuracy.
    let [x, y, z] = [x as f32, y as f32, z as f32];

    Ok(vector3d_to_object(
        activation,
        [
            mr[0] * x + mr[4] * y + mr[8] * z + mr[12],
            mr[1] * x + mr[5] * y + mr[9] * z + mr[13],
            mr[2] * x + mr[6] * y + mr[10] * z + mr[14],
            mr[3] * x + mr[7] * y + mr[11] * z + mr[15],
        ]
        .map(|f| f as f64),
    ))
}

/// Implements `Matrix3D.transformVectors`.
pub fn transform_vectors<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_matrix3d_object().unwrap();
    let vin = args
        .get_object(activation, 0, "vin")?
        .as_vector_object()
        .unwrap();
    let vout = args
        .get_object(activation, 1, "vout")?
        .as_vector_object()
        .unwrap();

    let vin_length = vin.storage().length();
    let vout_length = vout.storage().length();
    let result_vecs_length = (vin_length / 3) * 3;

    if result_vecs_length > vout_length {
        vout.storage_mut(activation.gc())
            .resize(result_vecs_length, activation)?;
    }

    // transformVectors(), in contrast to transformVector() or
    // deltaTransformVector(), uses 64-bit operations with higher precision.
    let mr = this.matrix_ref().raw_data.map(|f| f as f64);

    // 'vin' and 'vout' may be the same object, so borrows of their storage
    // are scoped to a single access, never held across a read and a write.
    let mut i = 0;
    while i < result_vecs_length {
        let (x, y, z) = {
            let storage = vin.storage();
            (
                storage.get(i, activation)?.as_f64(),
                storage.get(i + 1, activation)?.as_f64(),
                storage.get(i + 2, activation)?.as_f64(),
            )
        };

        let result = [
            mr[0] * x + mr[4] * y + mr[8] * z + mr[12],
            mr[1] * x + mr[5] * y + mr[9] * z + mr[13],
            mr[2] * x + mr[6] * y + mr[10] * z + mr[14],
        ];

        let mut storage = vout.storage_mut(activation.gc());
        for (offset, value) in result.into_iter().enumerate() {
            storage.set(i + offset, value.into(), activation)?;
        }

        i += 3;
    }

    Ok(Value::Undefined)
}

/// Implements `Matrix3D.determinant`'s getter.
pub fn get_determinant<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_matrix3d_object().unwrap();
    Ok(this.matrix_ref().determinant().into())
}

/// Implements `Matrix3D.invert`.
pub fn invert<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_matrix3d_object().unwrap();
    let Some(result) = this.matrix_ref().invert() else {
        return Ok(false.into());
    };

    this.replace_matrix(result);

    Ok(true.into())
}

/// Implements `Matrix3D.recompose`.
///
/// Based on OpenFL: https://github.com/openfl/openfl/blob/971a4c9e43b5472fd84d73920a2b7c1b3d8d9257/src/openfl/geom/Matrix3D.hx#L1437
pub fn recompose<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_matrix3d_object().unwrap();

    let components = args.get_object(activation, 0, "components")?;
    let components = components.as_vector_storage().unwrap();

    let orientation = args.get_string_non_null(activation, 1, "orientationStyle")?;
    let Some(orientation) = Orientation3D::from_avm2_str(&orientation) else {
        return Err(make_error_2187(activation, orientation));
    };

    // A null component makes 'recompose' fail, leaving the matrix untouched.
    let (Some(translation), Some(rotation), Some(scale)) = (
        read_vector3d_value(components.get_optional(0)),
        read_vector3d_value(components.get_optional(1)),
        read_vector3d_value(components.get_optional(2)),
    ) else {
        return Ok(false.into());
    };

    let [translation_x, translation_y, translation_z, _] = translation.map(|f| f as f32);
    let [rotation_x, rotation_y, rotation_z, rotation_w] = rotation;
    let [scale_x, scale_y, scale_z, _] = scale.map(|f| f as f32);

    let mut rotation_matrix = [0.0; 16];
    match orientation {
        Orientation3D::EulerAngles => {
            // Flash Player calculates Euler angle rotations at f32 precision,
            // apart from the sines and cosines, which it calculates at f64
            // precision.
            let sin_cos = |angle: f64| {
                let (sin, cos) = f64::from(angle as f32).sin_cos();
                (sin as f32, cos as f32)
            };
            let (sx, cx) = sin_cos(rotation_x);
            let (sy, cy) = sin_cos(rotation_y);
            let (sz, cz) = sin_cos(rotation_z);
            let sy_cz = sy * cz;
            let sy_sz = sy * sz;

            rotation_matrix[0] = (cy * cz).into();
            rotation_matrix[1] = (cy * sz).into();
            rotation_matrix[2] = (-sy).into();
            rotation_matrix[4] = (sx * sy_cz - cx * sz).into();
            rotation_matrix[5] = (sx * sy_sz + cx * cz).into();
            rotation_matrix[6] = (sx * cy).into();
            rotation_matrix[8] = (cx * sy_cz + sx * sz).into();
            rotation_matrix[9] = (cx * sy_sz - sx * cz).into();
            rotation_matrix[10] = (cx * cy).into();
        }

        Orientation3D::AxisAngle => {
            // Flash Player normalizes the axis at f32 precision, scaling the
            // angle along with it, and calculates the rotation from that at
            // f64 precision.
            let [mut x, mut y, mut z, mut angle] =
                [rotation_x, rotation_y, rotation_z, rotation_w].map(|f| f as f32);

            let length_squared = x * x + y * y + z * z;
            if length_squared != 0.0 {
                let inverse_length = (1.0 / f64::from(length_squared).sqrt()) as f32;
                x *= inverse_length;
                y *= inverse_length;
                z *= inverse_length;
                angle *= inverse_length;
            }

            let (sin, cos) = f64::from(angle).sin_cos();
            let t = 1.0 - cos;
            let [xx, yy, zz, xy, xz, yz] =
                [x * x, y * y, z * z, x * y, x * z, y * z].map(f64::from);
            let [x, y, z] = [x, y, z].map(f64::from);

            rotation_matrix[0] = xx * t + cos;
            rotation_matrix[1] = xy * t + z * sin;
            rotation_matrix[2] = xz * t - y * sin;
            rotation_matrix[4] = xy * t - z * sin;
            rotation_matrix[5] = yy * t + cos;
            rotation_matrix[6] = yz * t + x * sin;
            rotation_matrix[8] = xz * t + y * sin;
            rotation_matrix[9] = yz * t - x * sin;
            rotation_matrix[10] = zz * t + cos;
        }

        Orientation3D::Quaternion => {
            // Flash Player calculates quaternion rotations at f32 precision.
            let [x, y, z, w] = [rotation_x, rotation_y, rotation_z, rotation_w].map(|f| f as f32);

            // Quaternions that aren't unit ones (or aren't finite) are
            // rejected, leaving the matrix untouched.
            let length_squared = w * w + x * x + y * y + z * z;
            if !(0.99999..=1.00001).contains(&length_squared) {
                return Err(make_error_2004(activation, Error2004Type::ArgumentError));
            }

            rotation_matrix[0] = (1.0 - 2.0 * (y * y + z * z)).into();
            rotation_matrix[1] = (2.0 * x * y + 2.0 * w * z).into();
            rotation_matrix[2] = (2.0 * x * z - 2.0 * w * y).into();
            rotation_matrix[4] = (2.0 * x * y - 2.0 * w * z).into();
            rotation_matrix[5] = (1.0 - 2.0 * (x * x + z * z)).into();
            rotation_matrix[6] = (2.0 * y * z + 2.0 * w * x).into();
            rotation_matrix[8] = (2.0 * x * z + 2.0 * w * y).into();
            rotation_matrix[9] = (2.0 * y * z - 2.0 * w * x).into();
            rotation_matrix[10] = (1.0 - 2.0 * (x * x + y * y)).into();
        }
    }
    rotation_matrix[15] = 1.0;

    let translation = Matrix3D::translate(translation_x, translation_y, translation_z);
    let rotation = Matrix3D {
        raw_data: rotation_matrix.map(|f| f as f32),
    };
    let scale = Matrix3D::scale(scale_x, scale_y, scale_z);

    // The order of operations is observable when some of the components are not
    // finite.
    let result = translation.multiply(&rotation.multiply(&scale));
    this.replace_matrix(result);

    Ok(true.into())
}

/// Implements `Matrix3D.decompose`.
///
/// Based on OpenFL: https://github.com/openfl/openfl/blob/971a4c9e43b5472fd84d73920a2b7c1b3d8d9257/src/openfl/geom/Matrix3D.hx#L437
pub fn decompose<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_matrix3d_object().unwrap();

    let orientation = args.get_string_non_null(activation, 0, "orientationStyle")?;
    let Some(orientation) = Orientation3D::from_avm2_str(&orientation) else {
        return Err(make_error_2187(activation, orientation));
    };

    let raw = this.matrix_ref().raw_data;
    let translation = [raw[12], raw[13], raw[14], 0.0].map(f64::from);

    // Flash Player removes any skew by orthonormalizing the columns at f32
    // precision (Gram-Schmidt). The lengths of the columns only use their X,
    // Y and Z components, but the projections onto the previous columns use
    // their W components too.
    let dot4 = |a: [f32; 4], b: [f32; 4]| dot3(&a, &b) + a[3] * b[3];
    let normalize = |v: [f32; 4], length: f32| {
        let inverse = 1.0 / length;
        v.map(|c| c * inverse)
    };
    let [x_column, y_column, z_column] =
        [0, 4, 8].map(|i| [raw[i], raw[i + 1], raw[i + 2], raw[i + 3]]);

    let scale_x = dot3(&x_column, &x_column).sqrt();
    let x_axis = normalize(x_column, scale_x);

    let xy = dot4(x_axis, y_column);
    let y_axis: [f32; 4] = std::array::from_fn(|i| y_column[i] - xy * x_axis[i]);
    let scale_y = dot3(&y_axis, &y_axis).sqrt();
    let y_axis = normalize(y_axis, scale_y);

    let xz = dot4(x_axis, z_column);
    let yz = dot4(y_axis, z_column);
    let z_axis: [f32; 4] = std::array::from_fn(|i| z_column[i] - xz * x_axis[i] - yz * y_axis[i]);
    let mut scale_z = dot3(&z_axis, &z_axis).sqrt();
    let mut z_axis = normalize(z_axis, scale_z);

    // The axes are only treated as mirrored when they're (almost) exactly
    // orthonormal, which they might not be when the W components are used.
    let cross = [
        x_axis[1] * y_axis[2] - x_axis[2] * y_axis[1],
        x_axis[2] * y_axis[0] - x_axis[0] * y_axis[2],
        x_axis[0] * y_axis[1] - x_axis[1] * y_axis[0],
    ];
    if dot3(&cross, &z_axis) < -0.99999 {
        scale_z = -scale_z;
        z_axis = z_axis.map(|c| -c);
    }

    // Scales close to 1 are reported as exactly 1.
    let scale = [scale_x, scale_y, scale_z, 0.0].map(|s| {
        if (f64::from(s) - 1.0).abs() < 0.001 {
            1.0
        } else {
            f64::from(s)
        }
    });

    let [m0, m1, m2, _] = x_axis;
    let [m4, m5, m6, _] = y_axis;
    let [m8, m9, m10, _] = z_axis;

    // Flash Player calculates quaternions at f32 precision, and axis-angle
    // rotations at f64 precision from those.
    let quaternion = || {
        let trace = m0 + m5 + m10;
        let half_sqrt = |v: f32| (v + 1.0).sqrt() / 2.0;

        if trace > 0.0 {
            let w = half_sqrt(trace);
            let k = 1.0 / (4.0 * w);
            [(m6 - m9) * k, (m8 - m2) * k, (m1 - m4) * k, w]
        } else if m0 > m5 && m0 > m10 {
            let x = half_sqrt(m0 - (m5 + m10));
            let k = 1.0 / (4.0 * x);
            [x, (m1 + m4) * k, (m8 + m2) * k, (m6 - m9) * k]
        } else if m5 > m10 {
            let y = half_sqrt(m5 - (m0 + m10));
            let k = 1.0 / (4.0 * y);
            [(m1 + m4) * k, y, (m6 + m9) * k, (m8 - m2) * k]
        } else {
            let z = half_sqrt(m10 - (m0 + m5));
            let k = 1.0 / (4.0 * z);
            [(m8 + m2) * k, (m6 + m9) * k, z, (m1 - m4) * k]
        }
    };

    let rotation = match orientation {
        Orientation3D::Quaternion => quaternion().map(f64::from),

        Orientation3D::AxisAngle => {
            let [x, y, z, w] = quaternion().map(f64::from);
            let angle = 2.0 * w.acos();
            let length = (1.0 - w * w).sqrt();

            if length == 0.0 {
                [0.0, 0.0, 0.0, angle]
            } else {
                [x / length, y / length, z / length, angle]
            }
        }

        Orientation3D::EulerAngles => {
            let [m0, m1, m2, m4, m5, m6, m10] = [m0, m1, m2, m4, m5, m6, m10].map(f64::from);
            let rotation_y = (-m2).asin();

            if rotation_y.cos() > 0.0 {
                [m6.atan2(m10), rotation_y, m1.atan2(m0), 0.0]
            } else {
                [m4.atan2(m5), rotation_y, 0.0, 0.0]
            }
        }
    }
    .map(|v| f64::from(v as f32));

    // NOTE: It looks like Flash Player returns uninitialized values here when
    // it doesn't write to a certain component, e.g. rotation.w in case of
    // "eulerAngles". Take this into account when testing.
    let components = [
        vector3d_to_object(activation, translation),
        vector3d_to_object(activation, rotation),
        vector3d_to_object(activation, scale),
    ];

    let vector3d = activation
        .avm2()
        .classes()
        .vector3d
        .inner_class_definition();
    let storage = VectorStorage::from_values(components.to_vec(), false, Some(vector3d));

    Ok(VectorObject::from_vector(storage, activation).into())
}
