use crate::avm2::error::make_error_2187;
use crate::avm2::globals::slots::flash_geom_matrix_3d as matrix3d_slots;
use crate::avm2::globals::slots::flash_geom_vector_3d as vector3d_slots;
use crate::avm2::object::VectorObject;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::vector::VectorStorage;
use crate::avm2::{Activation, Avm2StrRepresentable as _, Error, Object, TObject as _, Value};
use crate::avm2_stub_method;
use ruffle_macros::Avm2Enum;

/// A 4x4 matrix, stored in column-major order, like `Matrix3D.rawData`.
type RawData = [f64; 16];

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
fn read_vector3d(value: Option<Value<'_>>) -> Option<[f64; 4]> {
    let object = value?.as_object()?;

    Some([
        object.get_slot(vector3d_slots::X).as_f64(),
        object.get_slot(vector3d_slots::Y).as_f64(),
        object.get_slot(vector3d_slots::Z).as_f64(),
        object.get_slot(vector3d_slots::W).as_f64(),
    ])
}

/// Multiplies two matrices.
fn multiply(lhs: &RawData, rhs: &RawData) -> RawData {
    let mut result = [0.0; 16];

    for column in 0..4 {
        for row in 0..4 {
            result[column * 4 + row] = lhs[row] * rhs[column * 4]
                + lhs[4 + row] * rhs[column * 4 + 1]
                + lhs[8 + row] * rhs[column * 4 + 2]
                + lhs[12 + row] * rhs[column * 4 + 3];
        }
    }

    result
}

/// Stores `raw_data` as the new `_rawData` of `this`.
fn set_raw_data<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    raw_data: RawData,
) -> Result<(), Error<'gc>> {
    let number = activation.avm2().class_defs().number;
    let storage = VectorStorage::from_values(
        raw_data.iter().map(|value| (*value).into()).collect(),
        false,
        Some(number),
    );

    let raw_data = VectorObject::from_vector(storage, activation);
    this.set_slot(matrix3d_slots::_RAW_DATA, raw_data.into(), activation)
}

/// Implements `Matrix3D.recompose`.
///
/// Based on OpenFL: https://github.com/openfl/openfl/blob/971a4c9e43b5472fd84d73920a2b7c1b3d8d9257/src/openfl/geom/Matrix3D.hx#L1437
pub fn recompose<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

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
        read_vector3d(components.get_optional(0)),
        read_vector3d(components.get_optional(1)),
        read_vector3d(components.get_optional(2)),
    ) else {
        return Ok(false.into());
    };

    let [translation_x, translation_y, translation_z, _] = translation;
    let [rotation_x, rotation_y, rotation_z, rotation_w] = rotation;
    let [scale_x, scale_y, scale_z, _] = scale;

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

    #[rustfmt::skip]
    let scale_matrix = [
        scale_x, 0.0,     0.0,     0.0,
        0.0,     scale_y, 0.0,     0.0,
        0.0,     0.0,     scale_z, 0.0,
        0.0,     0.0,     0.0,     1.0,
    ];

    #[rustfmt::skip]
    let translation_matrix = [
        1.0,           0.0,           0.0,           0.0,
        0.0,           1.0,           0.0,           0.0,
        0.0,           0.0,           1.0,           0.0,
        translation_x, translation_y, translation_z, 1.0,
    ];

    // The order of operations is observable when some of the components are not
    // finite.
    let raw_data = multiply(
        &translation_matrix,
        &multiply(&rotation_matrix, &scale_matrix),
    );

    set_raw_data(activation, this, raw_data)?;

    Ok(true.into())
}
