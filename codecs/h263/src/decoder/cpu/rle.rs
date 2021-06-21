//! Block run decompression

use crate::types::Block;
use std::cmp::{max, min};

const DEZIGZAG_MAPPING: [(u8, u8); 64] = [
    (0, 0),
    (1, 0),
    (0, 1),
    (0, 2),
    (1, 1),
    (2, 0),
    (3, 0),
    (2, 1),
    (1, 2),
    (0, 3),
    (0, 4),
    (1, 3),
    (2, 2),
    (3, 1),
    (4, 0),
    (5, 0),
    (4, 1),
    (3, 2),
    (2, 3),
    (1, 4),
    (0, 5),
    (0, 6),
    (1, 5),
    (2, 4),
    (3, 3),
    (4, 2),
    (5, 1),
    (6, 0),
    (7, 0),
    (6, 1),
    (5, 2),
    (4, 3),
    (3, 4),
    (2, 5),
    (1, 6),
    (0, 7),
    (1, 7),
    (2, 6),
    (3, 5),
    (4, 4),
    (5, 3),
    (6, 2),
    (7, 1),
    (7, 2),
    (6, 3),
    (5, 4),
    (4, 5),
    (3, 6),
    (2, 7),
    (3, 7),
    (4, 6),
    (5, 5),
    (6, 4),
    (7, 3),
    (7, 4),
    (6, 5),
    (5, 6),
    (4, 7),
    (5, 7),
    (6, 6),
    (7, 5),
    (7, 6),
    (6, 7),
    (7, 7),
];

/// Inverse RLE, dezigzag, and dequantize encoded block coefficient data.
///
/// `encoded_block` should be the block data as returned from `decode_block`.
/// `levels` will be filled with a row-major (x + y*8) decompressed list of
/// coefficients at the position `pos` (assuming a stride of
/// `samples_per_line`.)
///
/// This function assumes `levels` has already been initialized to zero. If the
/// levels array is reused, you must reinitialize it again.
pub fn inverse_rle(
    encoded_block: &Block,
    levels: &mut [[[f32; 8]; 8]],
    pos: (usize, usize),
    blk_per_line: usize,
    quant: u8,
) {
    let mut zigzag_index = 0;
    let block_id = pos.0 / 8 + (pos.1 / 8 * blk_per_line);
    let block = &mut levels[block_id];

    if let Some(dc) = encoded_block.intradc {
        block[0][0] = dc.into_level().into();
        zigzag_index += 1;
    }

    for tcoef in encoded_block.tcoef.iter() {
        zigzag_index += tcoef.run as usize;

        if zigzag_index >= DEZIGZAG_MAPPING.len() {
            return;
        }

        let (zig_x, zig_y) = DEZIGZAG_MAPPING[zigzag_index];
        let dequantized_level = quant as i16 * ((2 * tcoef.level.abs()) + 1);
        let parity = if quant % 2 == 1 { 0 } else { -1 };

        block[zig_x as usize][zig_y as usize] = min(
            2047,
            max(-2048, tcoef.level.signum() * (dequantized_level + parity)),
        )
        .into();
        zigzag_index += 1;
    }
}
