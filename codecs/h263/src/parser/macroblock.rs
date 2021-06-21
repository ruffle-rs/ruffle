//! Macroblock decoding

#![allow(clippy::unusual_byte_groupings)]

use crate::error::{Error, Result};
use crate::parser::reader::H263Reader;
use crate::parser::vlc::{Entry, Entry::End, Entry::Fork};
use crate::types::{
    CodedBlockPattern, HalfPel, Macroblock, MacroblockType, MotionVector, Picture, PictureOption,
    PictureTypeCode,
};
use std::io::Read;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BlockPatternEntry {
    Stuffing,

    Invalid,

    Valid(MacroblockType, bool, bool),
}

const MCBPC_I_TABLE: [Entry<BlockPatternEntry>; 21] = [
    Fork(2, 1), //x, slot 0
    End(BlockPatternEntry::Valid(
        MacroblockType::Intra,
        false,
        false,
    )), //1, slot 1
    Fork(6, 3), //0x, slot 2
    Fork(4, 5), //01x, slot 3
    End(BlockPatternEntry::Valid(MacroblockType::Intra, true, false)), //010, slot 4
    End(BlockPatternEntry::Valid(MacroblockType::Intra, true, true)), //011, slot 5
    Fork(8, 7), //00x, slot 6
    End(BlockPatternEntry::Valid(MacroblockType::Intra, false, true)), //001, slot 7
    Fork(10, 9), //000x, slot 8
    End(BlockPatternEntry::Valid(
        MacroblockType::IntraQ,
        false,
        false,
    )), //0001, slot 9
    Fork(14, 11), //0000x, slot 10
    Fork(12, 13), //00001x, slot 11
    End(BlockPatternEntry::Valid(
        MacroblockType::IntraQ,
        true,
        false,
    )), //000010, slot 12
    End(BlockPatternEntry::Valid(MacroblockType::IntraQ, true, true)), //000011, slot 13
    Fork(16, 20), //00000x, slot 14
    End(BlockPatternEntry::Invalid), //slot 15
    Fork(17, 15), //000000x, slot 16
    Fork(18, 15), //0000000x, slot 17
    Fork(15, 19), //00000000x, slot 18
    End(BlockPatternEntry::Stuffing), //000000001, slot 19
    End(BlockPatternEntry::Valid(
        MacroblockType::IntraQ,
        false,
        true,
    )), //000001, slot 20
];

const MCBPC_P_TABLE: [Entry<BlockPatternEntry>; 53] = [
    Fork(2, 1), //x, slot 0
    End(BlockPatternEntry::Valid(
        MacroblockType::Inter,
        false,
        false,
    )), //1, slot 1
    Fork(6, 3), //0x, slot 2
    Fork(4, 5), //01x, slot 3
    End(BlockPatternEntry::Valid(
        MacroblockType::Inter4V,
        false,
        false,
    )), //010, slot 4
    End(BlockPatternEntry::Valid(
        MacroblockType::InterQ,
        false,
        false,
    )), //011, slot 5
    Fork(10, 7), //00x, slot 6
    Fork(8, 9), //001x, slot 7
    End(BlockPatternEntry::Valid(MacroblockType::Inter, true, false)), //0010, slot 8
    End(BlockPatternEntry::Valid(MacroblockType::Inter, false, true)), //0011, slot 9
    Fork(16, 11), //000x, slot 10
    Fork(13, 12), //0001x, slot 11
    End(BlockPatternEntry::Valid(
        MacroblockType::Intra,
        false,
        false,
    )), //00011, slot 12
    Fork(14, 15), //00010x, slot 13
    End(BlockPatternEntry::Valid(
        MacroblockType::IntraQ,
        false,
        false,
    )), //000100, slot 14
    End(BlockPatternEntry::Valid(MacroblockType::Inter, true, true)), //000101, slot 15
    Fork(24, 17), //0000x, slot 16
    Fork(18, 21), //00001x, slot 17
    Fork(19, 20), //000010x, slot 18
    End(BlockPatternEntry::Valid(
        MacroblockType::Inter4V,
        true,
        false,
    )), //0000100, slot 19
    End(BlockPatternEntry::Valid(
        MacroblockType::Inter4V,
        false,
        true,
    )), //0000101, slot 20
    Fork(22, 23), //000011x, slot 21
    End(BlockPatternEntry::Valid(
        MacroblockType::InterQ,
        true,
        false,
    )), //0000110, slot 22
    End(BlockPatternEntry::Valid(
        MacroblockType::InterQ,
        false,
        true,
    )), //0000111, slot 23
    Fork(30, 25), //00000x, slot 24
    Fork(27, 26), //000001x, slot 25
    End(BlockPatternEntry::Valid(MacroblockType::Intra, true, true)), //0000011, slot 26
    Fork(28, 29), //0000010x, slot 27
    End(BlockPatternEntry::Valid(MacroblockType::Intra, false, true)), //00000100, slot 28
    End(BlockPatternEntry::Valid(
        MacroblockType::Inter4V,
        true,
        true,
    )), //00000101, slot 29
    Fork(36, 31), //000000x, slot 30
    Fork(33, 32), //0000001x, slot 31
    End(BlockPatternEntry::Valid(MacroblockType::Intra, true, false)), //00000011, slot 32
    Fork(34, 35), //00000010x, slot 33
    End(BlockPatternEntry::Valid(
        MacroblockType::IntraQ,
        false,
        true,
    )), //000000100, slot 34
    End(BlockPatternEntry::Valid(MacroblockType::InterQ, true, true)), //000000101, slot 35
    Fork(40, 37), //0000000x, slot 36
    Fork(38, 39), //00000001x, slot 37
    End(BlockPatternEntry::Valid(MacroblockType::IntraQ, true, true)), //000000010, slot 38
    End(BlockPatternEntry::Valid(
        MacroblockType::IntraQ,
        true,
        false,
    )), //000000011, slot 39
    Fork(42, 41), //00000000x, slot 40
    End(BlockPatternEntry::Stuffing), //000000001, slot 41
    Fork(43, 44), //000000000x, slot 42
    End(BlockPatternEntry::Invalid), //slot 43: no long runs of zeroes
    Fork(45, 46), //0000000001x, slot 44
    End(BlockPatternEntry::Valid(
        MacroblockType::Inter4Vq,
        false,
        false,
    )), //00000000010, slot 45
    Fork(47, 50), //00000000011x, slot 46
    Fork(48, 49), //000000000110x, slot 47
    End(BlockPatternEntry::Valid(
        MacroblockType::Inter4Vq,
        false,
        true,
    )), //0000000001100, slot 48
    End(BlockPatternEntry::Invalid), //0000000001101, slot 49
    Fork(51, 52), //000000000111x, slot 50
    End(BlockPatternEntry::Valid(
        MacroblockType::Inter4Vq,
        true,
        false,
    )), //0000000001110, slot 51
    End(BlockPatternEntry::Valid(
        MacroblockType::Inter4Vq,
        true,
        true,
    )), //0000000001111, slot 52
];

/// The decoding table for MODB (used in PB Frames).
///
/// The output of this table is two booleans, the first indicating the presence
/// of a `CodedBlockPattern` for the B-blocks, and the second indicating the
/// presence of motion vectors for the B-blocks.
const MODB_TABLE: [Entry<(bool, bool)>; 5] = [
    Fork(1, 2),          //x, slot 0
    End((false, false)), //0, slot 1
    Fork(3, 4),          //1x, slot 2
    End((false, true)),  //10, slot 3
    End((true, true)),   //11, slot 4
];

fn decode_cbpb<R>(reader: &mut H263Reader<R>) -> Result<CodedBlockPattern>
where
    R: Read,
{
    reader.with_transaction(|reader| {
        let cbp0 = reader.read_bits::<u8>(1)? == 1;
        let cbp1 = reader.read_bits::<u8>(1)? == 1;
        let cbp2 = reader.read_bits::<u8>(1)? == 1;
        let cbp3 = reader.read_bits::<u8>(1)? == 1;
        let cbp4 = reader.read_bits::<u8>(1)? == 1;
        let cbp5 = reader.read_bits::<u8>(1)? == 1;

        Ok(CodedBlockPattern {
            codes_luma: [cbp0, cbp1, cbp2, cbp3],
            codes_chroma_b: cbp4,
            codes_chroma_r: cbp5,
        })
    })
}

/// The coding table for all four possible luma block codes.
///
/// This specifically produces the INTRA block pattern; for the INTER block
/// pattern, flip all bits in the result. A `None` indicates an invalid
/// bitstream.
const CBPY_TABLE_INTRA: [Entry<Option<[bool; 4]>>; 33] = [
    Fork(1, 24),                             //x, slot 0
    Fork(2, 17),                             //0x, slot 1
    Fork(3, 12),                             //00x, slot 2
    Fork(4, 9),                              //000x, slot 3
    Fork(5, 6),                              //0000x, slot 4
    End(None),                               //00000, slot 5, not a valid prefix
    Fork(7, 8),                              //00001x, slot 6
    End(Some([false, true, true, false])),   //000010, slot 7
    End(Some([true, false, false, true])),   //000011, slot 8
    Fork(10, 11),                            //0001, slot 9
    End(Some([true, false, false, false])),  //00010, slot 10
    End(Some([false, true, false, false])),  //00011, slot 11
    Fork(13, 16),                            //001x, slot 12
    Fork(14, 15),                            //0010x, slot 13
    End(Some([false, false, true, false])),  //00100, slot 14
    End(Some([false, false, false, true])),  //00101, slot 15
    End(Some([false, false, false, false])), //0011, slot 16
    Fork(18, 21),                            //01x, slot 17
    Fork(19, 20),                            //010x, slot 18
    End(Some([true, true, false, false])),   //0100, slot 19
    End(Some([true, false, true, false])),   //0101, slot 20
    Fork(22, 23),                            //011x, slot 21
    End(Some([true, true, true, false])),    //0110, slot 22
    End(Some([false, true, false, true])),   //0111, slot 23
    Fork(25, 32),                            //1x, slot 24
    Fork(26, 29),                            //10x, slot 25
    Fork(27, 28),                            //100x, slot 26
    End(Some([true, true, false, true])),    //1000, slot 27
    End(Some([false, false, true, true])),   //1001, slot 28
    Fork(30, 31),                            //101x, slot 29
    End(Some([true, false, true, true])),    //1010, slot 30
    End(Some([false, true, true, true])),    //1011, slot 31
    End(Some([true, true, true, true])),     //11, slot 32
];

fn decode_dquant<R>(reader: &mut H263Reader<R>) -> Result<i8>
where
    R: Read,
{
    reader.with_transaction(|reader| {
        Ok(match reader.read_bits::<u8>(2)? {
            0 => -1,
            1 => -2,
            2 => 1,
            3 => 2,
            _ => return Err(Error::InternalDecoderError),
        })
    })
}

/// The standard motion vector decode table.
///
/// This table yields `f32`s, not `HalfPel`s, due to the fact that you cannot
/// construct a `HalfPel` in a `const` in stable Rust. It should be converted
/// the moment you get this data.
const MVD_TABLE: [Entry<Option<f32>>; 130] = [
    Fork(2, 1),       //x, slot 0
    End(Some(0.0)),   //1, slot 1
    Fork(6, 3),       //0x, slot 2
    Fork(4, 5),       //01x, slot 3
    End(Some(0.5)),   //010, slot 4
    End(Some(-0.5)),  //011, slot 5
    Fork(10, 7),      //00x, slot 6
    Fork(8, 9),       //001x, slot 7
    End(Some(1.0)),   //0010, slot 8
    End(Some(-1.0)),  //0011, slot 9
    Fork(14, 11),     //000x, slot 10
    Fork(12, 13),     //0001x, slot 11
    End(Some(1.5)),   //00010, slot 12
    End(Some(-1.5)),  //00011, slot 13
    Fork(26, 15),     //0000x, slot 14
    Fork(19, 16),     //00001x, slot 15
    Fork(17, 18),     //000011x, slot 16
    End(Some(2.0)),   //0000110, slot 17
    End(Some(-2.0)),  //0000111, slot 18
    Fork(23, 20),     //000010x, slot 19
    Fork(21, 22),     //0000101x, slot 20
    End(Some(2.5)),   //00001010, slot 21
    End(Some(-2.5)),  //00001011, slot 22
    Fork(24, 25),     //0000100x, slot 23
    End(Some(3.0)),   //00001000, slot 24
    End(Some(-3.0)),  //00001001, slot 25
    Fork(50, 27),     //00000x, slot 26
    Fork(31, 28),     //000001x, slot 27
    Fork(29, 30),     //0000011x, slot 28
    End(Some(3.5)),   //00000110, slot 29
    End(Some(-3.5)),  //00000111, slot 30
    Fork(39, 32),     //0000010x, slot 31
    Fork(36, 33),     //00000101x, slot 32
    Fork(34, 35),     //000001011x, slot 33
    End(Some(4.0)),   //0000010110, slot 34
    End(Some(-4.0)),  //0000010111, slot 35
    Fork(37, 38),     //000001010x, slot 36
    End(Some(4.5)),   //0000010100, slot 37
    End(Some(-4.5)),  //0000010101, slot 38
    Fork(43, 40),     //00000100x, slot 39
    Fork(41, 42),     //000001001x, slot 40
    End(Some(5.0)),   //0000010010, slot 41
    End(Some(-5.0)),  //0000010011, slot 42
    Fork(47, 44),     //000001000x, slot 43
    Fork(45, 46),     //0000010001x, slot 44
    End(Some(5.5)),   //00000100010, slot 45
    End(Some(-5.5)),  //00000100011, slot 46
    Fork(48, 49),     //0000010000x, slot 47
    End(Some(6.0)),   //00000100000, slot 48
    End(Some(-6.0)),  //00000100001, slot 49
    Fork(82, 51),     //000000x, slot 50
    Fork(67, 52),     //0000001x, slot 51
    Fork(60, 53),     //00000011x, slot 52
    Fork(57, 54),     //000000111x, slot 53
    Fork(55, 56),     //0000001111x, slot 54
    End(Some(6.5)),   //00000011110, slot 55
    End(Some(-6.5)),  //00000011111, slot 56
    Fork(58, 59),     //0000001110x, slot 57
    End(Some(7.0)),   //00000011100, slot 58
    End(Some(-7.0)),  //00000011101, slot 59
    Fork(64, 61),     //000000110x, slot 60
    Fork(62, 63),     //0000001101x, slot 61
    End(Some(7.5)),   //00000011010, slot 62
    End(Some(-7.5)),  //00000011011, slot 63
    Fork(65, 66),     //0000001100x, slot 64
    End(Some(8.0)),   //00000011000, slot 65
    End(Some(-8.0)),  //00000011001, slot 66
    Fork(75, 68),     //00000010x, slot 67
    Fork(72, 69),     //000000101x, slot 68
    Fork(70, 71),     //0000001011x, slot 69
    End(Some(8.5)),   //00000010110, slot 70
    End(Some(-8.5)),  //00000010111, slot 71
    Fork(73, 74),     //0000001010x, slot 72
    End(Some(9.0)),   //00000010100, slot 73
    End(Some(-9.0)),  //00000010101, slot 74
    Fork(79, 76),     //000000100x, slot 75
    Fork(77, 78),     //0000001001x, slot 76
    End(Some(9.5)),   //00000010010, slot 77
    End(Some(-9.5)),  //00000010011, slot 78
    Fork(80, 81),     //0000001000x, slot 79
    End(Some(10.0)),  //00000010000, slot 80
    End(Some(-10.0)), //00000010001, slot 81
    Fork(98, 83),     //0000000x, slot 82
    Fork(91, 84),     //00000001x, slot 83
    Fork(88, 85),     //000000011x, slot 84
    Fork(86, 87),     //0000000111x, slot 85
    End(Some(10.5)),  //00000001110, slot 86
    End(Some(-10.5)), //00000001111, slot 87
    Fork(89, 90),     //0000000110x, slot 88
    End(Some(11.0)),  //00000001100, slot 89
    End(Some(-11.0)), //00000001101, slot 90
    Fork(95, 92),     //000000010x, slot 91
    Fork(93, 94),     //0000000101x, slot 92
    End(Some(11.5)),  //00000001010, slot 93
    End(Some(-11.5)), //00000001011, slot 94
    Fork(96, 97),     //0000000100x, slot 95
    End(Some(12.0)),  //00000001000, slot 96
    End(Some(-12.0)), //00000001001, slot 97
    Fork(114, 99),    //00000000x, slot 98
    Fork(107, 100),   //000000001x, slot 99
    Fork(104, 101),   //0000000011x, slot 100
    Fork(102, 103),   //00000000111x, slot 101
    End(Some(12.5)),  //000000001110, slot 102
    End(Some(-12.5)), //000000001111, slot 103
    Fork(105, 106),   //00000000110x, slot 104
    End(Some(13.0)),  //00000001100, slot 105
    End(Some(-13.0)), //00000001101, slot 106
    Fork(111, 108),   //0000000010x, slot 107
    Fork(109, 110),   //00000000101x, slot 108
    End(Some(13.5)),  //000000001010, slot 109
    End(Some(-13.5)), //000000001011, slot 110
    Fork(112, 113),   //00000000100x, slot 111
    End(Some(14.0)),  //000000001000, slot 112
    End(Some(-14.0)), //000000001001, slot 113
    Fork(122, 115),   //000000000x, slot 114
    Fork(119, 116),   //0000000001x, slot 115
    Fork(117, 118),   //00000000011x, slot 116
    End(Some(14.5)),  //000000000110, slot 117
    End(Some(-14.5)), //000000000111, slot 118
    Fork(120, 121),   //00000000010x, slot 119
    End(Some(15.0)),  //000000000100, slot 120
    End(Some(-15.0)), //000000000101, slot 121
    Fork(129, 123),   //0000000000x, slot 122
    Fork(127, 124),   //00000000001x, slot 123
    Fork(125, 126),   //000000000011x, slot 124
    End(Some(15.5)),  //0000000000110, slot 125
    End(Some(-15.5)), //0000000000111, slot 126
    Fork(129, 128),   //000000000010x, slot 127
    End(Some(-16.0)), //0000000000101, slot 128
    End(None),        //00000000000 or 0000000000100 patterns, slot 129
];

/// Decode a motion vector from the bitstream.
///
/// This currently only handles standard range motion vectors, not unrestricted
/// ones.
fn decode_motion_vector<R>(
    reader: &mut H263Reader<R>,
    picture: &Picture,
    running_options: PictureOption,
) -> Result<MotionVector>
where
    R: Read,
{
    reader.with_transaction(|reader| {
        if running_options.contains(PictureOption::UNRESTRICTED_MOTION_VECTORS)
            && picture.has_plusptype
        {
            let x = reader.read_umv()?;
            let y = reader.read_umv()?;

            Ok((x, y).into())
        } else {
            let x = HalfPel::from(reader.read_vlc(&MVD_TABLE[..])?.ok_or(Error::InvalidMvd)?);
            let y = HalfPel::from(reader.read_vlc(&MVD_TABLE[..])?.ok_or(Error::InvalidMvd)?);

            Ok((x, y).into())
        }
    })
}

/// Decode a macroblock header from the bitstream referenced by `reader`.
///
/// The `running_options` should be the set of currently in-force options
/// present on the currently-decoded picture. This is not entirely equivalent
/// to the current picture's option set as some options can carry forward from
/// picture to picture without being explicitly mentioned.
pub fn decode_macroblock<R>(
    reader: &mut H263Reader<R>,
    picture: &Picture,
    running_options: PictureOption,
) -> Result<Macroblock>
where
    R: Read,
{
    reader.with_transaction(|reader| {
        let is_coded: u8 = if matches!(picture.picture_type, PictureTypeCode::IFrame) {
            0
        } else {
            reader.read_bits(1)?
        };

        if is_coded == 0 {
            let mcbpc = match picture.picture_type {
                PictureTypeCode::IFrame => reader.read_vlc(&MCBPC_I_TABLE[..])?,
                PictureTypeCode::PFrame => reader.read_vlc(&MCBPC_P_TABLE[..])?,
                _ => return Err(Error::UnimplementedDecoding),
            };

            let (mb_type, codes_chroma_b, codes_chroma_r) = match mcbpc {
                BlockPatternEntry::Stuffing => return Ok(Macroblock::Stuffing),
                BlockPatternEntry::Invalid => return Err(Error::InvalidMacroblockHeader),
                BlockPatternEntry::Valid(mbt, chroma_b, chroma_r) => (mbt, chroma_b, chroma_r),
            };

            let (has_cbpb, has_mvdb) = if matches!(picture.picture_type, PictureTypeCode::PbFrame) {
                reader.read_vlc(&MODB_TABLE[..])?
            } else {
                (false, false)
            };

            let codes_luma = if mb_type.is_intra() {
                match reader.read_vlc(&CBPY_TABLE_INTRA)? {
                    Some(v) => v,
                    None => return Err(Error::InvalidMacroblockCodedBits),
                }
            } else {
                match reader.read_vlc(&CBPY_TABLE_INTRA)? {
                    Some([v1, v2, v3, v4]) => [!v1, !v2, !v3, !v4],
                    None => return Err(Error::InvalidMacroblockCodedBits),
                }
            };

            let coded_block_pattern_b = if has_cbpb {
                Some(decode_cbpb(reader)?)
            } else {
                None
            };

            let d_quantizer = if running_options.contains(PictureOption::MODIFIED_QUANTIZATION) {
                return Err(Error::UnimplementedDecoding);
            } else if mb_type.has_quantizer() {
                Some(decode_dquant(reader)?)
            } else {
                None
            };

            let motion_vector = if mb_type.is_inter() || picture.picture_type.is_any_pbframe() {
                Some(decode_motion_vector(reader, picture, running_options)?)
            } else {
                None
            };

            let addl_motion_vectors = if mb_type.has_fourvec() {
                let mv2 = decode_motion_vector(reader, picture, running_options)?;
                let mv3 = decode_motion_vector(reader, picture, running_options)?;
                let mv4 = decode_motion_vector(reader, picture, running_options)?;

                Some([mv2, mv3, mv4])
            } else {
                None
            };

            let motion_vectors_b = if has_mvdb {
                let mv1 = decode_motion_vector(reader, picture, running_options)?;
                let mv2 = decode_motion_vector(reader, picture, running_options)?;
                let mv3 = decode_motion_vector(reader, picture, running_options)?;
                let mv4 = decode_motion_vector(reader, picture, running_options)?;

                Some([mv1, mv2, mv3, mv4])
            } else {
                None
            };

            Ok(Macroblock::Coded {
                mb_type,
                coded_block_pattern: CodedBlockPattern {
                    codes_luma,
                    codes_chroma_b,
                    codes_chroma_r,
                },
                coded_block_pattern_b,
                d_quantizer,
                motion_vector,
                addl_motion_vectors,
                motion_vectors_b,
            })
        } else {
            Ok(Macroblock::Uncoded)
        }
    })
}

#[cfg(test)]
mod tests {
    use crate::parser::macroblock::{
        BlockPatternEntry, CBPY_TABLE_INTRA, MCBPC_I_TABLE, MCBPC_P_TABLE, MODB_TABLE, MVD_TABLE,
    };
    use crate::parser::reader::H263Reader;
    use crate::types::MacroblockType;

    #[test]
    #[allow(clippy::inconsistent_digit_grouping)]
    fn macroblock_mcbpc_iframe() {
        let bit_pattern = vec![
            0b1_001_010_0,
            0b11_0001_00,
            0b0001_0000,
            0b10_000011,
            0b00000000,
            0b1_0000001,
        ];
        let mut reader = H263Reader::from_source(&bit_pattern[..]);

        assert_eq!(
            reader.read_vlc(&MCBPC_I_TABLE).unwrap(),
            BlockPatternEntry::Valid(MacroblockType::Intra, false, false)
        );
        assert_eq!(
            reader.read_vlc(&MCBPC_I_TABLE).unwrap(),
            BlockPatternEntry::Valid(MacroblockType::Intra, false, true)
        );
        assert_eq!(
            reader.read_vlc(&MCBPC_I_TABLE).unwrap(),
            BlockPatternEntry::Valid(MacroblockType::Intra, true, false)
        );
        assert_eq!(
            reader.read_vlc(&MCBPC_I_TABLE).unwrap(),
            BlockPatternEntry::Valid(MacroblockType::Intra, true, true)
        );
        assert_eq!(
            reader.read_vlc(&MCBPC_I_TABLE).unwrap(),
            BlockPatternEntry::Valid(MacroblockType::IntraQ, false, false)
        );
        assert_eq!(
            reader.read_vlc(&MCBPC_I_TABLE).unwrap(),
            BlockPatternEntry::Valid(MacroblockType::IntraQ, false, true)
        );
        assert_eq!(
            reader.read_vlc(&MCBPC_I_TABLE).unwrap(),
            BlockPatternEntry::Valid(MacroblockType::IntraQ, true, false)
        );
        assert_eq!(
            reader.read_vlc(&MCBPC_I_TABLE).unwrap(),
            BlockPatternEntry::Valid(MacroblockType::IntraQ, true, true)
        );
        assert_eq!(
            reader.read_vlc(&MCBPC_I_TABLE).unwrap(),
            BlockPatternEntry::Stuffing
        );
        assert_eq!(
            reader.read_vlc(&MCBPC_I_TABLE).unwrap(),
            BlockPatternEntry::Invalid
        );
    }

    #[test]
    #[allow(clippy::inconsistent_digit_grouping)]
    fn macroblock_mcbpc_pframe() {
        let bit_pattern = vec![
            0b1_0011_001,
            0b0_000101_0,
            0b11_000011,
            0b1_0000110,
            0b00000010,
            0b1_010_0000,
            0b101_00001,
            0b00_000001,
            0b01_00011_0,
            0b0000100_0,
            0b0000011_0,
            0b000011_00,
            0b0100_0000,
            0b00100_000,
            0b000011_00,
            0b0000010_0,
            0b00000001,
            0b00000000,
            0b010_00000,
            0b00001100,
            0b00000000,
            0b01110_000,
            0b00000011,
            0b11_000000,
            0b00000000,
        ];
        let mut reader = H263Reader::from_source(&bit_pattern[..]);

        assert_eq!(
            reader.read_vlc(&MCBPC_P_TABLE).unwrap(),
            BlockPatternEntry::Valid(MacroblockType::Inter, false, false)
        );
        assert_eq!(
            reader.read_vlc(&MCBPC_P_TABLE).unwrap(),
            BlockPatternEntry::Valid(MacroblockType::Inter, false, true)
        );
        assert_eq!(
            reader.read_vlc(&MCBPC_P_TABLE).unwrap(),
            BlockPatternEntry::Valid(MacroblockType::Inter, true, false)
        );
        assert_eq!(
            reader.read_vlc(&MCBPC_P_TABLE).unwrap(),
            BlockPatternEntry::Valid(MacroblockType::Inter, true, true)
        );
        assert_eq!(
            reader.read_vlc(&MCBPC_P_TABLE).unwrap(),
            BlockPatternEntry::Valid(MacroblockType::InterQ, false, false)
        );
        assert_eq!(
            reader.read_vlc(&MCBPC_P_TABLE).unwrap(),
            BlockPatternEntry::Valid(MacroblockType::InterQ, false, true)
        );
        assert_eq!(
            reader.read_vlc(&MCBPC_P_TABLE).unwrap(),
            BlockPatternEntry::Valid(MacroblockType::InterQ, true, false)
        );
        assert_eq!(
            reader.read_vlc(&MCBPC_P_TABLE).unwrap(),
            BlockPatternEntry::Valid(MacroblockType::InterQ, true, true)
        );
        assert_eq!(
            reader.read_vlc(&MCBPC_P_TABLE).unwrap(),
            BlockPatternEntry::Valid(MacroblockType::Inter4V, false, false)
        );
        assert_eq!(
            reader.read_vlc(&MCBPC_P_TABLE).unwrap(),
            BlockPatternEntry::Valid(MacroblockType::Inter4V, false, true)
        );
        assert_eq!(
            reader.read_vlc(&MCBPC_P_TABLE).unwrap(),
            BlockPatternEntry::Valid(MacroblockType::Inter4V, true, false)
        );
        assert_eq!(
            reader.read_vlc(&MCBPC_P_TABLE).unwrap(),
            BlockPatternEntry::Valid(MacroblockType::Inter4V, true, true)
        );
        assert_eq!(
            reader.read_vlc(&MCBPC_P_TABLE).unwrap(),
            BlockPatternEntry::Valid(MacroblockType::Intra, false, false)
        );
        assert_eq!(
            reader.read_vlc(&MCBPC_P_TABLE).unwrap(),
            BlockPatternEntry::Valid(MacroblockType::Intra, false, true)
        );
        assert_eq!(
            reader.read_vlc(&MCBPC_P_TABLE).unwrap(),
            BlockPatternEntry::Valid(MacroblockType::Intra, true, false)
        );
        assert_eq!(
            reader.read_vlc(&MCBPC_P_TABLE).unwrap(),
            BlockPatternEntry::Valid(MacroblockType::Intra, true, true)
        );
        assert_eq!(
            reader.read_vlc(&MCBPC_P_TABLE).unwrap(),
            BlockPatternEntry::Valid(MacroblockType::IntraQ, false, false)
        );
        assert_eq!(
            reader.read_vlc(&MCBPC_P_TABLE).unwrap(),
            BlockPatternEntry::Valid(MacroblockType::IntraQ, false, true)
        );
        assert_eq!(
            reader.read_vlc(&MCBPC_P_TABLE).unwrap(),
            BlockPatternEntry::Valid(MacroblockType::IntraQ, true, false)
        );
        assert_eq!(
            reader.read_vlc(&MCBPC_P_TABLE).unwrap(),
            BlockPatternEntry::Valid(MacroblockType::IntraQ, true, true)
        );
        assert_eq!(
            reader.read_vlc(&MCBPC_P_TABLE).unwrap(),
            BlockPatternEntry::Stuffing
        );
        assert_eq!(
            reader.read_vlc(&MCBPC_P_TABLE).unwrap(),
            BlockPatternEntry::Valid(MacroblockType::Inter4Vq, false, false)
        );
        assert_eq!(
            reader.read_vlc(&MCBPC_P_TABLE).unwrap(),
            BlockPatternEntry::Valid(MacroblockType::Inter4Vq, false, true)
        );
        assert_eq!(
            reader.read_vlc(&MCBPC_P_TABLE).unwrap(),
            BlockPatternEntry::Valid(MacroblockType::Inter4Vq, true, false)
        );
        assert_eq!(
            reader.read_vlc(&MCBPC_P_TABLE).unwrap(),
            BlockPatternEntry::Valid(MacroblockType::Inter4Vq, true, true)
        );
        assert_eq!(
            reader.read_vlc(&MCBPC_P_TABLE).unwrap(),
            BlockPatternEntry::Invalid
        );
    }

    #[test]
    #[allow(clippy::inconsistent_digit_grouping)]
    fn macroblock_modb_table() {
        let bit_pattern = vec![0b0_10_11_000];
        let mut reader = H263Reader::from_source(&bit_pattern[..]);

        assert_eq!(reader.read_vlc(&MODB_TABLE).unwrap(), (false, false));
        assert_eq!(reader.read_vlc(&MODB_TABLE).unwrap(), (false, true));
        assert_eq!(reader.read_vlc(&MODB_TABLE).unwrap(), (true, true));
    }

    #[test]
    #[allow(clippy::inconsistent_digit_grouping)]
    fn macroblock_cbpy_table() {
        let bit_pattern = vec![
            0b0011_0010,
            0b1_00100_10,
            0b01_00011_0,
            0b111_00001,
            0b0_1011_000,
            0b10_000011,
            0b0101_1010,
            0b0100_1000,
            0b0110_11_00,
            0b00_0000_00,
        ];
        let mut reader = H263Reader::from_source(&bit_pattern[..]);

        assert_eq!(
            reader.read_vlc(&CBPY_TABLE_INTRA).unwrap(),
            Some([false, false, false, false])
        );
        assert_eq!(
            reader.read_vlc(&CBPY_TABLE_INTRA).unwrap(),
            Some([false, false, false, true])
        );
        assert_eq!(
            reader.read_vlc(&CBPY_TABLE_INTRA).unwrap(),
            Some([false, false, true, false])
        );
        assert_eq!(
            reader.read_vlc(&CBPY_TABLE_INTRA).unwrap(),
            Some([false, false, true, true])
        );
        assert_eq!(
            reader.read_vlc(&CBPY_TABLE_INTRA).unwrap(),
            Some([false, true, false, false])
        );
        assert_eq!(
            reader.read_vlc(&CBPY_TABLE_INTRA).unwrap(),
            Some([false, true, false, true])
        );
        assert_eq!(
            reader.read_vlc(&CBPY_TABLE_INTRA).unwrap(),
            Some([false, true, true, false])
        );
        assert_eq!(
            reader.read_vlc(&CBPY_TABLE_INTRA).unwrap(),
            Some([false, true, true, true])
        );
        assert_eq!(
            reader.read_vlc(&CBPY_TABLE_INTRA).unwrap(),
            Some([true, false, false, false])
        );
        assert_eq!(
            reader.read_vlc(&CBPY_TABLE_INTRA).unwrap(),
            Some([true, false, false, true])
        );
        assert_eq!(
            reader.read_vlc(&CBPY_TABLE_INTRA).unwrap(),
            Some([true, false, true, false])
        );
        assert_eq!(
            reader.read_vlc(&CBPY_TABLE_INTRA).unwrap(),
            Some([true, false, true, true])
        );
        assert_eq!(
            reader.read_vlc(&CBPY_TABLE_INTRA).unwrap(),
            Some([true, true, false, false])
        );
        assert_eq!(
            reader.read_vlc(&CBPY_TABLE_INTRA).unwrap(),
            Some([true, true, false, true])
        );
        assert_eq!(
            reader.read_vlc(&CBPY_TABLE_INTRA).unwrap(),
            Some([true, true, true, false])
        );
        assert_eq!(
            reader.read_vlc(&CBPY_TABLE_INTRA).unwrap(),
            Some([true, true, true, true])
        );
        assert_eq!(reader.read_vlc(&CBPY_TABLE_INTRA).unwrap(), None);
        assert_eq!(reader.read_vlc(&CBPY_TABLE_INTRA).unwrap(), None);
    }

    #[test]
    #[allow(clippy::inconsistent_digit_grouping)]
    fn macroblock_mvd_table() {
        let bit_pattern = vec![
            0b00000000,
            0b00101_000,
            0b00000001,
            0b11_000000,
            0b000101_00,
            0b00000001,
            0b11_000000,
            0b001001_00,
            0b00000010,
            0b11_000000,
            0b001101_00,
            0b00000011,
            0b11_000000,
            0b01001_000,
            0b00001011,
            0b00000001,
            0b101_00000,
            0b001111_00,
            0b00001000,
            0b1_0000001,
            0b0011_0000,
            0b0010101_0,
            0b00000101,
            0b11_000000,
            0b11001_000,
            0b00011011,
            0b00000011,
            0b101_00000,
            0b011111_00, //tail of -6.5, start of -6.0
            0b00010000,
            0b1_0000010,
            0b0011_0000,
            0b010011_00,
            0b00010101,
            0b00000101,
            0b11_000001,
            0b11_000010,
            0b01_000010,
            0b11_000011,
            0b1_00011_00,
            0b11_011_1_01,
            0b0_0010_000,
            0b10_000011,
            0b0_0000101,
            0b0_0000100,
            0b0_0000011,
            0b0_0000010,
            0b110_00000,
            0b10100_000,
            0b0010010_0,
            0b00001000,
            0b10_000001,
            0b00000_000,
            0b00011110,
            0b00000011,
            0b100_00000,
            0b011010_00,
            0b00001100,
            0b0_0000001,
            0b0110_0000,
            0b0010100_0,
            0b00000100,
            0b10_000000,
            0b10000_000,
            0b00001110,
            0b00000001,
            0b100_00000,
            0b001010_00,
            0b00000100,
            0b0_0000000,
            0b01110_000,
            0b00000110,
            0b0_0000000,
            0b01010_000,
            0b00000100,
            0b0_0000000,
            0b00110_000,
            0b00000010,
            0b0_0000000,
            0b000110_00,
            0b00000000,
            0b100_00000,
            0b00000100,
            0b00000000,
            0b00100_000,
            0b00000000,
        ];
        let mut reader = H263Reader::from_source(&bit_pattern[..]);

        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(-16.0));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(-15.5));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(-15.0));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(-14.5));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(-14.0));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(-13.5));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(-13.0));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(-12.5));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(-12.0));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(-11.5));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(-11.0));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(-10.5));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(-10.0));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(-9.5));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(-9.0));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(-8.5));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(-8.0));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(-7.5));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(-7.0));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(-6.5));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(-6.0));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(-5.5));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(-5.0));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(-4.5));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(-4.0));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(-3.5));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(-3.0));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(-2.5));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(-2.0));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(-1.5));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(-1.0));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(-0.5));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(0.0));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(0.5));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(1.0));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(1.5));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(2.0));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(2.5));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(3.0));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(3.5));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(4.0));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(4.5));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(5.0));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(5.5));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(6.0));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(6.5));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(7.0));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(7.5));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(8.0));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(8.5));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(9.0));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(9.5));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(10.0));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(10.5));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(11.0));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(11.5));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(12.0));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(12.5));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(13.0));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(13.5));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(14.0));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(14.5));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(15.0));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), Some(15.5));
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), None);
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), None);
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), None);
        assert_eq!(reader.read_vlc(&MVD_TABLE).unwrap(), None);
    }
}
