//! Block decoding

#![allow(clippy::unusual_byte_groupings)]

use crate::decoder::DecoderOption;
use crate::error::{Error, Result};
use crate::parser::reader::H263Reader;
use crate::parser::vlc::{Entry, Entry::*};
use crate::types::{Block, IntraDc, MacroblockType, Picture, PictureOption, TCoefficient};
use std::io::Read;

/// Represents a partially decoded short `TCOEF` entry.
#[derive(Clone, Debug, PartialEq, Eq)]
enum ShortTCoefficient {
    /// Indicates that a long `TCOEF` entry follows in the bitstream.
    EscapeToLong,

    /// An almost-fully-decoded short `TCOEF` entry.
    Run {
        last: bool,

        /// The size of the zero-coefficient run.
        run: u8,

        /// The magnitude of the non-zero coefficient.
        ///
        /// It's sign bit directly follows in the bitstream.
        level: u8,
    },
}

use ShortTCoefficient::*;

/// The table of TCOEF values.
///
/// `ESCAPE` is encoded as an `EscapeToLong`, the actual coded values are not
/// decoded by this table. Same for the sign bit, which should be read after
/// the `Run`.
const TCOEF_TABLE: [Entry<Option<ShortTCoefficient>>; 207] = [
    Fork(8, 1), //x, slot 0
    Fork(2, 3), //1x, slot 1
    End(Some(Run {
        last: false,
        run: 0,
        level: 1,
    })), //10, slot 2
    Fork(4, 5), //11x, slot 3
    End(Some(Run {
        last: false,
        run: 1,
        level: 1,
    })), //110, slot 4
    Fork(6, 7), //111x, slot 5
    End(Some(Run {
        last: false,
        run: 2,
        level: 1,
    })), //1110, slot 6
    End(Some(Run {
        last: false,
        run: 0,
        level: 2,
    })), //1111, slot 7
    Fork(28, 9), //0x, slot 8
    Fork(15, 10), //01x, slot 9
    Fork(12, 11), //011x, slot 10
    End(Some(Run {
        last: true,
        run: 0,
        level: 1,
    })), //0111, slot 11
    Fork(13, 14), //0110x, slot 12
    End(Some(Run {
        last: false,
        run: 4,
        level: 1,
    })), //01100, slot 13
    End(Some(Run {
        last: false,
        run: 3,
        level: 1,
    })), //01101, slot 14
    Fork(16, 23), //010x, slot 15
    Fork(17, 20), //0100x, slot 16
    Fork(18, 19), //01000x, slot 17
    End(Some(Run {
        last: false,
        run: 9,
        level: 1,
    })), //010000, slot 18
    End(Some(Run {
        last: false,
        run: 8,
        level: 1,
    })), //010001, slot 19
    Fork(21, 22), //01001x, slot 20
    End(Some(Run {
        last: false,
        run: 7,
        level: 1,
    })), //010010, slot 21
    End(Some(Run {
        last: false,
        run: 6,
        level: 1,
    })), //010011, slot 22
    Fork(25, 24), //0101x, slot 23
    End(Some(Run {
        last: false,
        run: 5,
        level: 1,
    })), //01011, slot 24
    Fork(26, 27), //01010x, slot 25
    End(Some(Run {
        last: false,
        run: 1,
        level: 2,
    })), //010100, slot 26
    End(Some(Run {
        last: false,
        run: 0,
        level: 3,
    })), //010101, slot 27
    Fork(52, 29), //00x, slot 28
    Fork(37, 30), //001x, slot 29
    Fork(31, 34), //0011x, slot 30
    Fork(32, 33), //00110x, slot 31
    End(Some(Run {
        last: true,
        run: 4,
        level: 1,
    })), //001100, slot 32
    End(Some(Run {
        last: true,
        run: 3,
        level: 1,
    })), //001101, slot 33
    Fork(35, 36), //00111x, slot 34
    End(Some(Run {
        last: true,
        run: 2,
        level: 1,
    })), //001110, slot 35
    End(Some(Run {
        last: true,
        run: 1,
        level: 1,
    })), //001111, slot 36
    Fork(38, 45), //0010x, slot 37
    Fork(39, 42), //00100x, slot 38
    Fork(40, 41), //001000x, slot 39
    End(Some(Run {
        last: true,
        run: 8,
        level: 1,
    })), //0010000, slot 40
    End(Some(Run {
        last: true,
        run: 7,
        level: 1,
    })), //0010001, slot 41
    Fork(43, 44), //001001x, slot 42
    End(Some(Run {
        last: true,
        run: 6,
        level: 1,
    })), //0010010, slot 43
    End(Some(Run {
        last: true,
        run: 5,
        level: 1,
    })), //0010011, slot 44
    Fork(46, 49), //00101x, slot 45
    Fork(47, 48), //001010x, slot 46
    End(Some(Run {
        last: false,
        run: 12,
        level: 1,
    })), //0010100, slot 47
    End(Some(Run {
        last: false,
        run: 11,
        level: 1,
    })), //0010101, slot 48
    Fork(50, 51), //001011x, slot 49
    End(Some(Run {
        last: false,
        run: 10,
        level: 1,
    })), //0010110, slot 50
    End(Some(Run {
        last: false,
        run: 0,
        level: 4,
    })), //0010111, slot 51
    Fork(90, 53), //000x, slot 52
    Fork(69, 54), //0001x, slot 53
    Fork(55, 62), //00011x, slot 54
    Fork(56, 59), //000110x, slot 55
    Fork(57, 58), //0001100x, slot 56
    End(Some(Run {
        last: true,
        run: 11,
        level: 1,
    })), //00011000, slot 57
    End(Some(Run {
        last: true,
        run: 10,
        level: 1,
    })), //00011001, slot 58
    Fork(60, 61), //0001101x, slot 59
    End(Some(Run {
        last: true,
        run: 9,
        level: 1,
    })), //00011010, slot 60
    End(Some(Run {
        last: false,
        run: 14,
        level: 1,
    })), //00011011, slot 61
    Fork(63, 66), //000111x, slot 62
    Fork(64, 65), //0001110x, slot 63
    End(Some(Run {
        last: false,
        run: 13,
        level: 1,
    })), //00011100, slot 64
    End(Some(Run {
        last: false,
        run: 2,
        level: 2,
    })), //00011101, slot 65
    Fork(67, 68), //0001111x, slot 66
    End(Some(Run {
        last: false,
        run: 1,
        level: 3,
    })), //00011110, slot 67
    End(Some(Run {
        last: false,
        run: 0,
        level: 5,
    })), //00011111, slot 68
    Fork(77, 70), //00010x, slot 69
    Fork(71, 74), //000101x, slot 70
    Fork(72, 73), //0001010x, slot 71
    End(Some(Run {
        last: true,
        run: 15,
        level: 1,
    })), //00010100, slot 72
    End(Some(Run {
        last: true,
        run: 14,
        level: 1,
    })), //00010101, slot 73
    Fork(75, 76), //0001011x, slot 74
    End(Some(Run {
        last: true,
        run: 13,
        level: 1,
    })), //00010110, slot 75
    End(Some(Run {
        last: true,
        run: 12,
        level: 1,
    })), //00010111, slot 76
    Fork(78, 85), //000100x, slot 77
    Fork(79, 82), //0001000x, slot 78
    Fork(80, 81), //00010000x, slot 79
    End(Some(Run {
        last: false,
        run: 16,
        level: 1,
    })), //000100000, slot 80
    End(Some(Run {
        last: false,
        run: 15,
        level: 1,
    })), //000100001, slot 81
    Fork(83, 84), //00010001x, slot 82
    End(Some(Run {
        last: false,
        run: 4,
        level: 2,
    })), //000100010, slot 83
    End(Some(Run {
        last: false,
        run: 3,
        level: 2,
    })), //000100011, slot 84
    Fork(86, 89), //0001001x, slot 85
    Fork(87, 88), //00010010x, slot 86
    End(Some(Run {
        last: false,
        run: 0,
        level: 7,
    })), //000100100, slot 87
    End(Some(Run {
        last: false,
        run: 0,
        level: 6,
    })), //000100101, slot 88
    End(Some(Run {
        last: true,
        run: 16,
        level: 1,
    })), //00010011x, slot 89
    Fork(124, 91), //0000x, slot 90
    Fork(92, 109), //00001x, slot 91
    Fork(93, 102), //000010x, slot 92
    Fork(94, 99), //0000100x, slot 93
    Fork(95, 98), //00001000x, slot 94
    Fork(96, 97), //000010000x, slot 95
    End(Some(Run {
        last: false,
        run: 0,
        level: 9,
    })), //0000100000, slot 96
    End(Some(Run {
        last: false,
        run: 0,
        level: 8,
    })), //0000100001, slot 97
    End(Some(Run {
        last: true,
        run: 24,
        level: 1,
    })), //000010001, slot 98
    Fork(100, 101), //00001001x, slot 99
    End(Some(Run {
        last: true,
        run: 23,
        level: 1,
    })), //000010010, slot 100
    End(Some(Run {
        last: true,
        run: 22,
        level: 1,
    })), //000010011, slot 101
    Fork(103, 106), //0000101x, slot 102
    Fork(104, 105), //00001010x, slot 103
    End(Some(Run {
        last: true,
        run: 21,
        level: 1,
    })), //000010100, slot 104
    End(Some(Run {
        last: true,
        run: 20,
        level: 1,
    })), //000010101, slot 105
    Fork(107, 108), //00001011x, slot 106
    End(Some(Run {
        last: true,
        run: 19,
        level: 1,
    })), //000010110, slot 107
    End(Some(Run {
        last: true,
        run: 18,
        level: 1,
    })), //000010111, slot 108
    Fork(110, 117), //000011x, slot 109
    Fork(111, 114), //0000110x, slot 110
    Fork(112, 113), //00001100x, slot 111
    End(Some(Run {
        last: true,
        run: 17,
        level: 1,
    })), //000011000, slot 112
    End(Some(Run {
        last: true,
        run: 0,
        level: 2,
    })), //000011001, slot 113
    Fork(115, 116), //00001101x, slot 114
    End(Some(Run {
        last: false,
        run: 22,
        level: 1,
    })), //000011010, slot 115
    End(Some(Run {
        last: false,
        run: 21,
        level: 1,
    })), //000011011, slot 116
    Fork(118, 121), //0000111x, slot 117
    Fork(119, 120), //00001110x, slot 118
    End(Some(Run {
        last: false,
        run: 20,
        level: 1,
    })), //000011100, slot 119
    End(Some(Run {
        last: false,
        run: 19,
        level: 1,
    })), //000011101, slot 120
    Fork(122, 123), //00001111x, slot 121
    End(Some(Run {
        last: false,
        run: 18,
        level: 1,
    })), //000011110, slot 122
    End(Some(Run {
        last: false,
        run: 17,
        level: 1,
    })), //000011111, slot 123
    Fork(174, 125), //00000x, slot 124
    Fork(127, 126), //000001x, slot 125
    End(Some(EscapeToLong)), //0000011, slot 126
    Fork(128, 143), //0000010x, slot 127
    Fork(129, 136), //00000100x, slot 128
    Fork(130, 133), //000001000x, slot 129
    Fork(131, 132), //0000010000x, slot 130
    End(Some(Run {
        last: false,
        run: 0,
        level: 12,
    })), //00000100000, slot 131
    End(Some(Run {
        last: false,
        run: 1,
        level: 5,
    })), //00000100001, slot 132
    Fork(134, 135), //0000010001x, slot 133
    End(Some(Run {
        last: false,
        run: 23,
        level: 1,
    })), //00000100010, slot 134
    End(Some(Run {
        last: false,
        run: 24,
        level: 1,
    })), //00000100011, slot 135
    Fork(137, 140), //000001001x, slot 136
    Fork(138, 139), //0000010010x, slot 137
    End(Some(Run {
        last: true,
        run: 29,
        level: 1,
    })), //00000100100, slot 138
    End(Some(Run {
        last: true,
        run: 30,
        level: 1,
    })), //00000100101, slot 139
    Fork(141, 142), //0000010011x, slot 140
    End(Some(Run {
        last: true,
        run: 31,
        level: 1,
    })), //00000100110, slot 141
    End(Some(Run {
        last: true,
        run: 32,
        level: 1,
    })), //00000100111, slot 142
    Fork(144, 159), //00000101x, slot 143
    Fork(145, 152), //000001010x, slot 144
    Fork(146, 149), //0000010100x, slot 145
    Fork(147, 148), //00000101000x, slot 146
    End(Some(Run {
        last: false,
        run: 1,
        level: 6,
    })), //000001010000, slot 147
    End(Some(Run {
        last: false,
        run: 2,
        level: 4,
    })), //000001010001, slot 148
    Fork(150, 151), //00000101001x, slot 149
    End(Some(Run {
        last: false,
        run: 4,
        level: 3,
    })), //000001010010, slot 150
    End(Some(Run {
        last: false,
        run: 5,
        level: 3,
    })), //000001010011, slot 151
    Fork(153, 156), //0000010101x, slot 152
    Fork(154, 155), //00000101010x, slot 153
    End(Some(Run {
        last: false,
        run: 6,
        level: 3,
    })), //000001010100, slot 154
    End(Some(Run {
        last: false,
        run: 10,
        level: 2,
    })), //000001010101, slot 155
    Fork(157, 158), //00000101011x, slot 156
    End(Some(Run {
        last: false,
        run: 25,
        level: 1,
    })), //000001010110, slot 157
    End(Some(Run {
        last: false,
        run: 26,
        level: 1,
    })), //000001010111, slot 158
    Fork(160, 167), //000001011x, slot 159
    Fork(161, 164), //0000010110x, slot 160
    Fork(162, 163), //00000101100x, slot 161
    End(Some(Run {
        last: true,
        run: 33,
        level: 1,
    })), //000001011000, slot 162
    End(Some(Run {
        last: true,
        run: 34,
        level: 1,
    })), //000001011001, slot 163
    Fork(165, 166), //00000101101x, slot 164
    End(Some(Run {
        last: true,
        run: 35,
        level: 1,
    })), //000001011010, slot 165
    End(Some(Run {
        last: true,
        run: 36,
        level: 1,
    })), //000001011011, slot 166
    Fork(168, 171), //0000010111x, slot 167
    Fork(169, 170), //00000101110x, slot 168
    End(Some(Run {
        last: true,
        run: 37,
        level: 1,
    })), //000001011100, slot 169
    End(Some(Run {
        last: true,
        run: 38,
        level: 1,
    })), //000001011101, slot 170
    Fork(172, 173), //00000101111x, slot 171
    End(Some(Run {
        last: true,
        run: 39,
        level: 1,
    })), //000001011110, slot 172
    End(Some(Run {
        last: true,
        run: 40,
        level: 1,
    })), //000001011111, slot 173
    Fork(190, 175), //000000x, slot 174
    Fork(176, 183), //0000001x, slot 175
    Fork(177, 180), //00000010x, slot 176
    Fork(178, 179), //000000100x, slot 177
    End(Some(Run {
        last: false,
        run: 9,
        level: 2,
    })), //0000001000, slot 178
    End(Some(Run {
        last: false,
        run: 8,
        level: 2,
    })), //0000001001, slot 179
    Fork(181, 182), //000000101x, slot 180
    End(Some(Run {
        last: false,
        run: 7,
        level: 2,
    })), //0000001010, slot 181
    End(Some(Run {
        last: false,
        run: 6,
        level: 2,
    })), //0000001011, slot 182
    Fork(184, 187), //00000011x, slot 183
    Fork(185, 186), //000000110x, slot 184
    End(Some(Run {
        last: false,
        run: 5,
        level: 2,
    })), //0000001100, slot 185
    End(Some(Run {
        last: false,
        run: 3,
        level: 3,
    })), //0000001101, slot 186
    Fork(188, 189), //000000111x, slot 187
    End(Some(Run {
        last: false,
        run: 2,
        level: 3,
    })), //0000001110, slot 188
    End(Some(Run {
        last: false,
        run: 1,
        level: 4,
    })), //0000001111, slot 189
    Fork(198, 191), //0000000x, slot 190
    Fork(192, 195), //00000001x, slot 191
    Fork(193, 194), //000000010x, slot 192
    End(Some(Run {
        last: true,
        run: 28,
        level: 1,
    })), //0000000100, slot 193
    End(Some(Run {
        last: true,
        run: 27,
        level: 1,
    })), //0000000101, slot 194
    Fork(196, 197), //000000011x, slot 195
    End(Some(Run {
        last: true,
        run: 26,
        level: 1,
    })), //0000000110, slot 196
    End(Some(Run {
        last: true,
        run: 25,
        level: 1,
    })), //0000000111, slot 197
    Fork(206, 199), //00000000x, slot 198
    Fork(200, 203), //000000001x, slot 199
    Fork(201, 202), //0000000010x, slot 200
    End(Some(Run {
        last: true,
        run: 1,
        level: 2,
    })), //00000000100, slot 201
    End(Some(Run {
        last: true,
        run: 0,
        level: 3,
    })), //00000000101, slot 202
    Fork(204, 205), //0000000011x, slot 203
    End(Some(Run {
        last: false,
        run: 0,
        level: 11,
    })), //00000000110, slot 204
    End(Some(Run {
        last: false,
        run: 0,
        level: 10,
    })), //00000000111, slot 205
    End(None),  //000000000x, slot 206
];

/// Decode a block from the bitstream.
///
/// The `running_options` should be the set of currently in-force options
/// present on the currently-decoded picture. This is not entirely equivalent
/// to the current picture's option set as some options can carry forward from
/// picture to picture without being explicitly mentioned.
///
/// The `macroblock_type` should be the `MacroblockType` recovered from the
/// currently-decoded macroblock.
///
/// `tcoef_present` should be flagged if the particular block being decoded is
/// flagged in the corresponding macroblock's `CodedBlockPattern` as having
/// transform coefficients.
pub fn decode_block<R>(
    reader: &mut H263Reader<R>,
    decoder_options: DecoderOption,
    picture: &Picture,
    running_options: PictureOption,
    macroblock_type: MacroblockType,
    mut tcoef_present: bool,
) -> Result<Block>
where
    R: Read,
{
    reader.with_transaction(|reader| {
        let intradc = if macroblock_type.is_intra() {
            Some(IntraDc::from_u8(reader.read_u8()?).ok_or(Error::InvalidIntraDc)?)
        } else {
            None
        };

        let mut tcoef = Vec::new();
        while tcoef_present {
            let short_tcoef = reader.read_vlc(&TCOEF_TABLE[..])?;

            match short_tcoef.ok_or(Error::InvalidShortCoefficient)? {
                EscapeToLong => {
                    let level_width = if decoder_options
                        .contains(DecoderOption::SORENSON_SPARK_BITSTREAM)
                        && picture.version == Some(1)
                    {
                        if reader.read_bits::<u8>(1)? == 1 {
                            11
                        } else {
                            7
                        }
                    } else {
                        8
                    };

                    let last = reader.read_bits::<u8>(1)? == 1;
                    let run: u8 = reader.read_bits(6)?;
                    let level = reader.read_signed_bits(level_width)?;

                    if level == 0 {
                        return Err(Error::InvalidLongCoefficient);
                    }

                    //TODO: Modified Quantization (Annex T)
                    if level == i16::MAX << level_width {
                        if running_options.contains(PictureOption::MODIFIED_QUANTIZATION) {
                            return Err(Error::UnimplementedDecoding);
                        } else {
                            return Err(Error::InvalidLongCoefficient);
                        }
                    }

                    tcoef.push(TCoefficient {
                        is_short: false,
                        run,
                        level,
                    });

                    tcoef_present = !last;
                }
                Run { last, run, level } => {
                    let sign: u8 = reader.read_bits(1)?;
                    if sign == 0 {
                        tcoef.push(TCoefficient {
                            is_short: true,
                            run,
                            level: level as i16,
                        })
                    } else {
                        tcoef.push(TCoefficient {
                            is_short: true,
                            run,
                            level: -(level as i16),
                        })
                    }

                    tcoef_present = !last;
                }
            };
        }

        Ok(Block { intradc, tcoef })
    })
}

#[cfg(test)]
mod tests {
    use crate::decoder::DecoderOption;
    use crate::parser::block::{decode_block, ShortTCoefficient, TCOEF_TABLE};
    use crate::parser::reader::H263Reader;
    use crate::types::{
        Block, IntraDc, MacroblockType, Picture, PictureOption, PictureTypeCode, TCoefficient,
    };

    #[test]
    #[allow(clippy::inconsistent_digit_grouping)]
    fn tcoef_table() {
        let bit_pattern = vec![
            0b10_1111_01,
            0b0101_0010,
            0b111_00011,
            0b111_00010,
            0b0101_0001,
            0b00100_000,
            0b0100001_0,
            0b00010000,
            0b0_0000000,
            0b0111_0000,
            0b0000110_0,
            0b00001000,
            0b00_110_010,
            0b100_00011,
            0b110_00000,
            0b01111_000,
            0b00100001,
            0b00000101,
            0b0000_1110,
            0b00011101,
            0b00000011,
            0b10_000001,
            0b010001_01,
            0b101_00010,
            0b0011_0000,
            0b001101_01,
            0b100_00010,
            0b0010_0000,
            0b01010010,
            0b01011_000,
            0b0001100_0,
            0b00001010,
            0b011_01001,
            0b1_0000001,
            0b011_00000,
            0b1010100_0,
            0b10010_000,
            0b0001010_0,
            0b10001_000,
            0b0001001_0,
            0b10000_000,
            0b0001000_0,
            0b010110_00,
            0b00010101,
            0b01_001010,
            0b1_0010100,
            0b00011100,
            0b00011011,
            0b00010000,
            0b1_0001000,
            0b00_000011,
            0b111_00001,
            0b1110_0000,
            0b11101_000,
            0b011100_00,
            0b0011011_0,
            0b00011010,
            0b00000100,
            0b010_00000,
            0b100011_00,
            0b00010101,
            0b10_000001,
            0b010111_01,
            0b11_000011,
            0b001_00000,
            0b000101_00,
            0b1111_0000,
            0b0000100_0,
            0b01110_001,
            0b101_00110,
            0b0_0010011,
            0b0010010_0,
            0b010001_00,
            0b10000_000,
            0b11010_000,
            0b11001_000,
            0b11000_000,
            0b10111_000,
            0b10110_000,
            0b10101_000,
            0b10100_000,
            0b10011_000,
            0b011000_00,
            0b0010111_0,
            0b00010110,
            0b00001010,
            0b1_0000101,
            0b00_000010,
            0b011_00001,
            0b0010_0000,
            0b10001_000,
            0b0000111_0,
            0b00000011,
            0b0_0000000,
            0b101_00000,
            0b00100_000,
            0b00100100,
            0b00000100,
            0b101_00000,
            0b100110_00,
            0b00010011,
            0b1_0000010,
            0b11000_000,
            0b00101100,
            0b1_0000010,
            0b11010_000,
            0b00101101,
            0b1_0000010,
            0b11100_000,
            0b00101110,
            0b1_0000010,
            0b11110_000,
            0b00101111,
            0b1_0000011,
            0b00000000,
            0b00000000,
        ];
        let mut reader = H263Reader::from_source(&bit_pattern[..]);

        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 0,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 0,
                level: 2
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 0,
                level: 3
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 0,
                level: 4
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 0,
                level: 5
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 0,
                level: 6
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 0,
                level: 7
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 0,
                level: 8
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 0,
                level: 9
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 0,
                level: 10
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 0,
                level: 11
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 0,
                level: 12
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 1,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 1,
                level: 2
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 1,
                level: 3
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 1,
                level: 4
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 1,
                level: 5
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 1,
                level: 6
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 2,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 2,
                level: 2
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 2,
                level: 3
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 2,
                level: 4
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 3,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 3,
                level: 2
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 3,
                level: 3
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 4,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 4,
                level: 2
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 4,
                level: 3
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 5,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 5,
                level: 2
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 5,
                level: 3
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 6,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 6,
                level: 2
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 6,
                level: 3
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 7,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 7,
                level: 2
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 8,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 8,
                level: 2
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 9,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 9,
                level: 2
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 10,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 10,
                level: 2
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 11,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 12,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 13,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 14,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 15,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 16,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 17,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 18,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 19,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 20,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 21,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 22,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 23,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 24,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 25,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: false,
                run: 26,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: true,
                run: 0,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: true,
                run: 0,
                level: 2
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: true,
                run: 0,
                level: 3
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: true,
                run: 1,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: true,
                run: 1,
                level: 2
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: true,
                run: 2,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: true,
                run: 3,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: true,
                run: 4,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: true,
                run: 5,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: true,
                run: 6,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: true,
                run: 7,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: true,
                run: 8,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: true,
                run: 9,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: true,
                run: 10,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: true,
                run: 11,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: true,
                run: 12,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: true,
                run: 13,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: true,
                run: 14,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: true,
                run: 15,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: true,
                run: 16,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: true,
                run: 17,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: true,
                run: 18,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: true,
                run: 19,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: true,
                run: 20,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: true,
                run: 21,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: true,
                run: 22,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: true,
                run: 23,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: true,
                run: 24,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: true,
                run: 25,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: true,
                run: 26,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: true,
                run: 27,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: true,
                run: 28,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: true,
                run: 29,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: true,
                run: 30,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: true,
                run: 31,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: true,
                run: 32,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: true,
                run: 33,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: true,
                run: 34,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: true,
                run: 35,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: true,
                run: 36,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: true,
                run: 37,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: true,
                run: 38,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: true,
                run: 39,
                level: 1
            })
        );
        assert_eq!(
            reader.read_vlc(&TCOEF_TABLE).unwrap(),
            Some(ShortTCoefficient::Run {
                last: true,
                run: 40,
                level: 1
            })
        );
    }

    #[test]
    fn empty_inter_block() {
        let bitstream = [0];
        let mut reader = H263Reader::from_source(&bitstream[..]);

        let picture = Picture {
            version: None,
            temporal_reference: 0,
            format: None,
            options: PictureOption::empty(),
            has_plusptype: false,
            has_opptype: false,
            picture_type: PictureTypeCode::PFrame,
            motion_vector_range: None,
            slice_submode: None,
            scalability_layer: None,
            reference_picture_selection_mode: None,
            prediction_reference: None,
            backchannel_message: None,
            reference_picture_resampling: None,
            quantizer: 1,
            multiplex_bitstream: None,
            pb_reference: None,
            pb_quantizer: None,
            extra: Vec::new(),
        };

        assert_eq!(
            Block {
                intradc: None,
                tcoef: vec![]
            },
            decode_block(
                &mut reader,
                DecoderOption::empty(),
                &picture,
                PictureOption::empty(),
                MacroblockType::Inter,
                false
            )
            .unwrap()
        )
    }

    #[test]
    fn empty_intra_block() {
        let bitstream = [0x63];
        let mut reader = H263Reader::from_source(&bitstream[..]);

        let picture = Picture {
            version: None,
            temporal_reference: 0,
            format: None,
            options: PictureOption::empty(),
            has_plusptype: false,
            has_opptype: false,
            picture_type: PictureTypeCode::IFrame,
            motion_vector_range: None,
            slice_submode: None,
            scalability_layer: None,
            reference_picture_selection_mode: None,
            prediction_reference: None,
            backchannel_message: None,
            reference_picture_resampling: None,
            quantizer: 1,
            multiplex_bitstream: None,
            pb_reference: None,
            pb_quantizer: None,
            extra: Vec::new(),
        };

        assert_eq!(
            Block {
                intradc: IntraDc::from_level(0x318),
                tcoef: vec![]
            },
            decode_block(
                &mut reader,
                DecoderOption::empty(),
                &picture,
                PictureOption::empty(),
                MacroblockType::Intra,
                false
            )
            .unwrap()
        )
    }

    #[test]
    fn long_coded_inter_block() {
        let bitstream = [0x06, 0x0C, 0x14, 0x1C, 0xC1, 0x00];
        let mut reader = H263Reader::from_source(&bitstream[..]);

        let picture = Picture {
            version: None,
            temporal_reference: 0,
            format: None,
            options: PictureOption::empty(),
            has_plusptype: false,
            has_opptype: false,
            picture_type: PictureTypeCode::IFrame,
            motion_vector_range: None,
            slice_submode: None,
            scalability_layer: None,
            reference_picture_selection_mode: None,
            prediction_reference: None,
            backchannel_message: None,
            reference_picture_resampling: None,
            quantizer: 1,
            multiplex_bitstream: None,
            pb_reference: None,
            pb_quantizer: None,
            extra: Vec::new(),
        };

        assert_eq!(
            Block {
                intradc: None,
                tcoef: vec![
                    TCoefficient {
                        is_short: false,
                        run: 3,
                        level: 5
                    },
                    TCoefficient {
                        is_short: false,
                        run: 12,
                        level: 16
                    }
                ]
            },
            decode_block(
                &mut reader,
                DecoderOption::empty(),
                &picture,
                PictureOption::empty(),
                MacroblockType::Inter,
                true
            )
            .unwrap()
        )
    }

    #[test]
    fn long_coded_intra_block() {
        let bitstream = [0x63, 0x06, 0x0C, 0x14, 0x1C, 0xC1, 0x00];
        let mut reader = H263Reader::from_source(&bitstream[..]);

        let picture = Picture {
            version: None,
            temporal_reference: 0,
            format: None,
            options: PictureOption::empty(),
            has_plusptype: false,
            has_opptype: false,
            picture_type: PictureTypeCode::IFrame,
            motion_vector_range: None,
            slice_submode: None,
            scalability_layer: None,
            reference_picture_selection_mode: None,
            prediction_reference: None,
            backchannel_message: None,
            reference_picture_resampling: None,
            quantizer: 1,
            multiplex_bitstream: None,
            pb_reference: None,
            pb_quantizer: None,
            extra: Vec::new(),
        };

        assert_eq!(
            Block {
                intradc: IntraDc::from_level(0x318),
                tcoef: vec![
                    TCoefficient {
                        is_short: false,
                        run: 3,
                        level: 5
                    },
                    TCoefficient {
                        is_short: false,
                        run: 12,
                        level: 16
                    }
                ]
            },
            decode_block(
                &mut reader,
                DecoderOption::empty(),
                &picture,
                PictureOption::empty(),
                MacroblockType::Intra,
                true
            )
            .unwrap()
        )
    }

    #[test]
    fn short_coded_inter_block() {
        let bitstream = [0x03, 0x00, 0x14];
        let mut reader = H263Reader::from_source(&bitstream[..]);

        let picture = Picture {
            version: None,
            temporal_reference: 0,
            format: None,
            options: PictureOption::empty(),
            has_plusptype: false,
            has_opptype: false,
            picture_type: PictureTypeCode::IFrame,
            motion_vector_range: None,
            slice_submode: None,
            scalability_layer: None,
            reference_picture_selection_mode: None,
            prediction_reference: None,
            backchannel_message: None,
            reference_picture_resampling: None,
            quantizer: 1,
            multiplex_bitstream: None,
            pb_reference: None,
            pb_quantizer: None,
            extra: Vec::new(),
        };

        assert_eq!(
            Block {
                intradc: None,
                tcoef: vec![
                    TCoefficient {
                        is_short: true,
                        run: 5,
                        level: 2
                    },
                    TCoefficient {
                        is_short: true,
                        run: 0,
                        level: 3
                    }
                ]
            },
            decode_block(
                &mut reader,
                DecoderOption::empty(),
                &picture,
                PictureOption::empty(),
                MacroblockType::Inter,
                true
            )
            .unwrap()
        )
    }

    #[test]
    fn short_coded_intra_block() {
        let bitstream = [0x63, 0x03, 0x00, 0x14];
        let mut reader = H263Reader::from_source(&bitstream[..]);

        let picture = Picture {
            version: None,
            temporal_reference: 0,
            format: None,
            options: PictureOption::empty(),
            has_plusptype: false,
            has_opptype: false,
            picture_type: PictureTypeCode::IFrame,
            motion_vector_range: None,
            slice_submode: None,
            scalability_layer: None,
            reference_picture_selection_mode: None,
            prediction_reference: None,
            backchannel_message: None,
            reference_picture_resampling: None,
            quantizer: 1,
            multiplex_bitstream: None,
            pb_reference: None,
            pb_quantizer: None,
            extra: Vec::new(),
        };

        assert_eq!(
            Block {
                intradc: IntraDc::from_level(0x318),
                tcoef: vec![
                    TCoefficient {
                        is_short: true,
                        run: 5,
                        level: 2
                    },
                    TCoefficient {
                        is_short: true,
                        run: 0,
                        level: 3
                    }
                ]
            },
            decode_block(
                &mut reader,
                DecoderOption::empty(),
                &picture,
                PictureOption::empty(),
                MacroblockType::Intra,
                true
            )
            .unwrap()
        )
    }

    #[test]
    fn sorenson_long_coded_intra_block() {
        let bitstream = [0x63, 0x06, 0x06, 0x14, 0x1A, 0x61, 0x00];
        let mut reader = H263Reader::from_source(&bitstream[..]);

        let picture = Picture {
            version: Some(1),
            temporal_reference: 0,
            format: None,
            options: PictureOption::empty(),
            has_plusptype: false,
            has_opptype: false,
            picture_type: PictureTypeCode::IFrame,
            motion_vector_range: None,
            slice_submode: None,
            scalability_layer: None,
            reference_picture_selection_mode: None,
            prediction_reference: None,
            backchannel_message: None,
            reference_picture_resampling: None,
            quantizer: 1,
            multiplex_bitstream: None,
            pb_reference: None,
            pb_quantizer: None,
            extra: Vec::new(),
        };

        assert_eq!(
            Block {
                intradc: IntraDc::from_level(0x318),
                tcoef: vec![
                    TCoefficient {
                        is_short: false,
                        run: 3,
                        level: 5
                    },
                    TCoefficient {
                        is_short: false,
                        run: 12,
                        level: 16
                    }
                ]
            },
            decode_block(
                &mut reader,
                DecoderOption::SORENSON_SPARK_BITSTREAM,
                &picture,
                PictureOption::empty(),
                MacroblockType::Intra,
                true
            )
            .unwrap()
        )
    }

    #[test]
    fn sorenson_xlong_coded_intra_block() {
        let bitstream = [0x63, 0x07, 0x06, 0x01, 0x41, 0xE6, 0x01, 0x00];
        let mut reader = H263Reader::from_source(&bitstream[..]);

        let picture = Picture {
            version: Some(1),
            temporal_reference: 0,
            format: None,
            options: PictureOption::empty(),
            has_plusptype: false,
            has_opptype: false,
            picture_type: PictureTypeCode::IFrame,
            motion_vector_range: None,
            slice_submode: None,
            scalability_layer: None,
            reference_picture_selection_mode: None,
            prediction_reference: None,
            backchannel_message: None,
            reference_picture_resampling: None,
            quantizer: 1,
            multiplex_bitstream: None,
            pb_reference: None,
            pb_quantizer: None,
            extra: Vec::new(),
        };

        assert_eq!(
            Block {
                intradc: IntraDc::from_level(0x318),
                tcoef: vec![
                    TCoefficient {
                        is_short: false,
                        run: 3,
                        level: 5
                    },
                    TCoefficient {
                        is_short: false,
                        run: 12,
                        level: 16
                    }
                ]
            },
            decode_block(
                &mut reader,
                DecoderOption::SORENSON_SPARK_BITSTREAM,
                &picture,
                PictureOption::empty(),
                MacroblockType::Intra,
                true
            )
            .unwrap()
        )
    }
}
