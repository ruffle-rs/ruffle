//! Inverse discrete cosine transform

use std::cmp::{max, min};

/*
use lazy_static::lazy_static;
use std::f32::consts::{FRAC_1_SQRT_2, PI};

/// The 1D basis function of the H.263 IDCT.
///
/// `freq` is the frequency of the component
/// `x` is the point at which the cosine is to be computed
fn basis(freq: f32, x: f32) -> f32 {
    f32::cos(PI * ((x as f32 + 0.5) / 8.0) * freq as f32)
}

lazy_static! {
    /// Lookup table for `basis`.
    ///
    /// The outer parameter represents all valid `spatial` inputs, while the
    /// inner represents all valid `freq` inputs.
    /// Already includes the former CUV_TABLE factors.
    static ref BASIS_TABLE : [[f32; 8]; 8] = [
        [basis(0.0, 0.0) * FRAC_1_SQRT_2, basis(0.0, 1.0) * FRAC_1_SQRT_2, basis(0.0, 2.0) * FRAC_1_SQRT_2, basis(0.0, 3.0) * FRAC_1_SQRT_2, basis(0.0, 4.0) * FRAC_1_SQRT_2, basis(0.0, 5.0) * FRAC_1_SQRT_2, basis(0.0, 6.0) * FRAC_1_SQRT_2, basis(0.0, 7.0) * FRAC_1_SQRT_2],
        [basis(1.0, 0.0), basis(1.0, 1.0), basis(1.0, 2.0), basis(1.0, 3.0),basis(1.0, 4.0),basis(1.0, 5.0),basis(1.0, 6.0),basis(1.0, 7.0)],
        [basis(2.0, 0.0), basis(2.0, 1.0), basis(2.0, 2.0), basis(2.0, 3.0),basis(2.0, 4.0),basis(2.0, 5.0),basis(2.0, 6.0),basis(2.0, 7.0)],
        [basis(3.0, 0.0), basis(3.0, 1.0), basis(3.0, 2.0), basis(3.0, 3.0),basis(3.0, 4.0),basis(3.0, 5.0),basis(3.0, 6.0),basis(3.0, 7.0)],
        [basis(4.0, 0.0), basis(4.0, 1.0), basis(4.0, 2.0), basis(4.0, 3.0),basis(4.0, 4.0),basis(4.0, 5.0),basis(4.0, 6.0),basis(4.0, 7.0)],
        [basis(5.0, 0.0), basis(5.0, 1.0), basis(5.0, 2.0), basis(5.0, 3.0),basis(5.0, 4.0),basis(5.0, 5.0),basis(5.0, 6.0),basis(5.0, 7.0)],
        [basis(6.0, 0.0), basis(6.0, 1.0), basis(6.0, 2.0), basis(6.0, 3.0),basis(6.0, 4.0),basis(6.0, 5.0),basis(6.0, 6.0),basis(6.0, 7.0)],
        [basis(7.0, 0.0), basis(7.0, 1.0), basis(7.0, 2.0), basis(7.0, 3.0),basis(7.0, 4.0),basis(7.0, 5.0),basis(7.0, 6.0),basis(7.0, 7.0)],
    ];
}
*/

// This is the precomputed version of the table above
#[rustfmt::skip]
#[allow(clippy::approx_constant)]
const BASIS_TABLE: [[f32; 8]; 8] = [
    [ 0.70710677,  0.70710677,  0.70710677,  0.70710677,  0.70710677,  0.70710677,  0.70710677,  0.70710677, ],
    [ 0.98078525,  0.8314696,   0.5555702,   0.19509023, -0.19509032, -0.55557036, -0.83146966, -0.9807853,  ],
    [ 0.9238795,   0.38268343, -0.38268352, -0.9238796,  -0.9238795,  -0.38268313,  0.3826836,   0.92387956, ],
    [ 0.8314696,  -0.19509032, -0.9807853,  -0.55557,     0.55557007,  0.98078525,  0.19509007, -0.8314698,  ],
    [ 0.70710677, -0.70710677, -0.70710665,  0.707107,    0.70710677, -0.70710725, -0.70710653,  0.7071068,  ],
    [ 0.5555702,  -0.9807853,   0.19509041,  0.83146936, -0.8314698,  -0.19508928,  0.9807853,  -0.55557007, ],
    [ 0.38268343, -0.9238795,   0.92387974, -0.3826839,  -0.38268384,  0.9238793,  -0.92387974,  0.3826839,  ],
    [ 0.19509023, -0.55557,     0.83146936, -0.9807852,   0.98078525, -0.83147013,  0.55557114, -0.19508967, ],
];

/// Performs a one-dimensional IDCT on the input, using some lookup tables
/// for the scaling of the DC component, and for the cosine values to be used.
fn idct_1d(input: &[f32; 8], output: &mut [f32; 8]) {
    *output = [0.0; 8];
    for freq in 0..8 {
        for (i, out) in output.iter_mut().enumerate() {
            *out += input[freq] * BASIS_TABLE[freq][i];
        }
    }
}

/// Given a list of reconstructed IDCT levels, transform it out of the
/// frequency domain.
///
/// The input of this function, `block_levels`, is an arbitrarily-sized block of
/// decompressed, dezigzagged transform coefficients in row-major (x + y*8)
/// order. It must have a width equal to `samples_per_line` and dimensions
/// divisible by 8.
///
/// The `output` of this IDCT is represented as an arbitrarily-sized list of
/// `u8`s, also in row-major order and formatted in the same way as
/// `block_levels`. If this is an INTER block and predicted pixel data already
/// exists from the motion compensation step, you should pre-initialize the
/// output array with the result of said step so that the IDCT and summation
/// step can happen simultaneously. Otherwise, you should provide an array of
/// zeroes.
pub fn idct_channel(
    block_levels: &[[[f32; 8]; 8]],
    output: &mut [u8],
    blk_per_line: usize,
    output_samples_per_line: usize,
) {
    let output_height = output.len() / output_samples_per_line;
    let blk_height = block_levels.len() / blk_per_line;

    // Taking advantage of the separability of the 2D IDCT, and
    // decomposing it into two subsequent orthogonal series of 1D IDCTs.
    let mut idct_intermediate: [[f32; 8]; 8] = [[0.0; 8]; 8];
    let mut idct_output: [[f32; 8]; 8] = [[0.0; 8]; 8];

    for y_base in 0..blk_height {
        for x_base in 0..blk_per_line {
            let block_id = x_base + (y_base * blk_per_line);
            if block_id >= block_levels.len() {
                continue;
            }

            let block = &block_levels[block_id];

            for row in 0..8 {
                idct_1d(&block[row], &mut idct_output[row]);
                for (i, interim_row) in idct_intermediate.iter_mut().enumerate() {
                    interim_row[row] = idct_output[row][i]; // there is a transposition here
                }
            }

            for row in 0..8 {
                idct_1d(&idct_intermediate[row], &mut idct_output[row]);
            }

            // The inverted notation of the `x` and `y` loops is intended to
            // reverse the above transposition
            for (y_offset, idct_row) in idct_output.iter().enumerate() {
                for (x_offset, idct) in idct_row.iter().enumerate() {
                    let x = x_base * 8 + x_offset;
                    let y = y_base * 8 + y_offset;

                    if x >= output_samples_per_line {
                        continue;
                    }

                    if y >= output_height {
                        continue;
                    }

                    let clipped_idct =
                        min(255, max(-256, (idct / 4.0 + idct.signum() * 0.5) as i16));
                    let mocomp_pixel = output[x + (y * output_samples_per_line)] as u16 as i16;

                    output[x + (y * output_samples_per_line)] =
                        min(255, max(0, clipped_idct + mocomp_pixel)) as u8;
                }
            }
        }
    }
}
