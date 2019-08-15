#![allow(clippy::inconsistent_digit_grouping, clippy::unreadable_literal)]

use crate::avm1::types::*;
use crate::avm2::read::tests::read_abc_from_file;
use crate::avm2::types::*;
use crate::read::read_swf;
use crate::read::tests::{read_tag_bytes_from_file, read_tag_bytes_from_file_with_index};
use crate::tag_code::TagCode;
use crate::types::*;
use crate::write::write_swf;
use std::fs::File;
use std::vec::Vec;

#[allow(dead_code)]
pub fn echo_swf(filename: &str) {
    let in_file = File::open(filename).unwrap();
    let swf = read_swf(in_file).unwrap();
    let out_file = File::create(filename).unwrap();
    write_swf(&swf, out_file).unwrap();
}

pub type TestData<T> = (u8, T, Vec<u8>);
pub type TagTestData = TestData<Tag>;
pub type Avm1TestData = TestData<Action>;
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
            Tag::DefineBinaryData {
                id: 1,
                data: vec![84, 101, 115, 116, 105, 110, 103, 33],
            },
            read_tag_bytes_from_file("tests/swfs/DefineBinaryData.swf", TagCode::DefineBinaryData),
        ),
        (
            1,
            Tag::DefineBits {
                id: 1,
                jpeg_data: vec![
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
                jpeg_data: vec![
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
                deblocking: 0.0,
                data: vec![
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
                alpha_data: vec![120, 218, 107, 104, 160, 12, 0, 0, 16, 124, 32, 1],
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
                num_colors: 0,
                data: vec![
                    120, 218, 251, 207, 192, 240, 255, 255, 8, 198, 0, 4, 128, 127, 129,
                ],
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
                num_colors: 0,
                data: vec![
                    120, 218, 107, 96, 96, 168, 107, 24, 193, 24, 0, 227, 81, 63, 129,
                ],
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
                        states: vec![ButtonState::Up, ButtonState::Over]
                            .into_iter()
                            .collect(),
                        depth: 1,
                        matrix: Matrix::new(),
                        color_transform: ColorTransform::new(),
                        filters: vec![],
                        blend_mode: BlendMode::Normal,
                    },
                    ButtonRecord {
                        id: 2,
                        states: vec![ButtonState::Down, ButtonState::HitTest]
                            .into_iter()
                            .collect(),
                        depth: 1,
                        matrix: Matrix::new(),
                        color_transform: ColorTransform::new(),
                        filters: vec![],
                        blend_mode: BlendMode::Normal,
                    },
                ],
                actions: vec![ButtonAction {
                    conditions: vec![ButtonActionCondition::OverDownToOverUp]
                        .into_iter()
                        .collect(),
                    key_code: None,
                    action_data: vec![0],
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
                        states: vec![ButtonState::Up, ButtonState::Over]
                            .into_iter()
                            .collect(),
                        depth: 1,
                        matrix: Matrix::new(),
                        color_transform: ColorTransform {
                            r_multiply: 1f32,
                            g_multiply: 1f32,
                            b_multiply: 1f32,
                            a_multiply: 1f32,
                            r_add: 200,
                            g_add: 0,
                            b_add: 0,
                            a_add: 0,
                        },
                        filters: vec![Filter::BlurFilter(Box::new(BlurFilter {
                            blur_x: 5f64,
                            blur_y: 5f64,
                            num_passes: 1,
                        }))],
                        blend_mode: BlendMode::Difference,
                    },
                    ButtonRecord {
                        id: 3,
                        states: vec![ButtonState::Down, ButtonState::HitTest]
                            .into_iter()
                            .collect(),
                        depth: 1,
                        matrix: Matrix::new(),
                        color_transform: ColorTransform {
                            r_multiply: 0f32,
                            g_multiply: 1f32,
                            b_multiply: 0f32,
                            a_multiply: 1f32,
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
                        conditions: vec![ButtonActionCondition::OverDownToOverUp]
                            .into_iter()
                            .collect(),
                        key_code: None,
                        action_data: vec![150, 3, 0, 0, 65, 0, 38, 0], // trace("A");
                    },
                    ButtonAction {
                        conditions: vec![ButtonActionCondition::KeyPress].into_iter().collect(),
                        key_code: Some(3),                             // Home
                        action_data: vec![150, 3, 0, 0, 66, 0, 38, 0], // trace("B");
                    },
                ],
            })),
            read_tag_bytes_from_file("tests/swfs/DefineButton2-CS6.swf", TagCode::DefineButton2),
        ),
        (
            2,
            Tag::DefineButtonColorTransform {
                id: 3,
                color_transforms: vec![
                    ColorTransform {
                        r_multiply: 1f32,
                        g_multiply: 0f32,
                        b_multiply: 0f32,
                        a_multiply: 1f32,
                        r_add: 1,
                        g_add: 0,
                        b_add: 0,
                        a_add: 0,
                    },
                    ColorTransform {
                        r_multiply: 0f32,
                        g_multiply: 1f32,
                        b_multiply: 0f32,
                        a_multiply: 1f32,
                        r_add: 0,
                        g_add: 1,
                        b_add: 0,
                        a_add: 0,
                    },
                    ColorTransform {
                        r_multiply: 0f32,
                        g_multiply: 0f32,
                        b_multiply: 1f32,
                        a_multiply: 1f32,
                        r_add: 0,
                        g_add: 0,
                        b_add: 1,
                        a_add: 0,
                    },
                ],
            },
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
            Tag::DefineEditText(Box::new(EditText {
                id: 2,
                bounds: Rectangle {
                    x_min: Twips::from_pixels(-2.0),
                    x_max: Twips::from_pixels(77.9),
                    y_min: Twips::from_pixels(-2.0),
                    y_max: Twips::from_pixels(23.9),
                },
                font_id: Some(1),
                font_class_name: None,
                height: Some(360),
                color: Some(Color {
                    r: 0,
                    g: 255,
                    b: 0,
                    a: 255,
                }),
                max_length: None,
                layout: Some(TextLayout {
                    align: TextAlign::Justify,
                    left_margin: Twips::from_pixels(3.0),
                    right_margin: Twips::from_pixels(4.0),
                    indent: Twips::from_pixels(1.0),
                    leading: Twips::from_pixels(2.0),
                }),
                variable_name: "foo".to_string(),
                initial_text: Some("-_-".to_string()),
                is_word_wrap: false,
                is_multiline: true,
                is_password: false,
                is_read_only: true,
                is_auto_size: false,
                is_selectable: true,
                has_border: true,
                was_static: false,
                is_html: false,
                is_device_font: true,
            })),
            read_tag_bytes_from_file("tests/swfs/DefineEditText-MX.swf", TagCode::DefineEditText),
        ),
        (
            1,
            Tag::DefineFont(Box::new(FontV1 {
                id: 1,
                glyphs: vec![
                    vec![
                        ShapeRecord::StyleChange(StyleChangeData {
                            move_to: Some((Twips::from_pixels(19.45), Twips::from_pixels(-14.0))),
                            fill_style_0: None,
                            fill_style_1: Some(1),
                            line_style: Some(0),
                            new_styles: None,
                        }),
                        ShapeRecord::StraightEdge {
                            delta_x: Twips::from_pixels(-15.6),
                            delta_y: Twips::from_pixels(0.0),
                        },
                        ShapeRecord::StraightEdge {
                            delta_x: Twips::from_pixels(0.0),
                            delta_y: Twips::from_pixels(-4.55),
                        },
                        ShapeRecord::StraightEdge {
                            delta_x: Twips::from_pixels(15.6),
                            delta_y: Twips::from_pixels(0.0),
                        },
                        ShapeRecord::StraightEdge {
                            delta_x: Twips::from_pixels(0.0),
                            delta_y: Twips::from_pixels(4.55),
                        },
                    ],
                    vec![
                        ShapeRecord::StyleChange(StyleChangeData {
                            move_to: Some((Twips::from_pixels(32.65), Twips::from_pixels(7.5))),
                            fill_style_0: None,
                            fill_style_1: Some(1),
                            line_style: Some(0),
                            new_styles: None,
                        }),
                        ShapeRecord::StraightEdge {
                            delta_x: Twips::from_pixels(-32.75),
                            delta_y: Twips::from_pixels(0.0),
                        },
                        ShapeRecord::StraightEdge {
                            delta_x: Twips::from_pixels(0.0),
                            delta_y: Twips::from_pixels(-3.0),
                        },
                        ShapeRecord::StraightEdge {
                            delta_x: Twips::from_pixels(32.75),
                            delta_y: Twips::from_pixels(0.0),
                        },
                        ShapeRecord::StraightEdge {
                            delta_x: Twips::from_pixels(0.0),
                            delta_y: Twips::from_pixels(3.0),
                        },
                    ],
                ],
            })),
            read_tag_bytes_from_file("tests/swfs/DefineFont-MX.swf", TagCode::DefineFont),
        ),
        (
            3,
            Tag::DefineFont2(Box::new(Font {
                version: 2,
                id: 1,
                name: "Verdana".to_string(),
                is_small_text: false,
                is_ansi: true,
                is_shift_jis: false,
                is_italic: false,
                is_bold: false,
                language: Language::Unknown,
                layout: None,
                glyphs: vec![],
            })),
            read_tag_bytes_from_file("tests/swfs/DefineEditText-MX.swf", TagCode::DefineFont2),
        ),
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
                            ShapeRecord::StyleChange(StyleChangeData {
                                move_to: Some((12.9, -37.2)),
                                fill_style_0: Some(1),
                                fill_style_1: None,
                                line_style: None,
                                new_styles: None
                            }),
                            ShapeRecord::StraightEdge { delta_x: -0.65, delta_y: 26.95 },
                            ShapeRecord::StraightEdge { delta_x: -4.25, delta_y: 0.0 },
                            ShapeRecord::StraightEdge { delta_x: -0.7, delta_y: -26.95 },
                            ShapeRecord::StraightEdge { delta_x: 5.6, delta_y: 0.0 },
                            ShapeRecord::StyleChange(StyleChangeData {
                                move_to: Some((12.65, 0.0)),
                                fill_style_0: None,
                                fill_style_1: None,
                                line_style: None,
                                new_styles: None
                            }),
                            ShapeRecord::StraightEdge { delta_x: -5.1, delta_y: 0.0 },
                            ShapeRecord::StraightEdge { delta_x: 0.0, delta_y: -5.25 },
                            ShapeRecord::StraightEdge { delta_x: 5.1, delta_y: 0.0 },
                            ShapeRecord::StraightEdge { delta_x: 0.0, delta_y: 5.25 }
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
                is_small_text: false,
                is_ansi: false,
                is_shift_jis: false,
                is_italic: false,
                is_bold: false,
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
                            ShapeRecord::StyleChange(StyleChangeData {
                                move_to: Some((205.5, -527.5)),
                                fill_style_0: Some(1),
                                fill_style_1: None,
                                line_style: None,
                                new_styles: None }
                            ),
                            ShapeRecord::StraightEdge { delta_x: 371.0, delta_y: 0.0 },
                            ShapeRecord::StraightEdge { delta_x: 0.0, delta_y: 65.0 },
                            ShapeRecord::StraightEdge { delta_x: -371.0, delta_y: 0.0 },
                            ShapeRecord::StraightEdge { delta_x: 0.0, delta_y: -65.0 }
                        ],
                        code: 65,
                        advance: Some(15400),
                        bounds: Some(Rectangle { x_min: 0.0, x_max: 0.0, y_min: 0.0, y_max: 0.0 })
                    },
                    Glyph {
                        shape_records: vec![
                            ShapeRecord::StyleChange(StyleChangeData {
                                move_to: Some((249.0, -694.0)),
                                fill_style_0: Some(1),
                                fill_style_1: None,
                                line_style: None,
                                new_styles: None
                            }),
                            ShapeRecord::StraightEdge { delta_x: 135.5, delta_y: 0.0 },
                            ShapeRecord::StraightEdge { delta_x: 0.0, delta_y: 660.5 },
                            ShapeRecord::StraightEdge { delta_x: -135.5, delta_y: 0.0 },
                            ShapeRecord::StraightEdge { delta_x: 0.0, delta_y: -660.5 }
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
                name: "Dummy".to_string(),
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
                name: "Verdana".to_string(),
                is_small_text: false,
                is_ansi: true,
                is_shift_jis: false,
                is_italic: false,
                is_bold: false,
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
                name: "Verdana".to_string(),
                is_small_text: false,
                is_ansi: true,
                is_shift_jis: false,
                is_italic: true,
                is_bold: true,
                language: Language::Latin,
                code_table: vec![45, 95],
            })),
            read_tag_bytes_from_file("tests/swfs/DefineText2-MX.swf", TagCode::DefineFontInfo2),
        ),
        (
            9,
            Tag::DefineFontName {
                id: 2,
                name: "Dummy".to_string(),
                copyright_info: "Dummy font for swf-rs tests".to_string(),
            },
            read_tag_bytes_from_file("tests/swfs/DefineFont4.swf", TagCode::DefineFontName),
        ),
        (
            3,
            Tag::DefineMorphShape(Box::new(DefineMorphShape {
                version: 1,
                id: 1,
                has_non_scaling_strokes: true,
                has_scaling_strokes: false,
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
                            translate_x: Twips::from_pixels(40.0),
                            translate_y: Twips::from_pixels(40.0),
                            scale_x: 0.024429321,
                            scale_y: 0.024429321,
                            rotate_skew_0: 0.024429321,
                            rotate_skew_1: -0.024429321,
                        },
                        spread: GradientSpread::Pad,
                        interpolation: GradientInterpolation::RGB,
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
                    line_styles: vec![LineStyle::new_v1(
                        Twips::from_pixels(10.0),
                        Color {
                            r: 0,
                            g: 255,
                            b: 0,
                            a: 255,
                        },
                    )],
                    shape: vec![
                        ShapeRecord::StyleChange(StyleChangeData {
                            move_to: Some((Twips::from_pixels(20.0), Twips::from_pixels(20.0))),
                            fill_style_0: None,
                            fill_style_1: None,
                            line_style: Some(1),
                            new_styles: None,
                        }),
                        ShapeRecord::StraightEdge {
                            delta_x: Twips::from_pixels(40.0),
                            delta_y: Twips::from_pixels(0.0),
                        },
                        ShapeRecord::StraightEdge {
                            delta_x: Twips::from_pixels(0.0),
                            delta_y: Twips::from_pixels(40.0),
                        },
                        ShapeRecord::StraightEdge {
                            delta_x: Twips::from_pixels(-40.0),
                            delta_y: Twips::from_pixels(0.0),
                        },
                        ShapeRecord::StraightEdge {
                            delta_x: Twips::from_pixels(0.0),
                            delta_y: Twips::from_pixels(-40.0),
                        },
                        ShapeRecord::StyleChange(StyleChangeData {
                            move_to: None,
                            fill_style_0: Some(1),
                            fill_style_1: None,
                            line_style: None,
                            new_styles: None,
                        }),
                        ShapeRecord::StraightEdge {
                            delta_x: Twips::from_pixels(40.0),
                            delta_y: Twips::from_pixels(0.0),
                        },
                        ShapeRecord::StraightEdge {
                            delta_x: Twips::from_pixels(0.0),
                            delta_y: Twips::from_pixels(40.0),
                        },
                        ShapeRecord::StraightEdge {
                            delta_x: Twips::from_pixels(-40.0),
                            delta_y: Twips::from_pixels(0.0),
                        },
                        ShapeRecord::StraightEdge {
                            delta_x: Twips::from_pixels(0.0),
                            delta_y: Twips::from_pixels(-40.0),
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
                            translate_x: Twips::from_pixels(48.4),
                            translate_y: Twips::from_pixels(34.65),
                            scale_x: 0.0058898926,
                            scale_y: 0.030914307,
                            rotate_skew_0: 0.0,
                            rotate_skew_1: 0.0,
                        },
                        spread: GradientSpread::Pad,
                        interpolation: GradientInterpolation::RGB,
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
                    line_styles: vec![LineStyle::new_v1(
                        Twips::from_pixels(2.0),
                        Color {
                            r: 255,
                            g: 255,
                            b: 0,
                            a: 255,
                        },
                    )],
                    shape: vec![
                        ShapeRecord::StyleChange(StyleChangeData {
                            move_to: Some((Twips::from_pixels(20.0), Twips::from_pixels(60.0))),
                            fill_style_0: None,
                            fill_style_1: None,
                            line_style: None,
                            new_styles: None,
                        }),
                        ShapeRecord::StraightEdge {
                            delta_x: Twips::from_pixels(17.4),
                            delta_y: Twips::from_pixels(-50.65),
                        },
                        ShapeRecord::StraightEdge {
                            delta_x: Twips::from_pixels(22.6),
                            delta_y: Twips::from_pixels(10.65),
                        },
                        ShapeRecord::CurvedEdge {
                            control_delta_x: Twips::from_pixels(28.15),
                            control_delta_y: Twips::from_pixels(19.1),
                            anchor_delta_x: Twips::from_pixels(-28.15),
                            anchor_delta_y: Twips::from_pixels(20.9),
                        },
                        ShapeRecord::CurvedEdge {
                            control_delta_x: Twips::from_pixels(-19.05),
                            control_delta_y: Twips::from_pixels(-22.0),
                            anchor_delta_x: Twips::from_pixels(-20.95),
                            anchor_delta_y: Twips::from_pixels(22.0),
                        },
                        ShapeRecord::StraightEdge {
                            delta_x: Twips::from_pixels(17.4),
                            delta_y: Twips::from_pixels(-50.65),
                        },
                        ShapeRecord::StraightEdge {
                            delta_x: Twips::from_pixels(22.6),
                            delta_y: Twips::from_pixels(10.65),
                        },
                        ShapeRecord::CurvedEdge {
                            control_delta_x: Twips::from_pixels(28.15),
                            control_delta_y: Twips::from_pixels(19.1),
                            anchor_delta_x: Twips::from_pixels(-28.15),
                            anchor_delta_y: Twips::from_pixels(20.9),
                        },
                        ShapeRecord::CurvedEdge {
                            control_delta_x: Twips::from_pixels(-19.05),
                            control_delta_y: Twips::from_pixels(-22.0),
                            anchor_delta_x: Twips::from_pixels(-20.95),
                            anchor_delta_y: Twips::from_pixels(22.0),
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
                has_non_scaling_strokes: false,
                has_scaling_strokes: true,
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
                                translate_x: Twips::from_pixels(116.05),
                                translate_y: Twips::from_pixels(135.05),
                                scale_x: 0.11468506,
                                scale_y: 0.18927002,
                                rotate_skew_0: 0.0,
                                rotate_skew_1: 0.0,
                            },
                            spread: GradientSpread::Pad,
                            interpolation: GradientInterpolation::RGB,
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
                        focal_point: 0.97265625,
                    }],
                    line_styles: vec![LineStyle {
                        width: Twips::from_pixels(10.0),
                        color: Color {
                            r: 0,
                            g: 255,
                            b: 0,
                            a: 255,
                        },
                        start_cap: LineCapStyle::Round,
                        end_cap: LineCapStyle::Round,
                        join_style: LineJoinStyle::Round,
                        fill_style: None,
                        allow_scale_x: true,
                        allow_scale_y: true,
                        is_pixel_hinted: false,
                        allow_close: true,
                    }],
                    shape: vec![
                        ShapeRecord::StyleChange(StyleChangeData {
                            move_to: Some((Twips::from_pixels(20.0), Twips::from_pixels(20.0))),
                            fill_style_0: None,
                            fill_style_1: None,
                            line_style: Some(1),
                            new_styles: None,
                        }),
                        ShapeRecord::StraightEdge {
                            delta_x: Twips::from_pixels(200.0),
                            delta_y: Twips::from_pixels(0.0),
                        },
                        ShapeRecord::StraightEdge {
                            delta_x: Twips::from_pixels(0.0),
                            delta_y: Twips::from_pixels(200.0),
                        },
                        ShapeRecord::StraightEdge {
                            delta_x: Twips::from_pixels(-200.0),
                            delta_y: Twips::from_pixels(0.0),
                        },
                        ShapeRecord::StraightEdge {
                            delta_x: Twips::from_pixels(0.0),
                            delta_y: Twips::from_pixels(-200.0),
                        },
                        ShapeRecord::StyleChange(StyleChangeData {
                            move_to: None,
                            fill_style_0: Some(1),
                            fill_style_1: None,
                            line_style: None,
                            new_styles: None,
                        }),
                        ShapeRecord::StraightEdge {
                            delta_x: Twips::from_pixels(200.0),
                            delta_y: Twips::from_pixels(0.0),
                        },
                        ShapeRecord::StraightEdge {
                            delta_x: Twips::from_pixels(0.0),
                            delta_y: Twips::from_pixels(200.0),
                        },
                        ShapeRecord::StraightEdge {
                            delta_x: Twips::from_pixels(-200.0),
                            delta_y: Twips::from_pixels(0.0),
                        },
                        ShapeRecord::StraightEdge {
                            delta_x: Twips::from_pixels(0.0),
                            delta_y: Twips::from_pixels(-200.0),
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
                                translate_x: Twips::from_pixels(164.0),
                                translate_y: Twips::from_pixels(150.05),
                                scale_x: 0.036087036,
                                scale_y: 0.041992188,
                                rotate_skew_0: 0.1347351,
                                rotate_skew_1: -0.15675354,
                            },
                            spread: GradientSpread::Pad,
                            interpolation: GradientInterpolation::RGB,
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
                        focal_point: -0.9921875,
                    }],
                    line_styles: vec![LineStyle {
                        width: Twips::from_pixels(2.0),
                        color: Color {
                            r: 255,
                            g: 255,
                            b: 0,
                            a: 255,
                        },
                        start_cap: LineCapStyle::Round,
                        end_cap: LineCapStyle::Round,
                        join_style: LineJoinStyle::Round,
                        fill_style: None,
                        allow_scale_x: true,
                        allow_scale_y: true,
                        is_pixel_hinted: false,
                        allow_close: true,
                    }],
                    shape: vec![
                        ShapeRecord::StyleChange(StyleChangeData {
                            move_to: Some((Twips::from_pixels(26.0), Twips::from_pixels(147.35))),
                            fill_style_0: None,
                            fill_style_1: None,
                            line_style: None,
                            new_styles: None,
                        }),
                        ShapeRecord::StraightEdge {
                            delta_x: Twips::from_pixels(95.0),
                            delta_y: Twips::from_pixels(-131.0),
                        },
                        ShapeRecord::StraightEdge {
                            delta_x: Twips::from_pixels(59.0),
                            delta_y: Twips::from_pixels(17.0),
                        },
                        ShapeRecord::CurvedEdge {
                            control_delta_x: Twips::from_pixels(62.1),
                            control_delta_y: Twips::from_pixels(57.0),
                            anchor_delta_x: Twips::from_pixels(-62.1),
                            anchor_delta_y: Twips::from_pixels(57.0),
                        },
                        ShapeRecord::CurvedEdge {
                            control_delta_x: Twips::from_pixels(-73.2),
                            control_delta_y: Twips::from_pixels(-70.6),
                            anchor_delta_x: Twips::from_pixels(-80.8),
                            anchor_delta_y: Twips::from_pixels(70.6),
                        },
                        ShapeRecord::StraightEdge {
                            delta_x: Twips::from_pixels(95.0),
                            delta_y: Twips::from_pixels(-131.0),
                        },
                        ShapeRecord::StraightEdge {
                            delta_x: Twips::from_pixels(59.0),
                            delta_y: Twips::from_pixels(17.0),
                        },
                        ShapeRecord::CurvedEdge {
                            control_delta_x: Twips::from_pixels(62.1),
                            control_delta_y: Twips::from_pixels(57.0),
                            anchor_delta_x: Twips::from_pixels(-62.1),
                            anchor_delta_y: Twips::from_pixels(57.0),
                        },
                        ShapeRecord::CurvedEdge {
                            control_delta_x: Twips::from_pixels(-73.2),
                            control_delta_y: Twips::from_pixels(-70.6),
                            anchor_delta_x: Twips::from_pixels(-80.8),
                            anchor_delta_y: Twips::from_pixels(70.6),
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
            1, // Minimum version not listed in SWF19.
            Tag::DefineSceneAndFrameLabelData(DefineSceneAndFrameLabelData {
                scenes: vec![
                    FrameLabelData {
                        frame_num: 0,
                        label: "Scene 1".to_string(),
                    },
                    FrameLabelData {
                        frame_num: 25,
                        label: "Scene2Scene2Scene2Scene2Scene2".to_string(),
                    },
                    FrameLabelData {
                        frame_num: 26,
                        label: "test日本語test".to_string(),
                    },
                ],
                frame_labels: vec![
                    FrameLabelData {
                        frame_num: 0,
                        label: "a".to_string(),
                    },
                    FrameLabelData {
                        frame_num: 9,
                        label: "b".to_string(),
                    },
                    FrameLabelData {
                        frame_num: 17,
                        label: "❤😁aaa".to_string(),
                    },
                    FrameLabelData {
                        frame_num: 25,
                        label: "frameInScene2".to_string(),
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
                has_fill_winding_rule: false,
                has_non_scaling_strokes: true,
                has_scaling_strokes: false,
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
                    ShapeRecord::StyleChange(StyleChangeData {
                        move_to: None,
                        fill_style_0: None,
                        fill_style_1: Some(1),
                        line_style: None,
                        new_styles: None,
                    }),
                    ShapeRecord::StraightEdge {
                        delta_x: Twips::from_pixels(20.0),
                        delta_y: Twips::from_pixels(0.0),
                    },
                    ShapeRecord::StraightEdge {
                        delta_x: Twips::from_pixels(0.0),
                        delta_y: Twips::from_pixels(20.0),
                    },
                    ShapeRecord::StraightEdge {
                        delta_x: Twips::from_pixels(-20.0),
                        delta_y: Twips::from_pixels(0.0),
                    },
                    ShapeRecord::StraightEdge {
                        delta_x: Twips::from_pixels(0.0),
                        delta_y: Twips::from_pixels(-20.0),
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
                has_fill_winding_rule: false,
                has_non_scaling_strokes: true,
                has_scaling_strokes: false,
                styles: ShapeStyles {
                    fill_styles: vec![FillStyle::RadialGradient(Gradient {
                        matrix: Matrix {
                            translate_x: Twips::from_pixels(24.95),
                            translate_y: Twips::from_pixels(24.95),
                            scale_x: 0.030731201f32,
                            scale_y: 0.030731201f32,
                            rotate_skew_0: 0f32,
                            rotate_skew_1: 0f32,
                        },
                        spread: GradientSpread::Pad,
                        interpolation: GradientInterpolation::RGB,
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
                    ShapeRecord::StyleChange(StyleChangeData {
                        move_to: Some((Twips::from_pixels(50.0), Twips::from_pixels(25.0))),
                        fill_style_0: None,
                        fill_style_1: Some(1),
                        line_style: None,
                        new_styles: None,
                    }),
                    ShapeRecord::CurvedEdge {
                        control_delta_x: Twips::from_pixels(0.0),
                        control_delta_y: Twips::from_pixels(10.35),
                        anchor_delta_x: Twips::from_pixels(-7.35),
                        anchor_delta_y: Twips::from_pixels(7.3),
                    },
                    ShapeRecord::CurvedEdge {
                        control_delta_x: Twips::from_pixels(-7.3),
                        control_delta_y: Twips::from_pixels(7.35),
                        anchor_delta_x: Twips::from_pixels(-10.35),
                        anchor_delta_y: Twips::from_pixels(0.0),
                    },
                    ShapeRecord::CurvedEdge {
                        control_delta_x: Twips::from_pixels(-10.35),
                        control_delta_y: Twips::from_pixels(0.0),
                        anchor_delta_x: Twips::from_pixels(-7.35),
                        anchor_delta_y: Twips::from_pixels(-7.35),
                    },
                    ShapeRecord::CurvedEdge {
                        control_delta_x: Twips::from_pixels(-7.3),
                        control_delta_y: Twips::from_pixels(-7.3),
                        anchor_delta_x: Twips::from_pixels(0.0),
                        anchor_delta_y: Twips::from_pixels(-10.35),
                    },
                    ShapeRecord::CurvedEdge {
                        control_delta_x: Twips::from_pixels(0.0),
                        control_delta_y: Twips::from_pixels(-10.35),
                        anchor_delta_x: Twips::from_pixels(7.3),
                        anchor_delta_y: Twips::from_pixels(-7.35),
                    },
                    ShapeRecord::CurvedEdge {
                        control_delta_x: Twips::from_pixels(7.35),
                        control_delta_y: Twips::from_pixels(-7.3),
                        anchor_delta_x: Twips::from_pixels(10.35),
                        anchor_delta_y: Twips::from_pixels(0.0),
                    },
                    ShapeRecord::CurvedEdge {
                        control_delta_x: Twips::from_pixels(10.35),
                        control_delta_y: Twips::from_pixels(0.0),
                        anchor_delta_x: Twips::from_pixels(7.3),
                        anchor_delta_y: Twips::from_pixels(7.3),
                    },
                    ShapeRecord::CurvedEdge {
                        control_delta_x: Twips::from_pixels(7.35),
                        control_delta_y: Twips::from_pixels(7.35),
                        anchor_delta_x: Twips::from_pixels(0.0),
                        anchor_delta_y: Twips::from_pixels(10.35),
                    },
                ],
            }),
            read_tag_bytes_from_file("tests/swfs/defineshape3.swf", TagCode::DefineShape3),
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
                has_fill_winding_rule: false,
                has_non_scaling_strokes: true,
                has_scaling_strokes: false,
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
                                    translate_x: Twips::from_pixels(49.55),
                                    translate_y: Twips::from_pixels(46.55),
                                    scale_x: 0.06199646f32,
                                    scale_y: 0.06199646f32,
                                    rotate_skew_0: 0f32,
                                    rotate_skew_1: 0f32,
                                },
                                spread: GradientSpread::Pad,
                                interpolation: GradientInterpolation::LinearRGB,
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
                            focal_point: 0.56640625f32,
                        },
                    ],
                    line_styles: vec![
                        LineStyle {
                            width: Twips::from_pixels(20.0),
                            color: Color {
                                r: 0,
                                g: 153,
                                b: 0,
                                a: 255,
                            },
                            start_cap: LineCapStyle::None,
                            end_cap: LineCapStyle::None,
                            join_style: LineJoinStyle::Bevel,
                            fill_style: None,
                            allow_scale_x: false,
                            allow_scale_y: false,
                            is_pixel_hinted: true,
                            allow_close: true,
                        },
                        LineStyle {
                            width: Twips::from_pixels(20.0),
                            color: Color {
                                r: 0,
                                g: 0,
                                b: 0,
                                a: 0,
                            },
                            start_cap: LineCapStyle::Round,
                            end_cap: LineCapStyle::Round,
                            join_style: LineJoinStyle::Round,
                            fill_style: Some(FillStyle::LinearGradient(Gradient {
                                matrix: Matrix {
                                    translate_x: Twips::from_pixels(50.0),
                                    translate_y: Twips::from_pixels(50.0),
                                    scale_x: 0.07324219f32,
                                    scale_y: 0.07324219f32,
                                    rotate_skew_0: 0f32,
                                    rotate_skew_1: 0f32,
                                },
                                spread: GradientSpread::Pad,
                                interpolation: GradientInterpolation::RGB,
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
                            })),
                            allow_scale_x: true,
                            allow_scale_y: false,
                            is_pixel_hinted: true,
                            allow_close: true,
                        },
                        LineStyle {
                            width: Twips::from_pixels(20.0),
                            color: Color {
                                r: 0,
                                g: 153,
                                b: 0,
                                a: 255,
                            },
                            start_cap: LineCapStyle::Round,
                            end_cap: LineCapStyle::Round,
                            join_style: LineJoinStyle::Miter(56f32),
                            fill_style: None,
                            allow_scale_x: true,
                            allow_scale_y: false,
                            is_pixel_hinted: true,
                            allow_close: true,
                        },
                    ],
                },
                shape: vec![
                    ShapeRecord::StyleChange(StyleChangeData {
                        move_to: Some((Twips::from_pixels(150.0), Twips::from_pixels(0.0))),
                        fill_style_0: None,
                        fill_style_1: Some(1),
                        line_style: Some(1),
                        new_styles: None,
                    }),
                    ShapeRecord::StraightEdge {
                        delta_x: Twips::from_pixels(100.0),
                        delta_y: Twips::from_pixels(0.0),
                    },
                    ShapeRecord::StraightEdge {
                        delta_x: Twips::from_pixels(0.0),
                        delta_y: Twips::from_pixels(100.0),
                    },
                    ShapeRecord::StyleChange(StyleChangeData {
                        move_to: None,
                        fill_style_0: None,
                        fill_style_1: None,
                        line_style: Some(3),
                        new_styles: None,
                    }),
                    ShapeRecord::StraightEdge {
                        delta_x: Twips::from_pixels(-100.0),
                        delta_y: Twips::from_pixels(0.0),
                    },
                    ShapeRecord::StraightEdge {
                        delta_x: Twips::from_pixels(0.0),
                        delta_y: Twips::from_pixels(-100.0),
                    },
                    ShapeRecord::StyleChange(StyleChangeData {
                        move_to: Some((Twips::from_pixels(100.0), Twips::from_pixels(50.0))),
                        fill_style_0: None,
                        fill_style_1: Some(2),
                        line_style: Some(2),
                        new_styles: None,
                    }),
                    ShapeRecord::CurvedEdge {
                        control_delta_x: Twips::from_pixels(0.0),
                        control_delta_y: Twips::from_pixels(20.65),
                        anchor_delta_x: Twips::from_pixels(-14.65),
                        anchor_delta_y: Twips::from_pixels(14.6),
                    },
                    ShapeRecord::CurvedEdge {
                        control_delta_x: Twips::from_pixels(-14.7),
                        control_delta_y: Twips::from_pixels(14.75),
                        anchor_delta_x: Twips::from_pixels(-20.65),
                        anchor_delta_y: Twips::from_pixels(0.0),
                    },
                    ShapeRecord::CurvedEdge {
                        control_delta_x: Twips::from_pixels(-20.7),
                        control_delta_y: Twips::from_pixels(0.0),
                        anchor_delta_x: Twips::from_pixels(-14.65),
                        anchor_delta_y: Twips::from_pixels(-14.75),
                    },
                    ShapeRecord::CurvedEdge {
                        control_delta_x: Twips::from_pixels(-14.65),
                        control_delta_y: Twips::from_pixels(-14.6),
                        anchor_delta_x: Twips::from_pixels(0.0),
                        anchor_delta_y: Twips::from_pixels(-20.65),
                    },
                    ShapeRecord::CurvedEdge {
                        control_delta_x: Twips::from_pixels(0.0),
                        control_delta_y: Twips::from_pixels(-20.7),
                        anchor_delta_x: Twips::from_pixels(14.65),
                        anchor_delta_y: Twips::from_pixels(-14.7),
                    },
                    ShapeRecord::CurvedEdge {
                        control_delta_x: Twips::from_pixels(14.65),
                        control_delta_y: Twips::from_pixels(-14.6),
                        anchor_delta_x: Twips::from_pixels(20.7),
                        anchor_delta_y: Twips::from_pixels(0.0),
                    },
                    ShapeRecord::CurvedEdge {
                        control_delta_x: Twips::from_pixels(20.65),
                        control_delta_y: Twips::from_pixels(0.0),
                        anchor_delta_x: Twips::from_pixels(14.7),
                        anchor_delta_y: Twips::from_pixels(14.6),
                    },
                    ShapeRecord::CurvedEdge {
                        control_delta_x: Twips::from_pixels(14.65),
                        control_delta_y: Twips::from_pixels(14.7),
                        anchor_delta_x: Twips::from_pixels(0.0),
                        anchor_delta_y: Twips::from_pixels(20.7),
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
                data: vec![
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
                matrix: Matrix::new(),
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
                    height: Some(320),
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
            Tag::DoAction(vec![
                150, 10, 0, 0, 84, 101, 115, 116, 105, 110, 103, 33, 0, 38, 0,
            ]),
            read_tag_bytes_from_file("tests/swfs/DoAction-CS6.swf", TagCode::DoAction),
        ),
        (
            6,
            Tag::DoInitAction {
                id: 2,
                action_data: vec![150, 6, 0, 0, 116, 101, 115, 116, 0, 38, 0],
            },
            read_tag_bytes_from_file("tests/swfs/DoInitAction-CS6.swf", TagCode::DoInitAction),
        ),
        (
            6,
            Tag::EnableDebugger("$1$ve$EG3LE6bumvJ2pR8F5qXny/".to_string()),
            read_tag_bytes_from_file(
                "tests/swfs/EnableDebugger2-CS6.swf",
                TagCode::EnableDebugger2,
            ),
        ),
        (
            10,
            Tag::EnableTelemetry {
                password_hash: vec![],
            },
            read_tag_bytes_from_file("tests/swfs/EnableTelemetry.swf", TagCode::EnableTelemetry),
        ),
        (
            10,
            Tag::EnableTelemetry {
                password_hash: vec![
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
                name: "Test💯".to_string(),
            }]),
            read_tag_bytes_from_file("tests/swfs/ExportAssets-CS6.swf", TagCode::ExportAssets),
        ),
        (
            8,
            Tag::FileAttributes(FileAttributes {
                use_direct_blit: false,
                use_gpu: true,
                has_metadata: false,
                is_action_script_3: true,
                use_network_sandbox: false,
            }),
            vec![0b01_000100, 0b00010001, 0b00101000, 0, 0, 0],
        ),
        (
            3,
            Tag::FrameLabel(FrameLabel {
                label: "test".to_string(),
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
                label: "anchor_tag".to_string(),
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
                url: "ExportAssets-CS6.swf".to_string(),
                imports: vec![ExportedAsset {
                    id: 1,
                    name: "Test💯".to_string(),
                }],
            },
            read_tag_bytes_from_file("tests/swfs/ImportAssets-CS6.swf", TagCode::ImportAssets),
        ),
        (
            8,
            Tag::ImportAssets {
                url: "ExportAssets-CS6.swf".to_string(),
                imports: vec![ExportedAsset {
                    id: 1,
                    name: "Test💯".to_string(),
                }],
            },
            read_tag_bytes_from_file("tests/swfs/ImportAssets2-CS6.swf", TagCode::ImportAssets2),
        ),
        (
            1,
            Tag::JpegTables(vec![
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
            Tag::Metadata("aa!".to_string()),
            vec![0b01_000100, 0b000_10011, b'a', b'a', b'!', 0],
        ),
        (
            4,
            Tag::PlaceObject(Box::new(PlaceObject {
                version: 2,
                action: PlaceObjectAction::Place(1),
                depth: 1,
                matrix: Some(Matrix::new()),
                color_transform: None,
                ratio: None,
                name: None,
                clip_depth: None,
                class_name: None,
                filters: vec![],
                background_color: None,
                blend_mode: BlendMode::Normal,
                clip_actions: vec![],
                is_image: false,
                is_bitmap_cached: false,
                is_visible: true,
                amf_data: None,
            })),
            read_tag_bytes_from_file("tests/swfs/DefineShape.swf", TagCode::PlaceObject2),
        ),
        (
            6, // ClipActions added in SWF version 5-6.
            Tag::PlaceObject(Box::new(PlaceObject {
                version: 2,
                action: PlaceObjectAction::Place(2),
                depth: 1,
                matrix: Some(Matrix::new()),
                color_transform: None,
                ratio: None,
                name: None,
                clip_depth: None,
                class_name: None,
                filters: vec![],
                background_color: None,
                blend_mode: BlendMode::Normal,
                clip_actions: vec![
                    ClipAction {
                        events: vec![ClipEvent::Press, ClipEvent::Release]
                            .into_iter()
                            .collect(),
                        key_code: None,
                        action_data: vec![150, 3, 0, 0, 65, 0, 38, 0],
                    },
                    ClipAction {
                        events: vec![ClipEvent::KeyPress].into_iter().collect(),
                        key_code: Some(99),
                        action_data: vec![150, 3, 0, 0, 66, 0, 38, 0],
                    },
                    ClipAction {
                        events: vec![ClipEvent::EnterFrame].into_iter().collect(),
                        key_code: None,
                        action_data: vec![150, 3, 0, 0, 67, 0, 38, 0],
                    },
                ],
                is_image: false,
                is_bitmap_cached: false,
                is_visible: true,
                amf_data: None,
            })),
            read_tag_bytes_from_file(
                "tests/swfs/PlaceObject2-ClipActions-CS6.swf",
                TagCode::PlaceObject2,
            ),
        ),
        (
            8,
            Tag::PlaceObject(Box::new(PlaceObject {
                version: 3,
                action: PlaceObjectAction::Place(2),
                depth: 1,
                matrix: Some(Matrix {
                    translate_x: Twips::from_pixels(10.0),
                    translate_y: Twips::from_pixels(10.0),
                    rotate_skew_0: 0f32,
                    rotate_skew_1: 0f32,
                    scale_x: 2.0f32,
                    scale_y: 2.0f32,
                }),
                color_transform: Some(ColorTransform {
                    a_multiply: 1.0f32,
                    a_add: 80,
                    r_multiply: 0.5f32,
                    r_add: 60,
                    g_multiply: 0.25f32,
                    g_add: 40,
                    b_multiply: 0.75f32,
                    b_add: 20,
                }),
                ratio: None,
                name: Some("test".to_string()),
                clip_depth: None,
                class_name: None,
                filters: vec![
                    Filter::GradientBevelFilter(Box::new(GradientBevelFilter {
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
                        blur_x: 5f64,
                        blur_y: 5f64,
                        angle: 0.7853851318359375f64,
                        distance: 5f64,
                        strength: 1f32,
                        is_inner: true,
                        is_knockout: true,
                        is_on_top: false,
                        num_passes: 3,
                    })),
                    Filter::GradientGlowFilter(Box::new(GradientGlowFilter {
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
                        blur_x: 30f64,
                        blur_y: 30f64,
                        angle: 0.174530029296875f64,
                        distance: 5f64,
                        strength: 0.19921875f32,
                        is_inner: false,
                        is_knockout: false,
                        is_on_top: true,
                        num_passes: 1,
                    })),
                    Filter::BlurFilter(Box::new(BlurFilter {
                        blur_x: 30f64,
                        blur_y: 20f64,
                        num_passes: 2,
                    })),
                ],
                background_color: Some(Color {
                    r: 255,
                    g: 0,
                    b: 0,
                    a: 255,
                }),
                blend_mode: BlendMode::Difference,
                clip_actions: vec![
                    ClipAction {
                        events: vec![ClipEvent::ReleaseOutside, ClipEvent::RollOver]
                            .into_iter()
                            .collect(),
                        key_code: None,
                        action_data: vec![0],
                    },
                    ClipAction {
                        events: vec![ClipEvent::Data].into_iter().collect(),
                        key_code: None,
                        action_data: vec![150, 3, 0, 0, 66, 0, 38, 0],
                    },
                ],
                is_image: false,
                is_bitmap_cached: true,
                is_visible: false,
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
                    translate_x: Twips::from_pixels(10.0),
                    translate_y: Twips::from_pixels(10.0),
                    rotate_skew_0: 0.0,
                    rotate_skew_1: 0.0,
                    scale_x: 1.0,
                    scale_y: 1.0,
                }),
                color_transform: None,
                ratio: None,
                name: None,
                clip_depth: None,
                class_name: None,
                filters: vec![],
                background_color: None,
                blend_mode: BlendMode::Normal,
                clip_actions: vec![],
                is_image: false,
                is_bitmap_cached: false,
                is_visible: true,
                amf_data: Some(vec![
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
            Tag::Protect(Some("$1$d/$yMscKH17OJ0paJT.e67iz0".to_string())),
            read_tag_bytes_from_file("tests/swfs/protect.swf", TagCode::Protect),
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
                    class_name: "foo.Test".to_string(),
                },
                SymbolClassLink {
                    id: 0,
                    class_name: "DocumentTest".to_string(),
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
                class_name: "TestSound".to_string(),
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
                data: vec![0, 0, 132, 0, 4, 4, 17, 38, 190, 190, 190, 190, 201, 182],
            }),
            read_tag_bytes_from_file("tests/swfs/DefineVideoStream.swf", TagCode::VideoFrame),
        ),
        (
            1,
            Tag::Unknown {
                tag_code: 512,
                data: vec![],
            },
            vec![0b00_000000, 0b10000000],
        ),
        (
            1,
            Tag::Unknown {
                tag_code: 513,
                data: vec![1, 2],
            },
            vec![0b01_000010, 0b10000000, 1, 2],
        ),
        (
            1,
            Tag::Unknown {
                tag_code: 513,
                data: vec![0; 64],
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
            Action::GetUrl {
                url: String::from("a"),
                target: String::from("b"),
            },
            vec![0x83, 4, 0, 97, 0, 98, 0],
        ),
        (
            4,
            Action::GetUrl2 {
                send_vars_method: SendVarsMethod::Post,
                is_target_sprite: true,
                is_load_vars: false,
            },
            vec![0x9A, 1, 0, 0b10_0000_10],
        ),
        (4, Action::GetVariable, vec![0x1C]),
        (3, Action::GotoFrame(11), vec![0x81, 2, 0, 11, 0]),
        (
            4,
            Action::GotoFrame2 {
                set_playing: false,
                scene_offset: 0,
            },
            vec![0x9F, 1, 0, 0],
        ),
        (
            4,
            Action::GotoFrame2 {
                set_playing: true,
                scene_offset: 259,
            },
            vec![0x9F, 3, 0, 0b11, 3, 1],
        ),
        (
            3,
            Action::GotoLabel("testb".to_string()),
            vec![0x8C, 6, 0, 116, 101, 115, 116, 98, 0],
        ),
        (4, Action::If { offset: 1 }, vec![0x9D, 2, 0, 1, 0]),
        (4, Action::Jump { offset: 1 }, vec![0x99, 2, 0, 1, 0]),
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
            Action::Push(vec![Value::Str("test".to_string())]),
            vec![0x96, 6, 0, 0, 116, 101, 115, 116, 0],
        ),
        (
            4,
            Action::Push(vec![Value::Float(0.0)]),
            vec![0x96, 5, 0, 1, 0, 0, 0, 0],
        ),
        (
            5,
            Action::Push(vec![Value::Double(1.5)]),
            vec![0x96, 9, 0, 6, 0, 0, 248, 63, 0, 0, 0, 0],
        ),
        (5, Action::Push(vec![Value::Null]), vec![0x96, 1, 0, 2]),
        (5, Action::Push(vec![Value::Undefined]), vec![0x96, 1, 0, 3]),
        (
            5,
            Action::Push(vec![Value::Null, Value::Undefined]),
            vec![0x96, 2, 0, 2, 3],
        ),
        (
            5,
            Action::Push(vec![Value::Register(1)]),
            vec![0x96, 2, 0, 4, 1],
        ),
        (
            5,
            Action::Push(vec![Value::Bool(false)]),
            vec![0x96, 2, 0, 5, 0],
        ),
        (
            5,
            Action::Push(vec![Value::Bool(true)]),
            vec![0x96, 2, 0, 5, 1],
        ),
        (
            5,
            Action::Push(vec![Value::Double(0.0)]),
            vec![0x96, 9, 0, 6, 0, 0, 0, 0, 0, 0, 0, 0],
        ),
        (
            5,
            Action::Push(vec![Value::Int(31)]),
            vec![0x96, 5, 0, 7, 31, 0, 0, 0],
        ),
        (
            5,
            Action::Push(vec![Value::ConstantPool(77)]),
            vec![0x96, 2, 0, 8, 77],
        ),
        (
            5,
            Action::Push(vec![Value::ConstantPool(257)]),
            vec![0x96, 3, 0, 9, 1, 1],
        ),
        (4, Action::RandomNumber, vec![0x30]),
        (
            3,
            Action::SetTarget("test".to_string()),
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
            Action::WaitForFrame {
                frame: 4,
                num_actions_to_skip: 10,
            },
            vec![0x8A, 3, 0, 4, 0, 10],
        ),
        (
            4,
            Action::WaitForFrame2 {
                num_actions_to_skip: 34,
            },
            vec![0x8D, 1, 0, 34],
        ),
        (
            1,
            Action::Unknown {
                opcode: 0x79,
                data: vec![],
            },
            vec![0x79],
        ),
        (
            1,
            Action::Unknown {
                opcode: 0xA0,
                data: vec![2, 3],
            },
            vec![0xA0, 2, 0, 2, 3],
        ),
    ]
}

pub fn avm2_tests() -> Vec<Avm2TestData> {
    vec![(
        10,
        AbcFile {
            major_version: 46,
            minor_version: 16,
            constant_pool: ConstantPool {
                ints: vec![],
                uints: vec![],
                doubles: vec![],
                strings: vec![
                    "".to_string(),
                    "void".to_string(),
                    "Avm2Test".to_string(),
                    "trace".to_string(),
                    "Test".to_string(),
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
                    needs_arguments_object: false,
                    needs_activation: false,
                    needs_rest: false,
                    needs_dxns: false,
                },
                Method {
                    name: Index::new(0),
                    params: vec![],
                    return_type: Index::new(0),
                    needs_arguments_object: false,
                    needs_activation: false,
                    needs_rest: false,
                    needs_dxns: false,
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
                    code: vec![
                        Op::GetLocal { index: 0 },
                        Op::PushScope,
                        Op::FindPropStrict {
                            index: Index::new(3),
                        },
                        Op::PushString {
                            value: Index::new(5),
                        },
                        Op::CallPropVoid {
                            index: Index::new(3),
                            num_args: 1,
                        },
                        Op::ReturnVoid,
                    ],
                    exceptions: vec![],
                    traits: vec![],
                },
                MethodBody {
                    method: Index::new(1),
                    max_stack: 1,
                    num_locals: 2,
                    init_scope_depth: 1,
                    max_scope_depth: 2,
                    code: vec![
                        Op::GetLocal { index: 0 },
                        Op::PushScope,
                        Op::FindPropStrict {
                            index: Index::new(2),
                        },
                        Op::CallProperty {
                            index: Index::new(2),
                            num_args: 0,
                        },
                        Op::CoerceA,
                        Op::SetLocal { index: 1 },
                        Op::GetLocal { index: 1 },
                        Op::ReturnValue,
                    ],
                    exceptions: vec![],
                    traits: vec![],
                },
            ],
        },
        read_abc_from_file("tests/swfs/Avm2Dummy.swf"),
    )]
}
