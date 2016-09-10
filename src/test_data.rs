use std::fs::File;
use std::vec::Vec;
use read::read_swf;
use read::tests::{read_tag_bytes_from_file, read_tag_bytes_from_file_with_index};
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

pub fn define_sprite() -> TagTestData {
    (
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
        read_tag_bytes_from_file("tests/swfs/define_sprite.swf", TagCode::DefineSprite)
    )
}

pub fn place_object_2() -> TagTestData {
    (
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
        })),
        read_tag_bytes_from_file("tests/swfs/define_shape.swf", TagCode::PlaceObject2)
    )
}

pub fn place_object_2_clip_actions() -> TagTestData {
    (
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
                    events: vec![ClipEvent::Press, ClipEvent::Release].into_iter().collect(),
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
        })),
        read_tag_bytes_from_file("tests/swfs/placeobject2-clipactions.swf", TagCode::PlaceObject2)
    )
}


pub fn place_object_3_the_works() -> TagTestData {
    (
        Tag::PlaceObject(Box::new(PlaceObject {
            version: 3,
            action: PlaceObjectAction::Place(2),
            depth: 1,
            matrix: Some(Matrix {
                translate_x: 10f32,
                translate_y: 10f32,
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
                        GradientRecord { ratio: 0, color: Color { r: 255, g: 0, b: 0, a: 255 } },
                        GradientRecord { ratio: 128, color: Color { r: 0, g: 255, b: 0, a: 0 } },
                        GradientRecord { ratio: 255, color: Color { r: 0, g: 0, b: 255, a: 0 } }
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
                        GradientRecord { ratio: 0, color: Color { r: 255, g: 255, b: 255, a: 0 } },
                        GradientRecord { ratio: 255, color: Color { r: 0, g: 0, b: 0, a: 255 } },
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
                }))
            ],
            background_color: Some(Color { r: 255, g: 0, b: 0, a: 255 }),
            blend_mode: BlendMode::Difference,
            clip_actions: vec![
                ClipAction {
                    events: vec![ClipEvent::ReleaseOutside, ClipEvent::RollOver].into_iter().collect(),
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
        })),
        read_tag_bytes_from_file("tests/swfs/placeobject3-theworks.swf", TagCode::PlaceObject3)
    )
}

pub fn frame_label() -> TagTestData {
    (
        Tag::FrameLabel { label: "test".to_string(), is_anchor: false },
        read_tag_bytes_from_file_with_index("tests/swfs/framelabel.swf", TagCode::FrameLabel, 0)
    )
}

pub fn frame_label_anchor() -> TagTestData {
    (
        Tag::FrameLabel { label: "anchor_tag".to_string(), is_anchor: true },
        read_tag_bytes_from_file_with_index("tests/swfs/framelabel.swf", TagCode::FrameLabel, 1)
    )
}