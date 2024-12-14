#![allow(clippy::unusual_byte_groupings)]

use crate::avm1::types::*;
use crate::avm2::read::tests::read_abc_from_file;
use crate::avm2::types::*;
use crate::read::tests::{read_tag_bytes_from_file, read_tag_bytes_from_file_with_index};
use crate::read::{decompress_swf, parse_swf};
use crate::string::{SwfStr, WINDOWS_1252};
use crate::tag_code::TagCode;
use crate::types::*;
use crate::write::write_swf;
use std::borrow::Cow;
use std::fs::File;

#[allow(dead_code)]
pub fn echo_swf(filename: &str) {
    let in_data = std::fs::read(filename).unwrap();
    let swf_buf = decompress_swf(&in_data[..]).unwrap();
    let swf = parse_swf(&swf_buf).unwrap();
    let out_file = File::create(filename).unwrap();
    write_swf(swf.header.swf_header(), &swf.tags, out_file).unwrap();
}

pub type TestData<T> = (u8, T, Vec<u8>);
pub type TagTestData = TestData<Tag<'static>>;
pub type Avm1TestData = TestData<Action<'static>>;
pub type Avm2TestData = TestData<AbcFile>;

pub fn tag_tests() -> Vec<TagTestData> {
    vec![
        (
            8,
            Tag::CsmTextSettings(CsmTextSettings {
                id: 2,
                use_advanced_rendering: true,
                grid_fit: TextGridFit::SubPixel,
                thickness: 1.0,
                sharpness: 2.0,
            }),
            read_tag_bytes_from_file("tests/swfs/DefineFont3-CS6.swf", TagCode::CsmTextSettings),
        ),
        (
            9, // Minimum version not listed in SWF19.
            Tag::DefineBinaryData(DefineBinaryData {
                id: 1,
                data: &[84, 101, 115, 116, 105, 110, 103, 33],
            }),
            read_tag_bytes_from_file("tests/swfs/DefineBinaryData.swf", TagCode::DefineBinaryData),
        ),
        (
            1,
            Tag::DefineBits {
                id: 1,
                jpeg_data: &[
                    255, 216, 255, 224, 0, 16, 74, 70, 73, 70, 0, 1, 1, 0, 0, 1, 0, 1, 0, 0, 255,
                    192, 0, 17, 8, 0, 5, 0, 6, 3, 1, 34, 0, 2, 17, 1, 3, 17, 1, 255, 218, 0, 12, 3,
                    1, 0, 2, 17, 3, 17, 0, 63, 0, 252, 215, 162, 138, 43, 248, 28, 255, 0, 180, 3,
                    255, 217,
                ],
            },
            read_tag_bytes_from_file(
                "tests/swfs/DefineBits-JpegTables-MX.swf",
                TagCode::DefineBits,
            ),
        ),
        (
            1,
            Tag::DefineBitsJpeg2 {
                id: 1,
                jpeg_data: &[
                    255, 216, 255, 219, 0, 67, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 255, 219, 0,
                    67, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 255, 196, 0, 31, 0, 0, 1, 5, 1, 1, 1,
                    1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 255, 196,
                    0, 181, 16, 0, 2, 1, 3, 3, 2, 4, 3, 5, 5, 4, 4, 0, 0, 1, 125, 1, 2, 3, 0, 4,
                    17, 5, 18, 33, 49, 65, 6, 19, 81, 97, 7, 34, 113, 20, 50, 129, 145, 161, 8, 35,
                    66, 177, 193, 21, 82, 209, 240, 36, 51, 98, 114, 130, 9, 10, 22, 23, 24, 25,
                    26, 37, 38, 39, 40, 41, 42, 52, 53, 54, 55, 56, 57, 58, 67, 68, 69, 70, 71, 72,
                    73, 74, 83, 84, 85, 86, 87, 88, 89, 90, 99, 100, 101, 102, 103, 104, 105, 106,
                    115, 116, 117, 118, 119, 120, 121, 122, 131, 132, 133, 134, 135, 136, 137, 138,
                    146, 147, 148, 149, 150, 151, 152, 153, 154, 162, 163, 164, 165, 166, 167, 168,
                    169, 170, 178, 179, 180, 181, 182, 183, 184, 185, 186, 194, 195, 196, 197, 198,
                    199, 200, 201, 202, 210, 211, 212, 213, 214, 215, 216, 217, 218, 225, 226, 227,
                    228, 229, 230, 231, 232, 233, 234, 241, 242, 243, 244, 245, 246, 247, 248, 249,
                    250, 255, 196, 0, 31, 1, 0, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 1,
                    2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 255, 196, 0, 181, 17, 0, 2, 1, 2, 4, 4, 3, 4,
                    7, 5, 4, 4, 0, 1, 2, 119, 0, 1, 2, 3, 17, 4, 5, 33, 49, 6, 18, 65, 81, 7, 97,
                    113, 19, 34, 50, 129, 8, 20, 66, 145, 161, 177, 193, 9, 35, 51, 82, 240, 21,
                    98, 114, 209, 10, 22, 36, 52, 225, 37, 241, 23, 24, 25, 26, 38, 39, 40, 41, 42,
                    53, 54, 55, 56, 57, 58, 67, 68, 69, 70, 71, 72, 73, 74, 83, 84, 85, 86, 87, 88,
                    89, 90, 99, 100, 101, 102, 103, 104, 105, 106, 115, 116, 117, 118, 119, 120,
                    121, 122, 130, 131, 132, 133, 134, 135, 136, 137, 138, 146, 147, 148, 149, 150,
                    151, 152, 153, 154, 162, 163, 164, 165, 166, 167, 168, 169, 170, 178, 179, 180,
                    181, 182, 183, 184, 185, 186, 194, 195, 196, 197, 198, 199, 200, 201, 202, 210,
                    211, 212, 213, 214, 215, 216, 217, 218, 226, 227, 228, 229, 230, 231, 232, 233,
                    234, 242, 243, 244, 245, 246, 247, 248, 249, 250, 255, 217, 255, 216, 255, 224,
                    0, 16, 74, 70, 73, 70, 0, 1, 1, 0, 0, 1, 0, 1, 0, 0, 255, 192, 0, 17, 8, 0, 5,
                    0, 5, 3, 1, 34, 0, 2, 17, 1, 3, 17, 1, 255, 218, 0, 12, 3, 1, 0, 2, 17, 3, 17,
                    0, 63, 0, 252, 215, 162, 138, 43, 248, 28, 255, 0, 180, 3, 255, 217,
                ],
            },
            read_tag_bytes_from_file(
                "tests/swfs/DefineBitsJpeg2-MX.swf",
                TagCode::DefineBitsJpeg2,
            ),
        ),
        (
            3,
            Tag::DefineBitsJpeg3(DefineBitsJpeg3 {
                id: 1,
                version: 3,
                deblocking: Fixed8::ZERO,
                data: &[
                    255, 216, 255, 224, 0, 16, 74, 70, 73, 70, 0, 1, 1, 0, 0, 1, 0, 1, 0, 0, 255,
                    219, 0, 67, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 255, 219, 0, 67, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 255, 192, 0, 17, 8, 0, 8, 0, 8, 3, 1, 34, 0, 2, 17,
                    1, 3, 17, 1, 255, 196, 0, 21, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 10, 255, 196, 0, 20, 16, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 255, 196, 0, 21, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 9,
                    255, 196, 0, 20, 17, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255,
                    218, 0, 12, 3, 1, 0, 2, 17, 3, 17, 0, 63, 0, 134, 240, 23, 224, 94, 255, 217,
                ],
                alpha_data: &[120, 218, 107, 104, 160, 12, 0, 0, 16, 124, 32, 1],
            }),
            read_tag_bytes_from_file("tests/swfs/DefineBitsJpeg3.swf", TagCode::DefineBitsJpeg3),
        ),
        /* TODO(Herschel): How do I get Flash to generate a DefineBitsJPEG4 tag?
        (
            10,
            Tag::DefineBitsJpeg3(DefineBitsJpeg3 {
                id: 1,
                version: 4,
                deblocking: 0.0,
                data: vec![
                    255, 216, 255, 224, 0, 16, 74, 70, 73, 70, 0, 1, 1, 0, 0, 1, 0, 1, 0,0, 255, 219,
                    0, 67, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 255, 219, 0, 67, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                    255, 192, 0, 17, 8, 0, 8, 0, 8, 3, 1, 34, 0, 2, 17, 1, 3, 17, 1, 255, 196, 0, 21,
                    0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 255, 196, 0, 20, 16, 1,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 196, 0, 21, 1, 1, 1, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 9, 255, 196, 0, 20, 17, 1, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 218, 0, 12, 3, 1, 0, 2, 17, 3, 17, 0, 63, 0, 134,
                    240, 23, 224, 94, 255, 217
                ],
                alpha_data: vec![120, 218, 107, 104, 160, 12, 0, 0, 16, 124, 32, 1],
            }),
            read_tag_bytes_from_file("tests/swfs/DefineBitsJpeg4-CC.swf", TagCode::DefineBitsJpeg4)
        ),
        */
        (
            3,
            Tag::DefineBitsLossless(DefineBitsLossless {
                version: 1,
                id: 1,
                format: BitmapFormat::Rgb32,
                width: 8,
                height: 8,
                data: Cow::Borrowed(&[
                    120, 218, 251, 207, 192, 240, 255, 255, 8, 198, 0, 4, 128, 127, 129,
                ]),
            }),
            read_tag_bytes_from_file(
                "tests/swfs/DefineBitsLossless.swf",
                TagCode::DefineBitsLossless,
            ),
        ),
        (
            3,
            Tag::DefineBitsLossless(DefineBitsLossless {
                version: 2,
                id: 1,
                format: BitmapFormat::Rgb32,
                width: 8,
                height: 8,
                data: Cow::Borrowed(&[
                    120, 218, 107, 96, 96, 168, 107, 24, 193, 24, 0, 227, 81, 63, 129,
                ]),
            }),
            read_tag_bytes_from_file(
                "tests/swfs/DefineBitsLossless2.swf",
                TagCode::DefineBitsLossless2,
            ),
        ),
        (
            1,
            Tag::DefineButton(Box::new(Button {
                id: 3,
                is_track_as_menu: false,
                records: vec![
                    ButtonRecord {
                        id: 1,
                        states: ButtonState::UP | ButtonState::OVER,
                        depth: 1,
                        matrix: Matrix::IDENTITY,
                        color_transform: ColorTransform::IDENTITY,
                        filters: vec![],
                        blend_mode: BlendMode::Normal,
                    },
                    ButtonRecord {
                        id: 2,
                        states: ButtonState::DOWN | ButtonState::HIT_TEST,
                        depth: 1,
                        matrix: Matrix::IDENTITY,
                        color_transform: ColorTransform::IDENTITY,
                        filters: vec![],
                        blend_mode: BlendMode::Normal,
                    },
                ],
                actions: vec![ButtonAction {
                    conditions: ButtonActionCondition::OVER_DOWN_TO_OVER_UP,
                    action_data: &[0],
                }],
            })),
            read_tag_bytes_from_file("tests/swfs/DefineButton-MX.swf", TagCode::DefineButton),
        ),
        (
            3,
            Tag::DefineButton2(Box::new(Button {
                id: 4,
                is_track_as_menu: true,
                records: vec![
                    ButtonRecord {
                        id: 2,
                        states: ButtonState::UP | ButtonState::OVER,
                        depth: 1,
                        matrix: Matrix::IDENTITY,
                        color_transform: ColorTransform {
                            r_add: 200,
                            g_add: 0,
                            b_add: 0,
                            a_add: 0,
                            ..Default::default()
                        },
                        filters: vec![Filter::BlurFilter(Box::new(BlurFilter {
                            blur_x: Fixed16::from_f32(5.0),
                            blur_y: Fixed16::from_f32(5.0),
                            flags: BlurFilterFlags::from_passes(1),
                        }))],
                        blend_mode: BlendMode::Difference,
                    },
                    ButtonRecord {
                        id: 3,
                        states: ButtonState::DOWN | ButtonState::HIT_TEST,
                        depth: 1,
                        matrix: Matrix::IDENTITY,
                        color_transform: ColorTransform {
                            r_multiply: 0.into(),
                            g_multiply: 1.into(),
                            b_multiply: 0.into(),
                            a_multiply: 1.into(),
                            r_add: 0,
                            g_add: 0,
                            b_add: 0,
                            a_add: 0,
                        },
                        filters: vec![],
                        blend_mode: BlendMode::Normal,
                    },
                ],
                actions: vec![
                    ButtonAction {
                        conditions: ButtonActionCondition::OVER_DOWN_TO_OVER_UP,
                        action_data: &[150, 3, 0, 0, 65, 0, 38, 0], // trace("A");
                    },
                    ButtonAction {
                        conditions: ButtonActionCondition::from_key_code(3), // Home
                        action_data: &[150, 3, 0, 0, 66, 0, 38, 0],          // trace("B");
                    },
                ],
            })),
            read_tag_bytes_from_file("tests/swfs/DefineButton2-CS6.swf", TagCode::DefineButton2),
        ),
        (
            2,
            Tag::DefineButtonColorTransform(ButtonColorTransform {
                id: 3,
                color_transforms: vec![
                    ColorTransform {
                        r_multiply: 1.into(),
                        g_multiply: 0.into(),
                        b_multiply: 0.into(),
                        a_multiply: 1.into(),
                        r_add: 1,
                        g_add: 0,
                        b_add: 0,
                        a_add: 0,
                    },
                    ColorTransform {
                        r_multiply: 0.into(),
                        g_multiply: 1.into(),
                        b_multiply: 0.into(),
                        a_multiply: 1.into(),
                        r_add: 0,
                        g_add: 1,
                        b_add: 0,
                        a_add: 0,
                    },
                    ColorTransform {
                        r_multiply: 0.into(),
                        g_multiply: 0.into(),
                        b_multiply: 1.into(),
                        a_multiply: 1.into(),
                        r_add: 0,
                        g_add: 0,
                        b_add: 1,
                        a_add: 0,
                    },
                ],
            }),
            read_tag_bytes_from_file(
                "tests/swfs/DefineButtonCxformSound-MX.swf",
                TagCode::DefineButtonCxform,
            ),
        ),
        (
            2,
            Tag::DefineButtonSound(Box::new(ButtonSounds {
                id: 3,
                up_to_over_sound: Some((
                    2,
                    SoundInfo {
                        event: SoundEvent::Event,
                        in_sample: None,
                        out_sample: None,
                        num_loops: 1,
                        envelope: None,
                    },
                )),
                over_to_down_sound: Some((
                    2,
                    SoundInfo {
                        event: SoundEvent::Start,
                        in_sample: None,
                        out_sample: None,
                        num_loops: 2,
                        envelope: None,
                    },
                )),
                down_to_over_sound: None,
                over_to_up_sound: None,
            })),
            read_tag_bytes_from_file(
                "tests/swfs/DefineButtonCxformSound-MX.swf",
                TagCode::DefineButtonSound,
            ),
        ),
        (
            4,
            Tag::DefineEditText(Box::new(
                EditText::new()
                    .with_id(2)
                    .with_font_id(1, Twips::from_pixels(18.0))
                    .with_bounds(Rectangle {
                        x_min: Twips::from_pixels(-2.0),
                        x_max: Twips::from_pixels(77.9),
                        y_min: Twips::from_pixels(-2.0),
                        y_max: Twips::from_pixels(23.9),
                    })
                    .with_color(Some(Color {
                        r: 0,
                        g: 255,
                        b: 0,
                        a: 255,
                    }))
                    .with_layout(Some(TextLayout {
                        align: TextAlign::Justify,
                        left_margin: Twips::from_pixels(3.0),
                        right_margin: Twips::from_pixels(4.0),
                        indent: Twips::from_pixels(1.0),
                        leading: Twips::from_pixels(2.0),
                    }))
                    .with_variable_name(
                        SwfStr::from_str_with_encoding("foo", WINDOWS_1252).unwrap(),
                    )
                    .with_initial_text(Some(
                        SwfStr::from_str_with_encoding("-_-", WINDOWS_1252).unwrap(),
                    ))
                    .with_is_read_only(true)
                    .with_has_border(true)
                    .with_is_multiline(true)
                    .with_use_outlines(false),
            )),
            read_tag_bytes_from_file("tests/swfs/DefineEditText-MX.swf", TagCode::DefineEditText),
        ),
        (
            1,
            Tag::DefineFont(Box::new(FontV1 {
                id: 1,
                glyphs: vec![
                    vec![
                        ShapeRecord::StyleChange(Box::new(StyleChangeData {
                            move_to: Some(Point::from_pixels(19.45, -14.0)),
                            fill_style_0: None,
                            fill_style_1: Some(1),
                            line_style: Some(0),
                            new_styles: None,
                        })),
                        ShapeRecord::StraightEdge {
                            delta: PointDelta::from_pixels(-15.6, 0.0),
                        },
                        ShapeRecord::StraightEdge {
                            delta: PointDelta::from_pixels(0.0, -4.55),
                        },
                        ShapeRecord::StraightEdge {
                            delta: PointDelta::from_pixels(15.6, 0.0),
                        },
                        ShapeRecord::StraightEdge {
                            delta: PointDelta::from_pixels(0.0, 4.55),
                        },
                    ],
                    vec![
                        ShapeRecord::StyleChange(Box::new(StyleChangeData {
                            move_to: Some(Point::from_pixels(32.65, 7.5)),
                            fill_style_0: None,
                            fill_style_1: Some(1),
                            line_style: Some(0),
                            new_styles: None,
                        })),
                        ShapeRecord::StraightEdge {
                            delta: PointDelta::from_pixels(-32.75, 0.0),
                        },
                        ShapeRecord::StraightEdge {
                            delta: PointDelta::from_pixels(0.0, -3.0),
                        },
                        ShapeRecord::StraightEdge {
                            delta: PointDelta::from_pixels(32.75, 0.0),
                        },
                        ShapeRecord::StraightEdge {
                            delta: PointDelta::from_pixels(0.0, 3.0),
                        },
                    ],
                ],
            })),
            read_tag_bytes_from_file("tests/swfs/DefineFont-MX.swf", TagCode::DefineFont),
        ),
        /* TODO: Commented out because Flash MX wrote this file with a CodeTableOffset, but we don't.
         *
        (
            3,
            Tag::DefineFont2(Box::new(Font {
                version: 2,
                id: 1,
                name: "Verdana".to_string(),
                flags: FontFlag::IS_ANSI,
                language: Language::Unknown,
                layout: None,
                glyphs: vec![],
            })),
            read_tag_bytes_from_file("tests/swfs/DefineEditText-MX.swf", TagCode::DefineFont2),
        ),
        */
        /* TODO(Herschel): Flash writes out zero rectangles with 1-bit,
         * Causing this test to fail.
        (
            6,
            Tag::DefineFont2(Box::new(Font2 {
                version: 2,
                id: 1,
                name: "Verdana\u{0}".to_string(),
                is_small_text: false,
                is_ansi: false,
                is_shift_jis: false,
                is_italic: false,
                is_bold: false,
                language: Language::Latin,
                layout: Some(Font2Layout {
                    ascent: 1030,
                    descent: 215,
                    leading: 221,
                    kerning: vec![],
                }),
                glyphs: vec![
                    Glyph2 {
                        code: 33,
                        advance: Some(403),
                        bounds: Some(Rectangle {
                            x_min: 0.0,
                            x_max: 0.0,
                            y_min: 0.0,
                            y_max: 0.0,
                        }),
                        shape_records: vec![
                            ShapeRecord::StyleChange(Box::new(StyleChangeData {
                                move_to: Some(Point::from_pixels(12.9, -37.2)),
                                fill_style_0: Some(1),
                                fill_style_1: None,
                                line_style: None,
                                new_styles: None
                            })),
                            ShapeRecord::StraightEdge { delta: PointDelta::from_pixels(-0.65, 26.95) },
                            ShapeRecord::StraightEdge { delta: PointDelta::from_pixels(-4.25, 0.0) },
                            ShapeRecord::StraightEdge { delta: PointDelta::from_pixels(-0.7, -26.95) },
                            ShapeRecord::StraightEdge { delta: PointDelta::from_pixels(5.6, 0.0) },
                            ShapeRecord::StyleChange(Box::new(StyleChangeData {
                                move_to: Some(Point::from_pixels(12.65, 0.0)),
                                fill_style_0: None,
                                fill_style_1: None,
                                line_style: None,
                                new_styles: None
                            })),
                            ShapeRecord::StraightEdge { delta: PointDelta::from_pixels(-5.1, 0.0) },
                            ShapeRecord::StraightEdge { delta: PointDelta::from_pixels(0.0, -5.25) },
                            ShapeRecord::StraightEdge { delta: PointDelta::from_pixels(5.1, 0.0) },
                            ShapeRecord::StraightEdge { delta: PointDelta::from_pixels(0.0, 5.25) },
                        ],
                    }
                ],
            })),
            read_tag_bytes_from_file("tests/swfs/DefineFont2-CS55.swf", TagCode::DefineFont2)
        ),

        (
            8,
            Tag::DefineFont2(Box::new(Font {
                version: 3,
                id: 1,
                name: "Dummy\u{0}".to_string(), // TODO(Herschel): Extra null byte?
                flags: FontFlag::empty(),
                language: Language::Latin,
                layout: Some(FontLayout {
                    ascent: 17160,
                    descent: 4180,
                    leading: 860,
                    kerning: vec![
                        KerningRecord { left_code: 65, right_code: 65, adjustment: -5000 },
                        KerningRecord { left_code: 66, right_code: 65, adjustment: -25536 },
                        KerningRecord { left_code: 65, right_code: 66, adjustment: -15000 },
                        KerningRecord { left_code: 66, right_code: 66, adjustment: -5000 },
                    ],
                }),
                glyphs: vec![
                    Glyph {
                        shape_records: vec![
                            ShapeRecord::StyleChange(Box::new(StyleChangeData {
                                move_to: Some(Point::from_pixels(205.5, -527.5)),
                                fill_style_0: Some(1),
                                fill_style_1: None,
                                line_style: None,
                                new_styles: None }
                            )),
                            ShapeRecord::StraightEdge { delta: PointDelta::from_pixels(371.0, 0.0) },
                            ShapeRecord::StraightEdge { delta: PointDelta::from_pixels(0.0, 65.0) },
                            ShapeRecord::StraightEdge { delta: PointDelta::from_pixels(-371.0, 0.0) },
                            ShapeRecord::StraightEdge { delta: PointDelta::from_pixels(0.0, -65.0) },
                        ],
                        code: 65,
                        advance: Some(15400),
                        bounds: Some(Rectangle { x_min: 0.0, x_max: 0.0, y_min: 0.0, y_max: 0.0 })
                    },
                    Glyph {
                        shape_records: vec![
                            ShapeRecord::StyleChange(Box::new(StyleChangeData {
                                move_to: Some(Point::from_pixels(249.0, -694.0)),
                                fill_style_0: Some(1),
                                fill_style_1: None,
                                line_style: None,
                                new_styles: None
                            })),
                            ShapeRecord::StraightEdge { delta: PointDelta::from_pixels(135.5, 0.0) },
                            ShapeRecord::StraightEdge { delta: PointDelta::from_pixels(0.0, 660.5) },
                            ShapeRecord::StraightEdge { delta: PointDelta::from_pixels(-135.5, 0.0) },
                            ShapeRecord::StraightEdge { delta: PointDelta::from_pixels(0.0, -660.5) },
                        ],
                        code: 66,
                        advance: Some(12200),
                        bounds: Some(Rectangle { x_min: 0.0, x_max: 0.0, y_min: 0.0, y_max: 0.0 })
                    }
                ],
            })),
            read_tag_bytes_from_file("tests/swfs/DefineFont3-CS55.swf", TagCode::DefineFont3)
        ),
        */
        /* Commented out because font name has a trailing null byte in the SWF.
         (
            11,
            Tag::DefineFont2(Box::new(Font {
                version: 3,
                id: 1,
                name: "_sans",
                flags: FontFlag::empty(),
                language: Language::Latin,
                layout: None,
                glyphs: vec![],
            })),
            read_tag_bytes_from_file(
                "tests/swfs/DefineFont3-DeviceText.swf",
                TagCode::DefineFont3,
            ),
        ),
        */
        (
            8,
            Tag::DefineFontAlignZones {
                id: 1,
                thickness: FontThickness::Thin,
                zones: vec![
                    FontAlignZone {
                        left: 13098,
                        width: 0,
                        bottom: 0,
                        height: 17102,
                    },
                    FontAlignZone {
                        left: 15333,
                        width: 0,
                        bottom: 0,
                        height: 17102,
                    },
                ],
            },
            read_tag_bytes_from_file(
                "tests/swfs/DefineFont3-CS6.swf",
                TagCode::DefineFontAlignZones,
            ),
        ),
        (
            10,
            Tag::DefineFont4(Font4 {
                id: 1,
                name: "Dummy".into(),
                is_italic: false,
                is_bold: false,
                data: None,
            }),
            read_tag_bytes_from_file("tests/swfs/DefineFont4.swf", TagCode::DefineFont4),
        ),
        (
            1,
            Tag::DefineFontInfo(Box::new(FontInfo {
                id: 1,
                version: 1,
                name: SwfStr::from_str_with_encoding("Verdana", WINDOWS_1252).unwrap(),
                flags: FontInfoFlag::IS_ANSI,
                language: Language::Unknown,
                code_table: vec![45, 95],
            })),
            read_tag_bytes_from_file("tests/swfs/DefineFont-MX.swf", TagCode::DefineFontInfo),
        ),
        (
            6,
            Tag::DefineFontInfo(Box::new(FontInfo {
                id: 1,
                version: 2,
                name: "Verdana".into(),
                flags: FontInfoFlag::HAS_WIDE_CODES
                    | FontInfoFlag::IS_BOLD
                    | FontInfoFlag::IS_ITALIC
                    | FontInfoFlag::IS_ANSI,
                language: Language::Latin,
                code_table: vec![45, 95],
            })),
            read_tag_bytes_from_file("tests/swfs/DefineText2-MX.swf", TagCode::DefineFontInfo2),
        ),
        (
            9,
            Tag::DefineFontName {
                id: 2,
                name: "Dummy".into(),
                copyright_info: "Dummy font for swf-rs tests".into(),
            },
            read_tag_bytes_from_file("tests/swfs/DefineFont4.swf", TagCode::DefineFontName),
        ),
        (
            3,
            Tag::DefineMorphShape(Box::new(DefineMorphShape {
                version: 1,
                id: 1,
                flags: DefineMorphShapeFlag::HAS_NON_SCALING_STROKES,
                start: MorphShape {
                    shape_bounds: Rectangle {
                        x_min: Twips::from_pixels(15.0),
                        x_max: Twips::from_pixels(65.0),
                        y_min: Twips::from_pixels(15.0),
                        y_max: Twips::from_pixels(65.0),
                    },
                    edge_bounds: Rectangle {
                        x_min: Twips::from_pixels(15.0),
                        x_max: Twips::from_pixels(65.0),
                        y_min: Twips::from_pixels(15.0),
                        y_max: Twips::from_pixels(65.0),
                    },
                    fill_styles: vec![FillStyle::LinearGradient(Gradient {
                        matrix: Matrix {
                            tx: Twips::from_pixels(40.0),
                            ty: Twips::from_pixels(40.0),
                            a: Fixed16::from_f32(0.024429321),
                            d: Fixed16::from_f32(0.024429321),
                            b: Fixed16::from_f32(0.024429321),
                            c: Fixed16::from_f32(-0.024429321),
                        },
                        spread: GradientSpread::Pad,
                        interpolation: GradientInterpolation::Rgb,
                        records: vec![
                            GradientRecord {
                                ratio: 0,
                                color: Color {
                                    r: 255,
                                    g: 255,
                                    b: 255,
                                    a: 255,
                                },
                            },
                            GradientRecord {
                                ratio: 255,
                                color: Color {
                                    r: 0,
                                    g: 0,
                                    b: 0,
                                    a: 255,
                                },
                            },
                        ],
                    })],
                    line_styles: vec![LineStyle::new()
                        .with_width(Twips::from_pixels(10.0))
                        .with_color(Color::from_rgba(0xff00ff00))],
                    shape: vec![
                        ShapeRecord::StyleChange(Box::new(StyleChangeData {
                            move_to: Some(Point::from_pixels(20.0, 20.0)),
                            fill_style_0: None,
                            fill_style_1: None,
                            line_style: Some(1),
                            new_styles: None,
                        })),
                        ShapeRecord::StraightEdge {
                            delta: PointDelta::from_pixels(40.0, 0.0),
                        },
                        ShapeRecord::StraightEdge {
                            delta: PointDelta::from_pixels(0.0, 40.0),
                        },
                        ShapeRecord::StraightEdge {
                            delta: PointDelta::from_pixels(-40.0, 0.0),
                        },
                        ShapeRecord::StraightEdge {
                            delta: PointDelta::from_pixels(0.0, -40.0),
                        },
                        ShapeRecord::StyleChange(Box::new(StyleChangeData {
                            move_to: None,
                            fill_style_0: Some(1),
                            fill_style_1: None,
                            line_style: None,
                            new_styles: None,
                        })),
                        ShapeRecord::StraightEdge {
                            delta: PointDelta::from_pixels(40.0, 0.0),
                        },
                        ShapeRecord::StraightEdge {
                            delta: PointDelta::from_pixels(0.0, 40.0),
                        },
                        ShapeRecord::StraightEdge {
                            delta: PointDelta::from_pixels(-40.0, 0.0),
                        },
                        ShapeRecord::StraightEdge {
                            delta: PointDelta::from_pixels(0.0, -40.0),
                        },
                    ],
                },
                end: MorphShape {
                    shape_bounds: Rectangle {
                        x_min: Twips::from_pixels(19.0),
                        x_max: Twips::from_pixels(75.05),
                        y_min: Twips::from_pixels(8.35),
                        y_max: Twips::from_pixels(61.0),
                    },
                    edge_bounds: Rectangle {
                        x_min: Twips::from_pixels(19.0),
                        x_max: Twips::from_pixels(75.05),
                        y_min: Twips::from_pixels(8.35),
                        y_max: Twips::from_pixels(61.0),
                    },
                    fill_styles: vec![FillStyle::LinearGradient(Gradient {
                        matrix: Matrix {
                            tx: Twips::from_pixels(48.4),
                            ty: Twips::from_pixels(34.65),
                            a: Fixed16::from_f32(0.0058898926),
                            d: Fixed16::from_f32(0.030914307),
                            b: Fixed16::from_f32(0.0),
                            c: Fixed16::from_f32(0.0),
                        },
                        spread: GradientSpread::Pad,
                        interpolation: GradientInterpolation::Rgb,
                        records: vec![
                            GradientRecord {
                                ratio: 56,
                                color: Color {
                                    r: 255,
                                    g: 0,
                                    b: 0,
                                    a: 255,
                                },
                            },
                            GradientRecord {
                                ratio: 157,
                                color: Color {
                                    r: 0,
                                    g: 0,
                                    b: 255,
                                    a: 255,
                                },
                            },
                        ],
                    })],
                    line_styles: vec![LineStyle::new()
                        .with_width(Twips::from_pixels(2.0))
                        .with_color(Color::from_rgba(0xffffff00))],
                    shape: vec![
                        ShapeRecord::StyleChange(Box::new(StyleChangeData {
                            move_to: Some(Point::from_pixels(20.0, 60.0)),
                            fill_style_0: None,
                            fill_style_1: None,
                            line_style: None,
                            new_styles: None,
                        })),
                        ShapeRecord::StraightEdge {
                            delta: PointDelta::from_pixels(17.4, -50.65),
                        },
                        ShapeRecord::StraightEdge {
                            delta: PointDelta::from_pixels(22.6, 10.65),
                        },
                        ShapeRecord::CurvedEdge {
                            control_delta: PointDelta::from_pixels(28.15, 19.1),
                            anchor_delta: PointDelta::from_pixels(-28.15, 20.9),
                        },
                        ShapeRecord::CurvedEdge {
                            control_delta: PointDelta::from_pixels(-19.05, -22.0),
                            anchor_delta: PointDelta::from_pixels(-20.95, 22.0),
                        },
                        ShapeRecord::StraightEdge {
                            delta: PointDelta::from_pixels(17.4, -50.65),
                        },
                        ShapeRecord::StraightEdge {
                            delta: PointDelta::from_pixels(22.6, 10.65),
                        },
                        ShapeRecord::CurvedEdge {
                            control_delta: PointDelta::from_pixels(28.15, 19.1),
                            anchor_delta: PointDelta::from_pixels(-28.15, 20.9),
                        },
                        ShapeRecord::CurvedEdge {
                            control_delta: PointDelta::from_pixels(-19.05, -22.0),
                            anchor_delta: PointDelta::from_pixels(-20.95, 22.0),
                        },
                    ],
                },
            })),
            read_tag_bytes_from_file(
                "tests/swfs/DefineMorphShape-MX.swf",
                TagCode::DefineMorphShape,
            ),
        ),
        (
            8,
            Tag::DefineMorphShape(Box::new(DefineMorphShape {
                version: 2,
                id: 1,
                flags: DefineMorphShapeFlag::HAS_SCALING_STROKES,
                start: MorphShape {
                    shape_bounds: Rectangle {
                        x_min: Twips::from_pixels(15.0),
                        x_max: Twips::from_pixels(225.0),
                        y_min: Twips::from_pixels(15.0),
                        y_max: Twips::from_pixels(225.0),
                    },
                    edge_bounds: Rectangle {
                        x_min: Twips::from_pixels(20.0),
                        x_max: Twips::from_pixels(220.0),
                        y_min: Twips::from_pixels(20.0),
                        y_max: Twips::from_pixels(220.0),
                    },
                    fill_styles: vec![FillStyle::FocalGradient {
                        gradient: Gradient {
                            matrix: Matrix {
                                tx: Twips::from_pixels(116.05),
                                ty: Twips::from_pixels(135.05),
                                a: Fixed16::from_f32(0.11468506),
                                d: Fixed16::from_f32(0.18927002),
                                b: Fixed16::ZERO,
                                c: Fixed16::ZERO,
                            },
                            spread: GradientSpread::Pad,
                            interpolation: GradientInterpolation::Rgb,
                            records: vec![
                                GradientRecord {
                                    ratio: 0,
                                    color: Color {
                                        r: 255,
                                        g: 0,
                                        b: 0,
                                        a: 255,
                                    },
                                },
                                GradientRecord {
                                    ratio: 70,
                                    color: Color {
                                        r: 255,
                                        g: 0,
                                        b: 255,
                                        a: 255,
                                    },
                                },
                                GradientRecord {
                                    ratio: 255,
                                    color: Color {
                                        r: 0,
                                        g: 0,
                                        b: 0,
                                        a: 255,
                                    },
                                },
                            ],
                        },
                        focal_point: Fixed8::from_f64(0.97265625),
                    }],
                    line_styles: vec![LineStyle::new()
                        .with_width(Twips::from_pixels(10.0))
                        .with_color(Color::from_rgba(0xff00ff00))],
                    shape: vec![
                        ShapeRecord::StyleChange(Box::new(StyleChangeData {
                            move_to: Some(Point::from_pixels(20.0, 20.0)),
                            fill_style_0: None,
                            fill_style_1: None,
                            line_style: Some(1),
                            new_styles: None,
                        })),
                        ShapeRecord::StraightEdge {
                            delta: PointDelta::from_pixels(200.0, 0.0),
                        },
                        ShapeRecord::StraightEdge {
                            delta: PointDelta::from_pixels(0.0, 200.0),
                        },
                        ShapeRecord::StraightEdge {
                            delta: PointDelta::from_pixels(-200.0, 0.0),
                        },
                        ShapeRecord::StraightEdge {
                            delta: PointDelta::from_pixels(0.0, -200.0),
                        },
                        ShapeRecord::StyleChange(Box::new(StyleChangeData {
                            move_to: None,
                            fill_style_0: Some(1),
                            fill_style_1: None,
                            line_style: None,
                            new_styles: None,
                        })),
                        ShapeRecord::StraightEdge {
                            delta: PointDelta::from_pixels(200.0, 0.0),
                        },
                        ShapeRecord::StraightEdge {
                            delta: PointDelta::from_pixels(0.0, 200.0),
                        },
                        ShapeRecord::StraightEdge {
                            delta: PointDelta::from_pixels(-200.0, 0.0),
                        },
                        ShapeRecord::StraightEdge {
                            delta: PointDelta::from_pixels(0.0, -200.0),
                        },
                    ],
                },
                end: MorphShape {
                    shape_bounds: Rectangle {
                        x_min: Twips::from_pixels(25.0),
                        x_max: Twips::from_pixels(212.05),
                        y_min: Twips::from_pixels(15.35),
                        y_max: Twips::from_pixels(148.35),
                    },
                    edge_bounds: Rectangle {
                        x_min: Twips::from_pixels(26.0),
                        x_max: Twips::from_pixels(211.05),
                        y_min: Twips::from_pixels(16.35),
                        y_max: Twips::from_pixels(147.35),
                    },
                    fill_styles: vec![FillStyle::FocalGradient {
                        gradient: Gradient {
                            matrix: Matrix {
                                tx: Twips::from_pixels(164.0),
                                ty: Twips::from_pixels(150.05),
                                a: Fixed16::from_f32(0.036087036),
                                d: Fixed16::from_f32(0.041992188),
                                b: Fixed16::from_f32(0.1347351),
                                c: Fixed16::from_f32(-0.15675354),
                            },
                            spread: GradientSpread::Pad,
                            interpolation: GradientInterpolation::Rgb,
                            records: vec![
                                GradientRecord {
                                    ratio: 0,
                                    color: Color {
                                        r: 0,
                                        g: 255,
                                        b: 255,
                                        a: 255,
                                    },
                                },
                                GradientRecord {
                                    ratio: 183,
                                    color: Color {
                                        r: 0,
                                        g: 255,
                                        b: 0,
                                        a: 255,
                                    },
                                },
                                GradientRecord {
                                    ratio: 226,
                                    color: Color {
                                        r: 255,
                                        g: 0,
                                        b: 255,
                                        a: 255,
                                    },
                                },
                            ],
                        },
                        focal_point: Fixed8::from_f64(-0.9921875),
                    }],
                    line_styles: vec![LineStyle::new()
                        .with_width(Twips::from_pixels(2.0))
                        .with_color(Color::from_rgba(0xffffff00))],
                    shape: vec![
                        ShapeRecord::StyleChange(Box::new(StyleChangeData {
                            move_to: Some(Point::from_pixels(26.0, 147.35)),
                            fill_style_0: None,
                            fill_style_1: None,
                            line_style: None,
                            new_styles: None,
                        })),
                        ShapeRecord::StraightEdge {
                            delta: PointDelta::from_pixels(95.0, -131.0),
                        },
                        ShapeRecord::StraightEdge {
                            delta: PointDelta::from_pixels(59.0, 17.0),
                        },
                        ShapeRecord::CurvedEdge {
                            control_delta: PointDelta::from_pixels(62.1, 57.0),
                            anchor_delta: PointDelta::from_pixels(-62.1, 57.0),
                        },
                        ShapeRecord::CurvedEdge {
                            control_delta: PointDelta::from_pixels(-73.2, -70.6),
                            anchor_delta: PointDelta::from_pixels(-80.8, 70.6),
                        },
                        ShapeRecord::StraightEdge {
                            delta: PointDelta::from_pixels(95.0, -131.0),
                        },
                        ShapeRecord::StraightEdge {
                            delta: PointDelta::from_pixels(59.0, 17.0),
                        },
                        ShapeRecord::CurvedEdge {
                            control_delta: PointDelta::from_pixels(62.1, 57.0),
                            anchor_delta: PointDelta::from_pixels(-62.1, 57.0),
                        },
                        ShapeRecord::CurvedEdge {
                            control_delta: PointDelta::from_pixels(-73.2, -70.6),
                            anchor_delta: PointDelta::from_pixels(-80.8, 70.6),
                        },
                    ],
                },
            })),
            read_tag_bytes_from_file(
                "tests/swfs/DefineMorphShape2.swf",
                TagCode::DefineMorphShape2,
            ),
        ),
        (
            11,
            Tag::DefineMorphShape(Box::new(DefineMorphShape {
                version: 2,
                id: 1,
                flags: DefineMorphShapeFlag::HAS_SCALING_STROKES,
                start: MorphShape {
                    shape_bounds: Rectangle {
                        x_min: Twips::from_pixels(0.0),
                        x_max: Twips::from_pixels(200.0),
                        y_min: Twips::from_pixels(0.0),
                        y_max: Twips::from_pixels(200.0),
                    },
                    edge_bounds: Rectangle {
                        x_min: Twips::from_pixels(0.0),
                        x_max: Twips::from_pixels(200.0),
                        y_min: Twips::from_pixels(0.0),
                        y_max: Twips::from_pixels(200.0),
                    },
                    fill_styles: vec![FillStyle::RadialGradient(Gradient {
                        matrix: Matrix {
                            tx: Twips::from_pixels(100.00),
                            ty: Twips::from_pixels(100.00),
                            a: Fixed16::from_f32(0.1725769),
                            d: Fixed16::from_f32(0.1725769),
                            b: Fixed16::ZERO,
                            c: Fixed16::ZERO,
                        },
                        spread: GradientSpread::Reflect,
                        interpolation: GradientInterpolation::LinearRgb,
                        records: vec![
                            GradientRecord {
                                ratio: 0,
                                color: Color {
                                    r: 255,
                                    g: 0,
                                    b: 0,
                                    a: 255,
                                },
                            },
                            GradientRecord {
                                ratio: 255,
                                color: Color {
                                    r: 0,
                                    g: 0,
                                    b: 0,
                                    a: 255,
                                },
                            },
                        ],
                    })],
                    line_styles: vec![LineStyle::new()
                        .with_width(Twips::ZERO)
                        .with_color(Color::from_rgba(0x00000000))],
                    shape: vec![
                        ShapeRecord::StyleChange(Box::new(StyleChangeData {
                            move_to: None,
                            fill_style_0: Some(1),
                            fill_style_1: None,
                            line_style: Some(1),
                            new_styles: None,
                        })),
                        ShapeRecord::StraightEdge {
                            delta: PointDelta::from_pixels(200.0, 0.0),
                        },
                        ShapeRecord::StraightEdge {
                            delta: PointDelta::from_pixels(0.0, 200.0),
                        },
                        ShapeRecord::StraightEdge {
                            delta: PointDelta::from_pixels(-200.0, 0.0),
                        },
                        ShapeRecord::StraightEdge {
                            delta: PointDelta::from_pixels(0.0, -200.0),
                        },
                    ],
                },
                end: MorphShape {
                    shape_bounds: Rectangle {
                        x_min: Twips::from_pixels(0.0),
                        x_max: Twips::from_pixels(200.0),
                        y_min: Twips::from_pixels(0.0),
                        y_max: Twips::from_pixels(200.0),
                    },
                    edge_bounds: Rectangle {
                        x_min: Twips::from_pixels(0.0),
                        x_max: Twips::from_pixels(200.0),
                        y_min: Twips::from_pixels(0.0),
                        y_max: Twips::from_pixels(200.0),
                    },
                    fill_styles: vec![FillStyle::RadialGradient(Gradient {
                        matrix: Matrix {
                            tx: Twips::from_pixels(100.00),
                            ty: Twips::from_pixels(100.00),
                            a: Fixed16::from_f32(0.000015258789),
                            d: Fixed16::from_f32(0.000015258789),
                            b: Fixed16::from_f32(0.084503174),
                            c: Fixed16::from_f32(-0.084503174),
                        },
                        spread: GradientSpread::Reflect,
                        interpolation: GradientInterpolation::LinearRgb,
                        records: vec![
                            GradientRecord {
                                ratio: 0,
                                color: Color {
                                    r: 255,
                                    g: 0,
                                    b: 0,
                                    a: 255,
                                },
                            },
                            GradientRecord {
                                ratio: 255,
                                color: Color {
                                    r: 0,
                                    g: 0,
                                    b: 0,
                                    a: 255,
                                },
                            },
                        ],
                    })],
                    line_styles: vec![LineStyle::new()
                        .with_width(Twips::ZERO)
                        .with_color(Color::from_rgba(0x00000000))],
                    shape: vec![
                        ShapeRecord::StraightEdge {
                            delta: PointDelta::from_pixels(200.0, 0.0),
                        },
                        ShapeRecord::StraightEdge {
                            delta: PointDelta::from_pixels(0.0, 200.0),
                        },
                        ShapeRecord::StraightEdge {
                            delta: PointDelta::from_pixels(-200.0, 0.0),
                        },
                        ShapeRecord::StraightEdge {
                            delta: PointDelta::from_pixels(0.0, -200.0),
                        },
                    ],
                },
            })),
            read_tag_bytes_from_file(
                "tests/swfs/DefineMorphShape2-GradientFlags.swf",
                TagCode::DefineMorphShape2,
            ),
        ),
        (
            8,
            Tag::DefineScalingGrid {
                id: 2,
                splitter_rect: Rectangle {
                    x_min: Twips::from_pixels(10.0),
                    x_max: Twips::from_pixels(40.0),
                    y_min: Twips::from_pixels(10.0),
                    y_max: Twips::from_pixels(40.0),
                },
            },
            read_tag_bytes_from_file(
                "tests/swfs/DefineScalingGrid.swf",
                TagCode::DefineScalingGrid,
            ),
        ),
        (
            9, // Minimum version not listed in SWF19.
            Tag::DefineSceneAndFrameLabelData(DefineSceneAndFrameLabelData {
                scenes: vec![
                    FrameLabelData {
                        frame_num: 0,
                        label: "Scene 1".into(),
                    },
                    FrameLabelData {
                        frame_num: 25,
                        label: "Scene2Scene2Scene2Scene2Scene2".into(),
                    },
                    FrameLabelData {
                        frame_num: 26,
                        label: "test日本語test".into(),
                    },
                ],
                frame_labels: vec![
                    FrameLabelData {
                        frame_num: 0,
                        label: "a".into(),
                    },
                    FrameLabelData {
                        frame_num: 9,
                        label: "b".into(),
                    },
                    FrameLabelData {
                        frame_num: 17,
                        label: "❤😁aaa".into(),
                    },
                    FrameLabelData {
                        frame_num: 25,
                        label: "frameInScene2".into(),
                    },
                ],
            }),
            read_tag_bytes_from_file(
                "tests/swfs/DefineSceneAndFrameLabelData.swf",
                TagCode::DefineSceneAndFrameLabelData,
            ),
        ),
        (
            1,
            Tag::DefineShape(Shape {
                version: 1,
                id: 1,
                shape_bounds: Rectangle {
                    x_min: Twips::from_pixels(0.0),
                    x_max: Twips::from_pixels(20.0),
                    y_min: Twips::from_pixels(0.0),
                    y_max: Twips::from_pixels(20.0),
                },
                edge_bounds: Rectangle {
                    x_min: Twips::from_pixels(0.0),
                    x_max: Twips::from_pixels(20.0),
                    y_min: Twips::from_pixels(0.0),
                    y_max: Twips::from_pixels(20.0),
                },
                flags: ShapeFlag::HAS_NON_SCALING_STROKES,
                styles: ShapeStyles {
                    fill_styles: vec![FillStyle::Color(Color {
                        r: 255,
                        g: 0,
                        b: 0,
                        a: 255,
                    })],
                    line_styles: vec![],
                },
                shape: vec![
                    ShapeRecord::StyleChange(Box::new(StyleChangeData {
                        move_to: None,
                        fill_style_0: None,
                        fill_style_1: Some(1),
                        line_style: None,
                        new_styles: None,
                    })),
                    ShapeRecord::StraightEdge {
                        delta: PointDelta::from_pixels(20.0, 0.0),
                    },
                    ShapeRecord::StraightEdge {
                        delta: PointDelta::from_pixels(0.0, 20.0),
                    },
                    ShapeRecord::StraightEdge {
                        delta: PointDelta::from_pixels(-20.0, 0.0),
                    },
                    ShapeRecord::StraightEdge {
                        delta: PointDelta::from_pixels(0.0, -20.0),
                    },
                ],
            }),
            read_tag_bytes_from_file("tests/swfs/DefineShape.swf", TagCode::DefineShape),
        ),
        (
            8,
            Tag::DefineShape(Shape {
                version: 3,
                id: 1,
                shape_bounds: Rectangle {
                    x_min: Twips::from_pixels(0.0),
                    x_max: Twips::from_pixels(50.0),
                    y_min: Twips::from_pixels(0.0),
                    y_max: Twips::from_pixels(50.0),
                },
                edge_bounds: Rectangle {
                    x_min: Twips::from_pixels(0.0),
                    x_max: Twips::from_pixels(50.0),
                    y_min: Twips::from_pixels(0.0),
                    y_max: Twips::from_pixels(50.0),
                },
                flags: ShapeFlag::HAS_NON_SCALING_STROKES,
                styles: ShapeStyles {
                    fill_styles: vec![FillStyle::RadialGradient(Gradient {
                        matrix: Matrix {
                            tx: Twips::from_pixels(24.95),
                            ty: Twips::from_pixels(24.95),
                            a: Fixed16::from_f32(0.030731201),
                            d: Fixed16::from_f32(0.030731201),
                            b: Fixed16::ZERO,
                            c: Fixed16::ZERO,
                        },
                        spread: GradientSpread::Pad,
                        interpolation: GradientInterpolation::Rgb,
                        records: vec![
                            GradientRecord {
                                ratio: 0,
                                color: Color {
                                    r: 255,
                                    g: 0,
                                    b: 0,
                                    a: 255,
                                },
                            },
                            GradientRecord {
                                ratio: 255,
                                color: Color {
                                    r: 0,
                                    g: 0,
                                    b: 0,
                                    a: 0,
                                },
                            },
                        ],
                    })],
                    line_styles: vec![],
                },
                shape: vec![
                    ShapeRecord::StyleChange(Box::new(StyleChangeData {
                        move_to: Some(Point::from_pixels(50.0, 25.0)),
                        fill_style_0: None,
                        fill_style_1: Some(1),
                        line_style: None,
                        new_styles: None,
                    })),
                    ShapeRecord::CurvedEdge {
                        control_delta: PointDelta::from_pixels(0.0, 10.35),
                        anchor_delta: PointDelta::from_pixels(-7.35, 7.3),
                    },
                    ShapeRecord::CurvedEdge {
                        control_delta: PointDelta::from_pixels(-7.3, 7.35),
                        anchor_delta: PointDelta::from_pixels(-10.35, 0.0),
                    },
                    ShapeRecord::CurvedEdge {
                        control_delta: PointDelta::from_pixels(-10.35, 0.0),
                        anchor_delta: PointDelta::from_pixels(-7.35, -7.35),
                    },
                    ShapeRecord::CurvedEdge {
                        control_delta: PointDelta::from_pixels(-7.3, -7.3),
                        anchor_delta: PointDelta::from_pixels(0.0, -10.35),
                    },
                    ShapeRecord::CurvedEdge {
                        control_delta: PointDelta::from_pixels(0.0, -10.35),
                        anchor_delta: PointDelta::from_pixels(7.3, -7.35),
                    },
                    ShapeRecord::CurvedEdge {
                        control_delta: PointDelta::from_pixels(7.35, -7.3),
                        anchor_delta: PointDelta::from_pixels(10.35, 0.0),
                    },
                    ShapeRecord::CurvedEdge {
                        control_delta: PointDelta::from_pixels(10.35, 0.0),
                        anchor_delta: PointDelta::from_pixels(7.3, 7.3),
                    },
                    ShapeRecord::CurvedEdge {
                        control_delta: PointDelta::from_pixels(7.35, 7.35),
                        anchor_delta: PointDelta::from_pixels(0.0, 10.35),
                    },
                ],
            }),
            read_tag_bytes_from_file("tests/swfs/DefineShape3.swf", TagCode::DefineShape3),
        ),
        (
            8,
            Tag::DefineShape(Shape {
                version: 4,
                id: 1,
                shape_bounds: Rectangle {
                    x_min: Twips::from_pixels(-10.0),
                    x_max: Twips::from_pixels(260.0),
                    y_min: Twips::from_pixels(-10.0),
                    y_max: Twips::from_pixels(110.0),
                },
                edge_bounds: Rectangle {
                    x_min: Twips::from_pixels(0.0),
                    x_max: Twips::from_pixels(250.0),
                    y_min: Twips::from_pixels(0.0),
                    y_max: Twips::from_pixels(100.0),
                },
                flags: ShapeFlag::HAS_NON_SCALING_STROKES,
                styles: ShapeStyles {
                    fill_styles: vec![
                        FillStyle::Color(Color {
                            r: 255,
                            g: 0,
                            b: 0,
                            a: 255,
                        }),
                        FillStyle::FocalGradient {
                            gradient: Gradient {
                                matrix: Matrix {
                                    tx: Twips::from_pixels(49.55),
                                    ty: Twips::from_pixels(46.55),
                                    a: Fixed16::from_f32(0.06199646),
                                    b: Fixed16::ZERO,
                                    c: Fixed16::ZERO,
                                    d: Fixed16::from_f32(0.06199646),
                                },
                                spread: GradientSpread::Pad,
                                interpolation: GradientInterpolation::LinearRgb,
                                records: vec![
                                    GradientRecord {
                                        ratio: 0,
                                        color: Color {
                                            r: 255,
                                            g: 0,
                                            b: 0,
                                            a: 255,
                                        },
                                    },
                                    GradientRecord {
                                        ratio: 255,
                                        color: Color {
                                            r: 0,
                                            g: 0,
                                            b: 0,
                                            a: 0,
                                        },
                                    },
                                ],
                            },
                            focal_point: Fixed8::from_f64(0.56640625),
                        },
                    ],
                    line_styles: vec![
                        LineStyle::new()
                            .with_width(Twips::from_pixels(20.0))
                            .with_color(Color {
                                r: 0,
                                g: 153,
                                b: 0,
                                a: 255,
                            })
                            .with_allow_scale_x(false)
                            .with_allow_scale_y(false)
                            .with_is_pixel_hinted(true)
                            .with_join_style(LineJoinStyle::Bevel)
                            .with_start_cap(LineCapStyle::None)
                            .with_end_cap(LineCapStyle::None),
                        LineStyle::new()
                            .with_width(Twips::from_pixels(20.0))
                            .with_fill_style(FillStyle::LinearGradient(Gradient {
                                matrix: Matrix {
                                    tx: Twips::from_pixels(50.0),
                                    ty: Twips::from_pixels(50.0),
                                    a: Fixed16::from_f32(0.07324219),
                                    d: Fixed16::from_f32(0.07324219),
                                    b: Fixed16::ZERO,
                                    c: Fixed16::ZERO,
                                },
                                spread: GradientSpread::Pad,
                                interpolation: GradientInterpolation::Rgb,
                                records: vec![
                                    GradientRecord {
                                        ratio: 0,
                                        color: Color {
                                            r: 255,
                                            g: 255,
                                            b: 255,
                                            a: 255,
                                        },
                                    },
                                    GradientRecord {
                                        ratio: 255,
                                        color: Color {
                                            r: 0,
                                            g: 0,
                                            b: 0,
                                            a: 255,
                                        },
                                    },
                                ],
                            }))
                            .with_allow_scale_y(false)
                            .with_is_pixel_hinted(true),
                        LineStyle::new()
                            .with_width(Twips::from_pixels(20.0))
                            .with_color(Color {
                                r: 0,
                                g: 153,
                                b: 0,
                                a: 255,
                            })
                            .with_join_style(LineJoinStyle::Miter(Fixed8::from_f32(56.0)))
                            .with_allow_scale_y(false)
                            .with_is_pixel_hinted(true),
                    ],
                },
                shape: vec![
                    ShapeRecord::StyleChange(Box::new(StyleChangeData {
                        move_to: Some(Point::from_pixels(150.0, 0.0)),
                        fill_style_0: None,
                        fill_style_1: Some(1),
                        line_style: Some(1),
                        new_styles: None,
                    })),
                    ShapeRecord::StraightEdge {
                        delta: PointDelta::from_pixels(100.0, 0.0),
                    },
                    ShapeRecord::StraightEdge {
                        delta: PointDelta::from_pixels(0.0, 100.0),
                    },
                    ShapeRecord::StyleChange(Box::new(StyleChangeData {
                        move_to: None,
                        fill_style_0: None,
                        fill_style_1: None,
                        line_style: Some(3),
                        new_styles: None,
                    })),
                    ShapeRecord::StraightEdge {
                        delta: PointDelta::from_pixels(-100.0, 0.0),
                    },
                    ShapeRecord::StraightEdge {
                        delta: PointDelta::from_pixels(0.0, -100.0),
                    },
                    ShapeRecord::StyleChange(Box::new(StyleChangeData {
                        move_to: Some(Point::from_pixels(100.0, 50.0)),
                        fill_style_0: None,
                        fill_style_1: Some(2),
                        line_style: Some(2),
                        new_styles: None,
                    })),
                    ShapeRecord::CurvedEdge {
                        control_delta: PointDelta::from_pixels(0.0, 20.65),
                        anchor_delta: PointDelta::from_pixels(-14.65, 14.6),
                    },
                    ShapeRecord::CurvedEdge {
                        control_delta: PointDelta::from_pixels(-14.7, 14.75),
                        anchor_delta: PointDelta::from_pixels(-20.65, 0.0),
                    },
                    ShapeRecord::CurvedEdge {
                        control_delta: PointDelta::from_pixels(-20.7, 0.0),
                        anchor_delta: PointDelta::from_pixels(-14.65, -14.75),
                    },
                    ShapeRecord::CurvedEdge {
                        control_delta: PointDelta::from_pixels(-14.65, -14.6),
                        anchor_delta: PointDelta::from_pixels(0.0, -20.65),
                    },
                    ShapeRecord::CurvedEdge {
                        control_delta: PointDelta::from_pixels(0.0, -20.7),
                        anchor_delta: PointDelta::from_pixels(14.65, -14.7),
                    },
                    ShapeRecord::CurvedEdge {
                        control_delta: PointDelta::from_pixels(14.65, -14.6),
                        anchor_delta: PointDelta::from_pixels(20.7, 0.0),
                    },
                    ShapeRecord::CurvedEdge {
                        control_delta: PointDelta::from_pixels(20.65, 0.0),
                        anchor_delta: PointDelta::from_pixels(14.7, 14.6),
                    },
                    ShapeRecord::CurvedEdge {
                        control_delta: PointDelta::from_pixels(14.65, 14.7),
                        anchor_delta: PointDelta::from_pixels(0.0, 20.7),
                    },
                ],
            }),
            read_tag_bytes_from_file("tests/swfs/DefineShape4.swf", TagCode::DefineShape4),
        ),
        (
            4,
            Tag::DefineSound(Box::new(Sound {
                id: 1,
                format: SoundFormat {
                    compression: AudioCompression::Uncompressed,
                    sample_rate: 44100,
                    is_16_bit: true,
                    is_stereo: false,
                },
                num_samples: 10,
                data: &[
                    255, 127, 0, 128, 255, 127, 0, 128, 255, 127, 0, 128, 255, 127, 0, 128, 255,
                    127, 0, 128,
                ],
            })),
            read_tag_bytes_from_file("tests/swfs/DefineSound.swf", TagCode::DefineSound),
        ),
        (
            3,
            Tag::DefineSprite(Sprite {
                id: 1,
                num_frames: 5,
                tags: vec![
                    Tag::ShowFrame,
                    Tag::ShowFrame,
                    Tag::ShowFrame,
                    Tag::ShowFrame,
                    Tag::ShowFrame,
                ],
            }),
            read_tag_bytes_from_file("tests/swfs/DefineSprite.swf", TagCode::DefineSprite),
        ),
        (
            1,
            Tag::DefineText(Box::new(Text {
                id: 2,
                bounds: Rectangle {
                    x_min: Twips::from_pixels(1.2),
                    x_max: Twips::from_pixels(38.65),
                    y_min: Twips::from_pixels(4.1),
                    y_max: Twips::from_pixels(18.45),
                },
                matrix: Matrix::IDENTITY,
                records: vec![TextRecord {
                    font_id: Some(1),
                    color: Some(Color {
                        r: 0,
                        g: 0,
                        b: 0,
                        a: 255,
                    }),
                    x_offset: None,
                    y_offset: Some(Twips::from_pixels(16.1)),
                    height: Some(Twips::from_pixels(16.0)),
                    glyphs: vec![
                        GlyphEntry {
                            index: 0,
                            advance: 145,
                        },
                        GlyphEntry {
                            index: 1,
                            advance: 203,
                        },
                        GlyphEntry {
                            index: 0,
                            advance: 145,
                        },
                    ],
                }],
            })),
            read_tag_bytes_from_file("tests/swfs/DefineFont-MX.swf", TagCode::DefineText),
        ),
        (
            6,
            Tag::DefineVideoStream(DefineVideoStream {
                id: 1,
                num_frames: 4,
                width: 8,
                height: 8,
                deblocking: VideoDeblocking::UseVideoPacketValue,
                is_smoothed: false,
                codec: VideoCodec::H263,
            }),
            read_tag_bytes_from_file(
                "tests/swfs/DefineVideoStream.swf",
                TagCode::DefineVideoStream,
            ),
        ),
        (
            5,
            Tag::DoAction(&[
                150, 10, 0, 0, 84, 101, 115, 116, 105, 110, 103, 33, 0, 38, 0,
            ]),
            read_tag_bytes_from_file("tests/swfs/DoAction-CS6.swf", TagCode::DoAction),
        ),
        (
            6,
            Tag::DoInitAction {
                id: 2,
                action_data: &[150, 6, 0, 0, 116, 101, 115, 116, 0, 38, 0],
            },
            read_tag_bytes_from_file("tests/swfs/DoInitAction-CS6.swf", TagCode::DoInitAction),
        ),
        (
            6,
            Tag::EnableDebugger("$1$ve$EG3LE6bumvJ2pR8F5qXny/".into()),
            read_tag_bytes_from_file(
                "tests/swfs/EnableDebugger2-CS6.swf",
                TagCode::EnableDebugger2,
            ),
        ),
        (
            10,
            Tag::EnableTelemetry { password_hash: &[] },
            read_tag_bytes_from_file("tests/swfs/EnableTelemetry.swf", TagCode::EnableTelemetry),
        ),
        (
            10,
            Tag::EnableTelemetry {
                password_hash: &[
                    207, 128, 205, 138, 237, 72, 45, 93, 21, 39, 215, 220, 114, 252, 239, 248, 78,
                    99, 38, 89, 40, 72, 68, 125, 45, 192, 176, 232, 125, 252, 154, 144,
                ],
            },
            read_tag_bytes_from_file(
                "tests/swfs/EnableTelemetry-password.swf",
                TagCode::EnableTelemetry,
            ),
        ),
        (
            6,
            Tag::ExportAssets(vec![ExportedAsset {
                id: 2,
                name: "Test💯".into(),
            }]),
            read_tag_bytes_from_file("tests/swfs/ExportAssets-CS6.swf", TagCode::ExportAssets),
        ),
        (
            8,
            Tag::FileAttributes(FileAttributes::USE_GPU | FileAttributes::IS_ACTION_SCRIPT_3),
            vec![0b01_000100, 0b00010001, 0b00101000, 0, 0, 0],
        ),
        (
            3,
            Tag::FrameLabel(FrameLabel {
                label: SwfStr::from_str_with_encoding("test", WINDOWS_1252).unwrap(),
                is_anchor: false,
            }),
            read_tag_bytes_from_file_with_index(
                "tests/swfs/FrameLabel-CS6.swf",
                TagCode::FrameLabel,
                0,
            ),
        ),
        (
            6, // Anchor tags supported in SWF version 6 and later.
            Tag::FrameLabel(FrameLabel {
                label: "anchor_tag".into(),
                is_anchor: true,
            }),
            read_tag_bytes_from_file_with_index(
                "tests/swfs/FrameLabel-CS6.swf",
                TagCode::FrameLabel,
                1,
            ),
        ),
        (
            7,
            Tag::ImportAssets {
                url: "ExportAssets-CS6.swf".into(),
                imports: vec![ExportedAsset {
                    id: 1,
                    name: "Test💯".into(),
                }],
            },
            read_tag_bytes_from_file("tests/swfs/ImportAssets-CS6.swf", TagCode::ImportAssets),
        ),
        (
            8,
            Tag::ImportAssets {
                url: "ExportAssets-CS6.swf".into(),
                imports: vec![ExportedAsset {
                    id: 1,
                    name: "Test💯".into(),
                }],
            },
            read_tag_bytes_from_file("tests/swfs/ImportAssets2-CS6.swf", TagCode::ImportAssets2),
        ),
        (
            1,
            Tag::JpegTables(&[
                255, 216, 255, 219, 0, 67, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 255, 219, 0, 67, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 255, 196, 0, 31, 0, 0, 1, 5, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0,
                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 255, 196, 0, 181, 16, 0, 2, 1, 3, 3, 2, 4, 3,
                5, 5, 4, 4, 0, 0, 1, 125, 1, 2, 3, 0, 4, 17, 5, 18, 33, 49, 65, 6, 19, 81, 97, 7,
                34, 113, 20, 50, 129, 145, 161, 8, 35, 66, 177, 193, 21, 82, 209, 240, 36, 51, 98,
                114, 130, 9, 10, 22, 23, 24, 25, 26, 37, 38, 39, 40, 41, 42, 52, 53, 54, 55, 56,
                57, 58, 67, 68, 69, 70, 71, 72, 73, 74, 83, 84, 85, 86, 87, 88, 89, 90, 99, 100,
                101, 102, 103, 104, 105, 106, 115, 116, 117, 118, 119, 120, 121, 122, 131, 132,
                133, 134, 135, 136, 137, 138, 146, 147, 148, 149, 150, 151, 152, 153, 154, 162,
                163, 164, 165, 166, 167, 168, 169, 170, 178, 179, 180, 181, 182, 183, 184, 185,
                186, 194, 195, 196, 197, 198, 199, 200, 201, 202, 210, 211, 212, 213, 214, 215,
                216, 217, 218, 225, 226, 227, 228, 229, 230, 231, 232, 233, 234, 241, 242, 243,
                244, 245, 246, 247, 248, 249, 250, 255, 196, 0, 31, 1, 0, 3, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 0, 0, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 255, 196, 0, 181, 17, 0,
                2, 1, 2, 4, 4, 3, 4, 7, 5, 4, 4, 0, 1, 2, 119, 0, 1, 2, 3, 17, 4, 5, 33, 49, 6, 18,
                65, 81, 7, 97, 113, 19, 34, 50, 129, 8, 20, 66, 145, 161, 177, 193, 9, 35, 51, 82,
                240, 21, 98, 114, 209, 10, 22, 36, 52, 225, 37, 241, 23, 24, 25, 26, 38, 39, 40,
                41, 42, 53, 54, 55, 56, 57, 58, 67, 68, 69, 70, 71, 72, 73, 74, 83, 84, 85, 86, 87,
                88, 89, 90, 99, 100, 101, 102, 103, 104, 105, 106, 115, 116, 117, 118, 119, 120,
                121, 122, 130, 131, 132, 133, 134, 135, 136, 137, 138, 146, 147, 148, 149, 150,
                151, 152, 153, 154, 162, 163, 164, 165, 166, 167, 168, 169, 170, 178, 179, 180,
                181, 182, 183, 184, 185, 186, 194, 195, 196, 197, 198, 199, 200, 201, 202, 210,
                211, 212, 213, 214, 215, 216, 217, 218, 226, 227, 228, 229, 230, 231, 232, 233,
                234, 242, 243, 244, 245, 246, 247, 248, 249, 250, 255, 217,
            ]),
            read_tag_bytes_from_file(
                "tests/swfs/DefineBits-JpegTables-MX.swf",
                TagCode::JpegTables,
            ),
        ),
        (
            1,
            Tag::Metadata(SwfStr::from_str_with_encoding("aa!", WINDOWS_1252).unwrap()),
            vec![0b01_000100, 0b000_10011, b'a', b'a', b'!', 0],
        ),
        (
            4,
            Tag::PlaceObject(Box::new(PlaceObject {
                version: 2,
                action: PlaceObjectAction::Place(1),
                depth: 1,
                matrix: Some(Matrix::IDENTITY),
                color_transform: None,
                ratio: None,
                name: None,
                clip_depth: None,
                class_name: None,
                filters: None,
                background_color: None,
                blend_mode: None,
                clip_actions: None,
                has_image: false,
                is_bitmap_cached: None,
                is_visible: None,
                amf_data: None,
            })),
            read_tag_bytes_from_file("tests/swfs/DefineShape.swf", TagCode::PlaceObject2),
        ),
        (
            5, // Specifically test for SWFv5 ClipActions.
            Tag::PlaceObject(Box::new(PlaceObject {
                version: 2,
                action: PlaceObjectAction::Place(2),
                depth: 1,
                matrix: Some(Matrix::IDENTITY),
                color_transform: None,
                ratio: None,
                name: None,
                clip_depth: None,
                class_name: None,
                filters: None,
                background_color: None,
                blend_mode: None,
                clip_actions: Some(vec![ClipAction {
                    events: ClipEventFlag::ENTER_FRAME,
                    key_code: None,
                    action_data: &[150, 6, 0, 0, 99, 108, 105, 112, 0, 38, 0],
                }]),
                has_image: false,
                is_bitmap_cached: None,
                is_visible: None,
                amf_data: None,
            })),
            read_tag_bytes_from_file(
                "tests/swfs/PlaceObject2-ClipActionsV5-CS6.swf",
                TagCode::PlaceObject2,
            ),
        ),
        (
            6, // ClipActions added in SWF version 5-6.
            Tag::PlaceObject(Box::new(PlaceObject {
                version: 2,
                action: PlaceObjectAction::Place(2),
                depth: 1,
                matrix: Some(Matrix::IDENTITY),
                color_transform: None,
                ratio: None,
                name: None,
                clip_depth: None,
                class_name: None,
                filters: None,
                background_color: None,
                blend_mode: None,
                clip_actions: Some(vec![
                    ClipAction {
                        events: ClipEventFlag::PRESS | ClipEventFlag::RELEASE,
                        key_code: None,
                        action_data: &[150, 3, 0, 0, 65, 0, 38, 0],
                    },
                    ClipAction {
                        events: ClipEventFlag::KEY_PRESS,
                        key_code: Some(99),
                        action_data: &[150, 3, 0, 0, 66, 0, 38, 0],
                    },
                    ClipAction {
                        events: ClipEventFlag::ENTER_FRAME,
                        key_code: None,
                        action_data: &[150, 3, 0, 0, 67, 0, 38, 0],
                    },
                ]),
                has_image: false,
                is_bitmap_cached: None,
                is_visible: None,
                amf_data: None,
            })),
            read_tag_bytes_from_file(
                "tests/swfs/PlaceObject2-ClipActions-CS6.swf",
                TagCode::PlaceObject2,
            ),
        ),
        (
            11,
            Tag::PlaceObject(Box::new(PlaceObject {
                version: 3,
                action: PlaceObjectAction::Place(1),
                depth: 1,
                matrix: Some(Matrix {
                    tx: Twips::from_pixels(0.0),
                    ty: Twips::from_pixels(0.0),
                    a: Fixed16::ONE,
                    b: Fixed16::ZERO,
                    c: Fixed16::ZERO,
                    d: Fixed16::ONE,
                }),
                color_transform: None,
                ratio: None,
                name: None,
                clip_depth: None,
                class_name: None,
                filters: None,
                background_color: None,
                blend_mode: None,
                clip_actions: None,
                has_image: true,
                is_bitmap_cached: None,
                is_visible: None,
                amf_data: None,
            })),
            read_tag_bytes_from_file("tests/swfs/PlaceObject3-Image.swf", TagCode::PlaceObject3),
        ),
        (
            8,
            Tag::PlaceObject(Box::new(PlaceObject {
                version: 3,
                action: PlaceObjectAction::Place(2),
                depth: 1,
                matrix: Some(Matrix {
                    tx: Twips::from_pixels(10.0),
                    ty: Twips::from_pixels(10.0),
                    a: Fixed16::from_f32(2.0),
                    b: Fixed16::ZERO,
                    c: Fixed16::ZERO,
                    d: Fixed16::from_f32(2.0),
                }),
                color_transform: Some(ColorTransform {
                    a_multiply: Fixed8::from_f32(1.0),
                    a_add: 80,
                    r_multiply: Fixed8::from_f32(0.5),
                    r_add: 60,
                    g_multiply: Fixed8::from_f32(0.25),
                    g_add: 40,
                    b_multiply: Fixed8::from_f32(0.75),
                    b_add: 20,
                }),
                ratio: None,
                name: Some("test".into()),
                clip_depth: None,
                class_name: None,
                filters: Some(vec![
                    Filter::GradientBevelFilter(Box::new(GradientFilter {
                        colors: vec![
                            GradientRecord {
                                ratio: 0,
                                color: Color {
                                    r: 255,
                                    g: 0,
                                    b: 0,
                                    a: 255,
                                },
                            },
                            GradientRecord {
                                ratio: 128,
                                color: Color {
                                    r: 0,
                                    g: 255,
                                    b: 0,
                                    a: 0,
                                },
                            },
                            GradientRecord {
                                ratio: 255,
                                color: Color {
                                    r: 0,
                                    g: 0,
                                    b: 255,
                                    a: 0,
                                },
                            },
                        ],
                        blur_x: Fixed16::from_f32(5.0),
                        blur_y: Fixed16::from_f32(5.0),
                        angle: Fixed16::from_f64(0.7853851318359375),
                        distance: Fixed16::from_f32(5.0),
                        strength: Fixed8::ONE,
                        flags: GradientFilterFlags::INNER_SHADOW
                            | GradientFilterFlags::KNOCKOUT
                            | GradientFilterFlags::COMPOSITE_SOURCE
                            | GradientFilterFlags::from_passes(3),
                    })),
                    Filter::GradientGlowFilter(Box::new(GradientFilter {
                        colors: vec![
                            GradientRecord {
                                ratio: 0,
                                color: Color {
                                    r: 255,
                                    g: 255,
                                    b: 255,
                                    a: 0,
                                },
                            },
                            GradientRecord {
                                ratio: 255,
                                color: Color {
                                    r: 0,
                                    g: 0,
                                    b: 0,
                                    a: 255,
                                },
                            },
                        ],
                        blur_x: Fixed16::from_f32(30.0),
                        blur_y: Fixed16::from_f32(30.0),
                        angle: Fixed16::from_f64(0.174530029296875),
                        distance: Fixed16::from_f32(5.0),
                        strength: Fixed8::from_f64(0.19921875),
                        flags: GradientFilterFlags::COMPOSITE_SOURCE
                            | GradientFilterFlags::ON_TOP
                            | GradientFilterFlags::from_passes(1),
                    })),
                    Filter::BlurFilter(Box::new(BlurFilter {
                        blur_x: Fixed16::from_f32(30.0),
                        blur_y: Fixed16::from_f32(20.0),
                        flags: BlurFilterFlags::from_passes(2),
                    })),
                ]),
                background_color: Some(Color {
                    r: 255,
                    g: 0,
                    b: 0,
                    a: 255,
                }),
                blend_mode: Some(BlendMode::Difference),
                clip_actions: Some(vec![
                    ClipAction {
                        events: ClipEventFlag::RELEASE_OUTSIDE | ClipEventFlag::ROLL_OVER,
                        key_code: None,
                        action_data: &[0],
                    },
                    ClipAction {
                        events: ClipEventFlag::DATA,
                        key_code: None,
                        action_data: &[150, 3, 0, 0, 66, 0, 38, 0],
                    },
                ]),
                has_image: false,
                is_bitmap_cached: Some(true),
                is_visible: Some(false),
                amf_data: None,
            })),
            read_tag_bytes_from_file(
                "tests/swfs/PlaceObject3-theworks.swf",
                TagCode::PlaceObject3,
            ),
        ),
        // Undocumented PlaceObject4 tag.
        (
            19,
            Tag::PlaceObject(Box::new(PlaceObject {
                version: 4,
                action: PlaceObjectAction::Place(2),
                depth: 1,
                matrix: Some(Matrix {
                    tx: Twips::from_pixels(10.0),
                    ty: Twips::from_pixels(10.0),
                    b: Fixed16::ZERO,
                    c: Fixed16::ZERO,
                    a: Fixed16::ONE,
                    d: Fixed16::ONE,
                }),
                color_transform: None,
                ratio: None,
                name: None,
                clip_depth: None,
                class_name: None,
                filters: None,
                background_color: None,
                blend_mode: None,
                clip_actions: None,
                has_image: false,
                is_bitmap_cached: None,
                is_visible: None,
                amf_data: Some(&[
                    10, 11, 1, 9, 116, 101, 115, 116, 6, 17, 84, 101, 115, 116, 105, 110, 103, 33,
                    1,
                ]),
            })),
            read_tag_bytes_from_file("tests/swfs/PlaceObject4.swf", TagCode::PlaceObject4),
        ),
        (
            2,
            Tag::Protect(None),
            read_tag_bytes_from_file("tests/swfs/ProtectNoPassword.swf", TagCode::Protect),
        ),
        (
            5, // Password supported in SWF version 5 or later.
            Tag::Protect(Some(
                SwfStr::from_str_with_encoding("$1$d/$yMscKH17OJ0paJT.e67iz0", WINDOWS_1252)
                    .unwrap(),
            )),
            read_tag_bytes_from_file("tests/swfs/Protect.swf", TagCode::Protect),
        ),
        (
            1,
            Tag::SetBackgroundColor(Color {
                r: 64,
                g: 150,
                b: 255,
                a: 255,
            }),
            vec![0b01_000011, 0b00000010, 64, 150, 255],
        ),
        (
            7,
            Tag::SetTabIndex {
                depth: 2,
                tab_index: 1,
            },
            vec![0b10_000100, 0b000_10000, 2, 0, 1, 0],
        ),
        (
            7,
            Tag::ScriptLimits {
                max_recursion_depth: 256,
                timeout_in_seconds: 42,
            },
            read_tag_bytes_from_file("tests/swfs/ScriptLimits.swf", TagCode::ScriptLimits),
        ),
        (1, Tag::ShowFrame, vec![0b01_000000, 0]),
        (
            3,
            Tag::SoundStreamHead2(Box::new(SoundStreamHead {
                stream_format: SoundFormat {
                    compression: AudioCompression::Uncompressed,
                    sample_rate: 5512,
                    is_16_bit: true,
                    is_stereo: false,
                },
                playback_format: SoundFormat {
                    compression: AudioCompression::UncompressedUnknownEndian,
                    sample_rate: 5512,
                    is_16_bit: true,
                    is_stereo: false,
                },
                num_samples_per_block: 229,
                latency_seek: 0,
            })),
            read_tag_bytes_from_file("tests/swfs/SoundStreamHead2.swf", TagCode::SoundStreamHead2),
        ),
        (
            9,
            Tag::SymbolClass(vec![
                SymbolClassLink {
                    id: 2,
                    class_name: "foo.Test".into(),
                },
                SymbolClassLink {
                    id: 0,
                    class_name: "DocumentTest".into(),
                },
            ]),
            read_tag_bytes_from_file("tests/swfs/SymbolClass.swf", TagCode::SymbolClass),
        ),
        (
            4,
            Tag::StartSound(StartSound {
                id: 1,
                sound_info: Box::new(SoundInfo {
                    event: SoundEvent::Start,
                    in_sample: None,
                    out_sample: None,
                    num_loops: 3,
                    envelope: None,
                }),
            }),
            read_tag_bytes_from_file("tests/swfs/DefineSound.swf", TagCode::StartSound),
        ),
        (
            9,
            Tag::StartSound2 {
                class_name: "TestSound".into(),
                sound_info: Box::new(SoundInfo {
                    event: SoundEvent::Event,
                    in_sample: None,
                    out_sample: None,
                    num_loops: 1,
                    envelope: Some(vec![SoundEnvelopePoint {
                        sample: 0,
                        left_volume: 0.0,
                        right_volume: 1.0,
                    }]),
                }),
            },
            read_tag_bytes_from_file("tests/swfs/StartSound2.swf", TagCode::StartSound2),
        ),
        (
            6,
            Tag::VideoFrame(VideoFrame {
                stream_id: 1,
                frame_num: 0,
                data: &[0, 0, 132, 0, 4, 4, 17, 38, 190, 190, 190, 190, 201, 182],
            }),
            read_tag_bytes_from_file("tests/swfs/DefineVideoStream.swf", TagCode::VideoFrame),
        ),
        (
            1,
            Tag::Unknown {
                tag_code: 512,
                data: &[],
            },
            vec![0b00_000000, 0b10000000],
        ),
        (
            1,
            Tag::Unknown {
                tag_code: 513,
                data: &[1, 2],
            },
            vec![0b01_000010, 0b10000000, 1, 2],
        ),
        (
            1,
            Tag::Unknown {
                tag_code: 513,
                data: &[0; 64],
            },
            vec![
                0b01_111111,
                0b10000000,
                64,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        ),
        (
            8,
            Tag::DefineShape(Shape {
                version: 4,
                id: 2,
                shape_bounds: Rectangle {
                    x_min: Twips::from_pixels(-20.0),
                    x_max: Twips::from_pixels(220.0),
                    y_min: Twips::from_pixels(-20.0),
                    y_max: Twips::from_pixels(20.0),
                },
                edge_bounds: Rectangle {
                    x_min: Twips::from_pixels(0.0),
                    x_max: Twips::from_pixels(200.0),
                    y_min: Twips::from_pixels(0.0),
                    y_max: Twips::from_pixels(0.0),
                },
                flags: ShapeFlag::HAS_SCALING_STROKES,
                styles: ShapeStyles {
                    fill_styles: vec![],
                    line_styles: vec![LineStyle::new()
                        .with_width(Twips::from_pixels(40.0))
                        .with_fill_style(FillStyle::Bitmap {
                            id: 1,
                            matrix: Matrix {
                                a: Fixed16::from_f32(20.0),
                                d: Fixed16::from_f32(20.0),
                                tx: Twips::from_pixels(10.0),
                                ty: Twips::from_pixels(10.0),
                                ..Default::default()
                            },
                            is_smoothed: false,
                            is_repeating: true,
                        })],
                },
                shape: vec![
                    ShapeRecord::StyleChange(Box::new(StyleChangeData {
                        move_to: None,
                        fill_style_0: None,
                        fill_style_1: None,
                        line_style: Some(1),
                        new_styles: None,
                    })),
                    ShapeRecord::StraightEdge {
                        delta: PointDelta::from_pixels(200.0, 0.0),
                    },
                ],
            }),
            read_tag_bytes_from_file("tests/swfs/BitmapLineStyle.swf", TagCode::DefineShape4),
        ),
    ]
}

pub fn avm1_tests() -> Vec<Avm1TestData> {
    vec![
        (4, Action::Add, vec![0x0A]),
        (4, Action::AsciiToChar, vec![0x33]),
        (4, Action::Call, vec![0x9E, 0, 0]),
        (4, Action::CharToAscii, vec![0x32]),
        (4, Action::Divide, vec![0x0D]),
        (4, Action::Equals, vec![0x0E]),
        (4, Action::GetTime, vec![0x34]),
        (
            3,
            Action::GetUrl(GetUrl {
                url: SwfStr::from_str_with_encoding("a", WINDOWS_1252).unwrap(),
                target: SwfStr::from_str_with_encoding("b", WINDOWS_1252).unwrap(),
            }),
            vec![0x83, 4, 0, 97, 0, 98, 0],
        ),
        (
            4,
            Action::GetUrl2(GetUrl2::for_load_movie(SendVarsMethod::Post)),
            vec![0x9A, 1, 0, 0b01_0000_10],
        ),
        (
            4,
            Action::GetUrl2(GetUrl2::for_load_movie(SendVarsMethod::None)),
            vec![0x9A, 1, 0, 0b01_0000_00],
        ),
        (4, Action::GetVariable, vec![0x1C]),
        (
            3,
            Action::GotoFrame(GotoFrame { frame: 11 }),
            vec![0x81, 2, 0, 11, 0],
        ),
        (
            4,
            Action::GotoFrame2(GotoFrame2 {
                set_playing: false,
                scene_offset: 0,
            }),
            vec![0x9F, 1, 0, 0],
        ),
        (
            4,
            Action::GotoFrame2(GotoFrame2 {
                set_playing: true,
                scene_offset: 259,
            }),
            vec![0x9F, 3, 0, 0b11, 3, 1],
        ),
        (
            3,
            Action::GotoLabel(GotoLabel {
                label: SwfStr::from_str_with_encoding("testb", WINDOWS_1252).unwrap(),
            }),
            vec![0x8C, 6, 0, 116, 101, 115, 116, 98, 0],
        ),
        (4, Action::If(If { offset: 1 }), vec![0x9D, 2, 0, 1, 0]),
        (4, Action::Jump(Jump { offset: 1 }), vec![0x99, 2, 0, 1, 0]),
        (4, Action::Less, vec![0x0F]),
        (4, Action::MBAsciiToChar, vec![0x37]),
        (4, Action::MBCharToAscii, vec![0x36]),
        (4, Action::MBStringExtract, vec![0x35]),
        (4, Action::MBStringLength, vec![0x31]),
        (4, Action::Multiply, vec![0x0C]),
        (3, Action::NextFrame, vec![0x04]),
        (4, Action::And, vec![0x10]),
        (4, Action::Not, vec![0x12]),
        (4, Action::Or, vec![0x11]),
        (3, Action::Play, vec![0x06]),
        (4, Action::Pop, vec![0x17]),
        (3, Action::PreviousFrame, vec![0x05]),
        (
            4,
            Action::Push(Push {
                values: vec![Value::Str(
                    SwfStr::from_str_with_encoding("test", WINDOWS_1252).unwrap(),
                )],
            }),
            vec![0x96, 6, 0, 0, 116, 101, 115, 116, 0],
        ),
        (
            4,
            Action::Push(Push {
                values: vec![Value::Float(0.0)],
            }),
            vec![0x96, 5, 0, 1, 0, 0, 0, 0],
        ),
        (
            5,
            Action::Push(Push {
                values: vec![Value::Double(1.5)],
            }),
            vec![0x96, 9, 0, 6, 0, 0, 248, 63, 0, 0, 0, 0],
        ),
        (
            5,
            Action::Push(Push {
                values: vec![Value::Null],
            }),
            vec![0x96, 1, 0, 2],
        ),
        (
            5,
            Action::Push(Push {
                values: vec![Value::Undefined],
            }),
            vec![0x96, 1, 0, 3],
        ),
        (
            5,
            Action::Push(Push {
                values: vec![Value::Null, Value::Undefined],
            }),
            vec![0x96, 2, 0, 2, 3],
        ),
        (
            5,
            Action::Push(Push {
                values: vec![Value::Register(1)],
            }),
            vec![0x96, 2, 0, 4, 1],
        ),
        (
            5,
            Action::Push(Push {
                values: vec![Value::Bool(false)],
            }),
            vec![0x96, 2, 0, 5, 0],
        ),
        (
            5,
            Action::Push(Push {
                values: vec![Value::Bool(true)],
            }),
            vec![0x96, 2, 0, 5, 1],
        ),
        (
            5,
            Action::Push(Push {
                values: vec![Value::Double(0.0)],
            }),
            vec![0x96, 9, 0, 6, 0, 0, 0, 0, 0, 0, 0, 0],
        ),
        (
            5,
            Action::Push(Push {
                values: vec![Value::Int(31)],
            }),
            vec![0x96, 5, 0, 7, 31, 0, 0, 0],
        ),
        (
            5,
            Action::Push(Push {
                values: vec![Value::Int(-50)],
            }),
            vec![0x96, 5, 0, 7, 206, 255, 255, 255],
        ),
        (
            5,
            Action::Push(Push {
                values: vec![Value::ConstantPool(77)],
            }),
            vec![0x96, 2, 0, 8, 77],
        ),
        (
            5,
            Action::Push(Push {
                values: vec![Value::ConstantPool(257)],
            }),
            vec![0x96, 3, 0, 9, 1, 1],
        ),
        (4, Action::RandomNumber, vec![0x30]),
        (
            3,
            Action::SetTarget(SetTarget {
                target: SwfStr::from_str_with_encoding("test", WINDOWS_1252).unwrap(),
            }),
            vec![0x8B, 5, 0, 116, 101, 115, 116, 0],
        ),
        (4, Action::SetVariable, vec![0x1D]),
        (3, Action::Stop, vec![0x07]),
        (3, Action::StopSounds, vec![0x09]),
        (4, Action::StringAdd, vec![0x21]),
        (4, Action::StringEquals, vec![0x13]),
        (4, Action::StringExtract, vec![0x15]),
        (4, Action::StringLength, vec![0x14]),
        (4, Action::StringLess, vec![0x29]),
        (4, Action::Subtract, vec![0x0B]),
        (3, Action::ToggleQuality, vec![0x08]),
        (4, Action::ToInteger, vec![0x18]),
        (4, Action::Trace, vec![0x26]),
        (
            3,
            Action::WaitForFrame(WaitForFrame {
                frame: 4,
                num_actions_to_skip: 10,
            }),
            vec![0x8A, 3, 0, 4, 0, 10],
        ),
        (
            4,
            Action::WaitForFrame2(WaitForFrame2 {
                num_actions_to_skip: 34,
            }),
            vec![0x8D, 1, 0, 34],
        ),
        (
            1,
            Action::Unknown(Unknown {
                opcode: 0x79,
                data: &[],
            }),
            vec![0x79],
        ),
        (
            1,
            Action::Unknown(Unknown {
                opcode: 0xA0,
                data: &[2, 3],
            }),
            vec![0xA0, 2, 0, 2, 3],
        ),
        (
            5,
            Action::DefineFunction(DefineFunction {
                name: SwfStr::from_str_with_encoding("cliche", WINDOWS_1252).unwrap(),
                params: vec![
                    SwfStr::from_str_with_encoding("greeting", WINDOWS_1252).unwrap(),
                    SwfStr::from_str_with_encoding("name", WINDOWS_1252).unwrap(),
                ],
                actions: &[
                    0x96, 0x0a, 0x00, 0x00, 0x67, 0x72, 0x65, 0x65, 0x74, 0x69, 0x6e, 0x67, 0x00,
                    0x1c, 0x96, 0x03, 0x00, 0x00, 0x20, 0x00, 0x47, 0x96, 0x06, 0x00, 0x00, 0x6e,
                    0x61, 0x6d, 0x65, 0x00, 0x1c, 0x47, 0x3e,
                ],
            }),
            vec![
                0x9b, 0x19, 0x00, 0x63, 0x6c, 0x69, 0x63, 0x68, 0x65, 0x00, 0x02, 0x00, 0x67, 0x72,
                0x65, 0x65, 0x74, 0x69, 0x6e, 0x67, 0x00, 0x6e, 0x61, 0x6d, 0x65, 0x00, 0x21, 0x00,
                0x96, 0x0a, 0x00, 0x00, 0x67, 0x72, 0x65, 0x65, 0x74, 0x69, 0x6e, 0x67, 0x00, 0x1c,
                0x96, 0x03, 0x00, 0x00, 0x20, 0x00, 0x47, 0x96, 0x06, 0x00, 0x00, 0x6e, 0x61, 0x6d,
                0x65, 0x00, 0x1c, 0x47, 0x3e,
            ],
        ),
    ]
}

pub fn avm2_tests() -> Vec<Avm2TestData> {
    vec![(
        10,
        AbcFile {
            major_version: 46,
            minor_version: 16,
            constant_pool: crate::avm2::types::ConstantPool {
                ints: vec![],
                uints: vec![],
                doubles: vec![],
                strings: vec![
                    "".to_string().into_bytes(),
                    "void".to_string().into_bytes(),
                    "Avm2Test".to_string().into_bytes(),
                    "trace".to_string().into_bytes(),
                    "Test".to_string().into_bytes(),
                ],
                namespaces: vec![Namespace::Package(Index::new(1))],
                namespace_sets: vec![],
                multinames: vec![
                    Multiname::QName {
                        namespace: Index::new(1),
                        name: Index::new(2),
                    },
                    Multiname::QName {
                        namespace: Index::new(1),
                        name: Index::new(3),
                    },
                    Multiname::QName {
                        namespace: Index::new(1),
                        name: Index::new(4),
                    },
                ],
            },
            methods: vec![
                Method {
                    name: Index::new(0),
                    params: vec![],
                    return_type: Index::new(1),
                    flags: MethodFlags::empty(),
                    body: Some(Index::new(0)),
                },
                Method {
                    name: Index::new(0),
                    params: vec![],
                    return_type: Index::new(0),
                    flags: MethodFlags::empty(),
                    body: Some(Index::new(1)),
                },
            ],
            metadata: vec![],
            instances: vec![],
            classes: vec![],
            scripts: vec![Script {
                init_method: Index::new(1),
                traits: vec![Trait {
                    name: Index::new(2),
                    kind: TraitKind::Method {
                        disp_id: 1,
                        method: Index::new(0),
                    },
                    metadata: vec![],
                    is_final: false,
                    is_override: false,
                }],
            }],
            method_bodies: vec![
                MethodBody {
                    method: Index::new(0),
                    max_stack: 2,
                    num_locals: 1,
                    init_scope_depth: 1,
                    max_scope_depth: 2,
                    code: vec![208, 48, 93, 3, 44, 5, 79, 3, 1, 71],
                    exceptions: vec![],
                    traits: vec![],
                },
                MethodBody {
                    method: Index::new(1),
                    max_stack: 1,
                    num_locals: 2,
                    init_scope_depth: 1,
                    max_scope_depth: 2,
                    code: vec![208, 48, 93, 2, 70, 2, 0, 130, 213, 209, 72],
                    exceptions: vec![],
                    traits: vec![],
                },
            ],
        },
        read_abc_from_file("tests/swfs/Avm2Dummy.swf"),
    )]
}
