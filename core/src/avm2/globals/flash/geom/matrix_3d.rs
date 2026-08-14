use crate::avm2::error::{Error2004Type, make_error_2004, make_error_2183, make_error_2187};
use crate::avm2::globals::slots::flash_geom_vector_3d as vector3d_slots;
use crate::avm2::object::VectorObject;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::vector::VectorStorage;
use crate::avm2::{Activation, Avm2StrRepresentable as _, Error, Object, TObject as _, Value};
use crate::avm2_stub_method;
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
    _args: &[Value<'gc>],
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
    args: &[Value<'gc>],
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
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_matrix3d_object().unwrap();

    this.replace_matrix(Matrix3D::IDENTITY);

    Ok(Value::Undefined)
}

/// Implements `Matrix3D.appendTranslation`.
pub fn append_translation<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
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
    args: &[Value<'gc>],
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
    args: &[Value<'gc>],
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
    args: &[Value<'gc>],
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
    args: &[Value<'gc>],
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
    args: &[Value<'gc>],
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
    args: &[Value<'gc>],
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
    args: &[Value<'gc>],
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
    args: &[Value<'gc>],
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
    args: &[Value<'gc>],
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
    args: &[Value<'gc>],
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

/// Implements `Matrix3D.copyRawDataFrom`.
pub fn copy_raw_data_from<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
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
    args: &[Value<'gc>],
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

/// Implements `Matrix3D.position`'s getter.
pub fn get_position<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
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
    args: &[Value<'gc>],
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
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_matrix3d_object().unwrap();
    this.matrix_mut().transpose_in_place();
    Ok(Value::Undefined)
}

/// Implements `Matrix3D.deltaTransformVector`.
pub fn delta_transform_vector<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
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
    args: &[Value<'gc>],
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
    args: &[Value<'gc>],
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
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_matrix3d_object().unwrap();
    Ok(this.matrix_ref().determinant().into())
}

/// Implements `Matrix3D.invert`.
pub fn invert<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
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
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_matrix3d_object().unwrap();

    let components = args.get_object(activation, 0, "components")?;
    let components = components.as_vector_storage().unwrap();

    let orientation = args.get_string_non_null(activation, 1, "orientationStyle")?;
    let Some(orientation) = Orientation3D::from_avm2_str(&orientation) else {
        return Err(make_error_2187(activation, orientation));
    };

    if orientation == Orientation3D::Quaternion {
        // Flash rejects quaternions that aren't unit ones with an
        // ArgumentError, which we don't reproduce yet.
        avm2_stub_method!(
            activation,
            "flash.geom.Matrix3D",
            "recompose",
            "Orientation3D.QUATERNION"
        );
    }

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
            let cx = rotation_x.cos();
            let cy = rotation_y.cos();
            let cz = rotation_z.cos();
            let sx = rotation_x.sin();
            let sy = rotation_y.sin();
            let sz = rotation_z.sin();

            rotation_matrix[0] = cy * cz;
            rotation_matrix[1] = cy * sz;
            rotation_matrix[2] = -sy;
            rotation_matrix[4] = sx * sy * cz - cx * sz;
            rotation_matrix[5] = sx * sy * sz + cx * cz;
            rotation_matrix[6] = sx * cy;
            rotation_matrix[8] = cx * sy * cz + sx * sz;
            rotation_matrix[9] = cx * sy * sz - sx * cz;
            rotation_matrix[10] = cx * cy;
        }

        Orientation3D::AxisAngle | Orientation3D::Quaternion => {
            let (mut x, mut y, mut z, mut w) = (rotation_x, rotation_y, rotation_z, rotation_w);

            if orientation == Orientation3D::AxisAngle {
                x *= (w / 2.0).sin();
                y *= (w / 2.0).sin();
                z *= (w / 2.0).sin();
                w = (w / 2.0).cos();
            }

            rotation_matrix[0] = 1.0 - 2.0 * y * y - 2.0 * z * z;
            rotation_matrix[1] = 2.0 * x * y + 2.0 * w * z;
            rotation_matrix[2] = 2.0 * x * z - 2.0 * w * y;
            rotation_matrix[4] = 2.0 * x * y - 2.0 * w * z;
            rotation_matrix[5] = 1.0 - 2.0 * x * x - 2.0 * z * z;
            rotation_matrix[6] = 2.0 * y * z + 2.0 * w * x;
            rotation_matrix[8] = 2.0 * x * z + 2.0 * w * y;
            rotation_matrix[9] = 2.0 * y * z - 2.0 * w * x;
            rotation_matrix[10] = 1.0 - 2.0 * x * x - 2.0 * y * y;
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
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_matrix3d_object().unwrap();

    let orientation = args.get_string_non_null(activation, 0, "orientationStyle")?;
    let Some(orientation) = Orientation3D::from_avm2_str(&orientation) else {
        return Err(make_error_2187(activation, orientation));
    };

    let mut mr = this.matrix_ref().raw_data.map(|f| f as f64);

    let translation = [mr[12], mr[13], mr[14], 0.0];

    let scale_x = (mr[0] * mr[0] + mr[1] * mr[1] + mr[2] * mr[2]).sqrt();
    let scale_y = (mr[4] * mr[4] + mr[5] * mr[5] + mr[6] * mr[6]).sqrt();
    let mut scale_z = (mr[8] * mr[8] + mr[9] * mr[9] + mr[10] * mr[10]).sqrt();

    if mr[0] * (mr[5] * mr[10] - mr[6] * mr[9]) - mr[1] * (mr[4] * mr[10] - mr[6] * mr[8])
        + mr[2] * (mr[4] * mr[9] - mr[5] * mr[8])
        < 0.0
    {
        scale_z = -scale_z;
    }

    mr[0] /= scale_x;
    mr[1] /= scale_x;
    mr[2] /= scale_x;
    mr[4] /= scale_y;
    mr[5] /= scale_y;
    mr[6] /= scale_y;
    mr[8] /= scale_z;
    mr[9] /= scale_z;
    mr[10] /= scale_z;

    let mut rotation = [0.0; 4];

    match orientation {
        Orientation3D::AxisAngle => {
            rotation[3] = ((mr[0] + mr[5] + mr[10] - 1.0) / 2.0).acos();

            let length = ((mr[6] - mr[9]) * (mr[6] - mr[9])
                + (mr[8] - mr[2]) * (mr[8] - mr[2])
                + (mr[1] - mr[4]) * (mr[1] - mr[4]))
                .sqrt();

            if length != 0.0 {
                rotation[0] = (mr[6] - mr[9]) / length;
                rotation[1] = (mr[8] - mr[2]) / length;
                rotation[2] = (mr[1] - mr[4]) / length;
            }
        }

        Orientation3D::Quaternion => {
            let trace = mr[0] + mr[5] + mr[10];

            if trace > 0.0 {
                rotation[3] = (1.0 + trace).sqrt() / 2.0;

                rotation[0] = (mr[6] - mr[9]) / (4.0 * rotation[3]);
                rotation[1] = (mr[8] - mr[2]) / (4.0 * rotation[3]);
                rotation[2] = (mr[1] - mr[4]) / (4.0 * rotation[3]);
            } else if mr[0] > mr[5] && mr[0] > mr[10] {
                rotation[0] = (1.0 + mr[0] - mr[5] - mr[10]).sqrt() / 2.0;

                rotation[3] = (mr[6] - mr[9]) / (4.0 * rotation[0]);
                rotation[1] = (mr[1] + mr[4]) / (4.0 * rotation[0]);
                rotation[2] = (mr[8] + mr[2]) / (4.0 * rotation[0]);
            } else if mr[5] > mr[10] {
                rotation[1] = (1.0 + mr[5] - mr[0] - mr[10]).sqrt() / 2.0;

                rotation[0] = (mr[1] + mr[4]) / (4.0 * rotation[1]);
                rotation[3] = (mr[8] - mr[2]) / (4.0 * rotation[1]);
                rotation[2] = (mr[6] + mr[9]) / (4.0 * rotation[1]);
            } else {
                rotation[2] = (1.0 + mr[10] - mr[0] - mr[5]).sqrt() / 2.0;

                rotation[0] = (mr[8] + mr[2]) / (4.0 * rotation[2]);
                rotation[1] = (mr[6] + mr[9]) / (4.0 * rotation[2]);
                rotation[3] = (mr[1] - mr[4]) / (4.0 * rotation[2]);
            }
        }

        Orientation3D::EulerAngles => {
            rotation[1] = (-mr[2]).asin();

            if mr[2] != 1.0 && mr[2] != -1.0 {
                rotation[0] = mr[6].atan2(mr[10]);
                rotation[2] = mr[1].atan2(mr[0]);
            } else {
                rotation[2] = 0.0;
                rotation[0] = mr[4].atan2(mr[5]);
            }
        }
    }

    // NOTE: It looks like Flash Player returns uninitialized values here when
    // it doesn't write to a certain component, e.g. rotation.w in case of
    // "eulerAngles". Take this into account when testing.
    let components = [
        vector3d_to_object(activation, translation),
        vector3d_to_object(activation, rotation),
        vector3d_to_object(activation, [scale_x, scale_y, scale_z, 0.0]),
    ];

    let vector3d = activation
        .avm2()
        .classes()
        .vector3d
        .inner_class_definition();
    let storage = VectorStorage::from_values(components.to_vec(), false, Some(vector3d));

    Ok(VectorObject::from_vector(storage, activation).into())
}
