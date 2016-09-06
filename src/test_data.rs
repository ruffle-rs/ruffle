use std::fs::File;
use std::vec::Vec;
use read::read_swf;
use read::tests::read_tag_bytes_from_file;
use tag_codes::TagCode;
use types::*;
use write::write_swf;

#[allow(dead_code)]
pub fn echo_swf(filename: &str) {
    let in_file = File::open(filename).unwrap();
    let swf = read_swf(in_file).unwrap();
    let out_file = File::create(filename).unwrap();
    write_swf(&swf, out_file).unwrap();
}

pub type TagTestData = (Tag, Vec<u8>);

pub fn define_scene_and_frame_label_data() -> TagTestData {
    (
        Tag::DefineSceneAndFrameLabelData {
            scenes: vec![
                FrameLabel { frame_num: 0, label: "Scene 1".to_string() },
                FrameLabel {
                    frame_num: 25,
                    label: "Scene2Scene2Scene2Scene2Scene2".to_string()
                },
                FrameLabel { frame_num: 26, label: "test日本語test".to_string() },
            ],
            frame_labels: vec![
                FrameLabel { frame_num: 0, label: "a".to_string() },
                FrameLabel { frame_num: 9, label: "b".to_string() },
                FrameLabel { frame_num: 17, label: "❤😁aaa".to_string() },
                FrameLabel { frame_num: 25, label: "frameInScene2".to_string() },
            ],
        },
        read_tag_bytes_from_file(
            "tests/swfs/define_scene_and_frame_label_data.swf",
            TagCode::DefineSceneAndFrameLabelData
        )
    )
}

pub fn define_shape() -> TagTestData {
    (
        Tag::DefineShape(Shape {
            version: 1,
            id: 1,
            shape_bounds: Rectangle { x_min: 0f32, x_max: 20f32, y_min: 0f32, y_max: 20f32 },
            edge_bounds: Rectangle { x_min: 0f32, x_max: 20f32, y_min: 0f32, y_max: 20f32 },
            styles: ShapeStyles {
                fill_styles: vec![
                    FillStyle::Color(Color { r: 255, g: 0, b: 0, a: 255 })
                ],
                line_styles: vec![],
            },
            shape: vec![
                ShapeRecord::StyleChange(StyleChangeData {
                    move_delta_x: 0f32,
                    move_delta_y: 0f32,
                    fill_style_0: None,
                    fill_style_1: Some(1),
                    line_style: None,
                    new_styles: None,
                }),
                ShapeRecord::StraightEdge {
                    delta_x: 20f32,
                    delta_y: 0f32,
                },
                ShapeRecord::StraightEdge {
                    delta_x: 0f32,
                    delta_y: 20f32,
                },
                ShapeRecord::StraightEdge {
                    delta_x: -20f32,
                    delta_y: 0f32,
                },
                ShapeRecord::StraightEdge {
                    delta_x: 0f32,
                    delta_y: -20f32,
                },
            ]
        }),
        read_tag_bytes_from_file("tests/swfs/define_shape.swf", TagCode::DefineShape)
    )
}