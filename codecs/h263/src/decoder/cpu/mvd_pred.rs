//! Motion vector differential predictor
//!
//! This module covers functions relating to decoding motion vectors that have
//! been encoded as a series of differences from one or more predictors within
//! the current picture. Actual motion compensation happens in the gather step,
//! which is defined in the `gather` module.

use crate::decoder::picture::DecodedPicture;
use crate::types::{HalfPel, MotionVector, MotionVectorRange, PictureOption};

/// Produce a candidate motion vector predictor for a single block within a
/// given macroblock.
///
/// `current_predictors` is the set of already-decoded motion vectors for the
/// given block. You must decode all motion vectors within a block in the order
/// 0, 1, 2, and 3; any other order will result in incorrect results. This
/// means that vector decoding should be a process akin to:
///
/// 1. Predict the nth vector's candidate with `current_predictors`
/// 2. Decode the nth vector
/// 3. Store the nth vector in `current_predictors`
/// 4. Continue step 1 with n+1, stopping at the fourth motion vector
///
/// The `index` parameter indicates which of the four motion vectors within the
/// current block we are generating a candidate predictor for. This affects the
/// predictor sources.
pub fn predict_candidate(
    predictor_vectors: &[[MotionVector; 4]],
    current_predictors: &[MotionVector; 4],
    mb_per_line: usize,
    index: usize,
) -> MotionVector {
    let current_mb = predictor_vectors.len();
    let col_index = current_mb % mb_per_line;
    let mv1_pred = match index {
        0 | 2 if col_index == 0 => MotionVector::zero(),
        0 | 2 => predictor_vectors[current_mb as usize - 1][index + 1],
        1 | 3 => current_predictors[index - 1],
        _ => unreachable!(),
    };

    let line_index = current_mb / mb_per_line;
    let last_line_mb = (line_index.saturating_sub(1) * mb_per_line) + col_index;
    let mv2_pred = match index {
        0 | 1 if line_index == 0 => mv1_pred,
        0 | 1 => predictor_vectors
            .get(last_line_mb)
            .map(|mb| mb[index + 2])
            .unwrap_or(mv1_pred),
        2 | 3 => current_predictors[0],
        _ => unreachable!(),
    };

    let is_end_of_line = col_index == mb_per_line.saturating_sub(1);
    let mv3_pred = match index {
        0 | 1 if is_end_of_line => MotionVector::zero(),
        0 | 1 if line_index == 0 => mv1_pred,
        0 | 1 => predictor_vectors
            .get(last_line_mb + 1)
            .map(|mb| mb[2])
            .unwrap_or(mv1_pred),
        2 | 3 => current_predictors[1],
        _ => unreachable!(),
    };

    mv1_pred.median_of(mv2_pred, mv3_pred)
}

/// Decode a single component of a motion vector.
pub fn halfpel_decode(
    current_picture: &DecodedPicture,
    running_options: PictureOption,
    predictor: HalfPel,
    mvd: HalfPel,
    is_x: bool,
) -> HalfPel {
    let mut range = HalfPel::STANDARD_RANGE;
    let mut out = mvd + predictor;

    if running_options.contains(PictureOption::UNRESTRICTED_MOTION_VECTORS)
        && !current_picture.as_header().has_plusptype
    {
        if predictor.is_mv_within_range(HalfPel::STANDARD_RANGE) {
            return out;
        } else {
            range = HalfPel::EXTENDED_RANGE;
        }
    } else if running_options.contains(PictureOption::UNRESTRICTED_MOTION_VECTORS)
        && matches!(
            current_picture.as_header().motion_vector_range,
            Some(MotionVectorRange::Extended)
        )
    {
        if is_x {
            range = match current_picture.format().into_width_and_height() {
                Some((0..=352, _)) => HalfPel::EXTENDED_RANGE,
                Some((356..=704, _)) => HalfPel::EXTENDED_RANGE_QUADCIF,
                Some((708..=1408, _)) => HalfPel::EXTENDED_RANGE_SIXTEENCIF,
                Some((1412..=u16::MAX, _)) => HalfPel::EXTENDED_RANGE_BEYONDCIF,
                _ => HalfPel::EXTENDED_RANGE, // this is actually an error condition.
            };
        } else {
            range = match current_picture.format().into_width_and_height() {
                Some((_, 0..=288)) => HalfPel::EXTENDED_RANGE,
                Some((_, 292..=576)) => HalfPel::EXTENDED_RANGE_QUADCIF,
                Some((_, 580..=u16::MAX)) => HalfPel::EXTENDED_RANGE_SIXTEENCIF,
                _ => HalfPel::EXTENDED_RANGE, // this is actually an error condition.
            };
        }
    }

    if !out.is_mv_within_range(range) {
        out = mvd.invert() + predictor;
    }

    out
}

/// Given an encoded motion vector and it's predictor, produce the decoded,
/// ready-to-use motion vector.
pub fn mv_decode(
    current_picture: &DecodedPicture,
    running_options: PictureOption,
    predictor: MotionVector,
    mvd: MotionVector,
) -> MotionVector {
    let (mvx, mvy) = mvd.into();
    let (cpx, cpy) = predictor.into();

    let out_x = halfpel_decode(current_picture, running_options, cpx, mvx, true);
    let out_y = halfpel_decode(current_picture, running_options, cpy, mvy, false);

    (out_x, out_y).into()
}
