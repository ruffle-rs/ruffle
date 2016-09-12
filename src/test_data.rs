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

pub type TestData<T> = (u8, T, Vec<u8>);
pub type TagTestData = TestData<Tag>;

pub fn tag_tests() -> Vec<TagTestData> { vec![
    (
        9, // Minimum version not listed in SWF19.
        Tag::DefineBinaryData {
            id: 1,
            data: vec![84, 101, 115, 116, 105, 110, 103, 33]
        },
        read_tag_bytes_from_file("tests/swfs/definebinarydata.swf", TagCode::DefineBinaryData)
    ),

    (
        8,
        Tag::DefineScalingGrid {
            id: 2,
            splitter_rect: Rectangle { x_min: 10f32, x_max: 40f32, y_min: 10f32, y_max: 40f32 },
        },
        read_tag_bytes_from_file("tests/swfs/definescalinggrid.swf", TagCode::DefineScalingGrid)
    ),

    (
        1, // Minimum version not listed in SWF19.
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
    ),

    (
        1,
        Tag::DefineShape(Shape {
            version: 1,
            id: 1,
            shape_bounds: Rectangle { x_min: 0f32, x_max: 20f32, y_min: 0f32, y_max: 20f32 },
            edge_bounds: Rectangle { x_min: 0f32, x_max: 20f32, y_min: 0f32, y_max: 20f32 },
            has_fill_winding_rule: false,
            has_non_scaling_strokes: true,
            has_scaling_strokes: false,
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
    ),

    (
        8,
        Tag::DefineShape(Shape {
            version: 3,
            id: 1,
            shape_bounds: Rectangle { x_min: 0f32, x_max: 50f32, y_min: 0f32, y_max: 50f32 },
            edge_bounds: Rectangle { x_min: 0f32, x_max: 50f32, y_min: 0f32, y_max: 50f32 },
            has_fill_winding_rule: false,
            has_non_scaling_strokes: true,
            has_scaling_strokes: false,
            styles: ShapeStyles {
                fill_styles: vec![
                    FillStyle::RadialGradient(Gradient {
                        matrix: Matrix { translate_x: 24.95f32, translate_y: 24.95f32, scale_x: 0.030731201f32, scale_y: 0.030731201f32, rotate_skew_0: 0f32, rotate_skew_1: 0f32 },
                        spread: GradientSpread::Pad,
                        interpolation: GradientInterpolation::RGB,
                        records: vec![
                            GradientRecord { ratio: 0, color: Color { r: 255, g: 0, b: 0, a: 255 } },
                            GradientRecord { ratio: 255, color: Color { r: 0, g: 0, b: 0, a: 0 } }
                        ]
                    })
                ],
                line_styles: vec![]
            },
            shape: vec![
                ShapeRecord::StyleChange(StyleChangeData {
                    move_delta_x: 50f32,
                    move_delta_y: 25f32,
                    fill_style_0: None,
                    fill_style_1: Some(1),
                    line_style: None,
                    new_styles: None
                }),
                ShapeRecord::CurvedEdge { control_delta_x: 0f32, control_delta_y: 10.35f32, anchor_delta_x: -7.35f32, anchor_delta_y: 7.3f32 },
                ShapeRecord::CurvedEdge { control_delta_x: -7.3f32, control_delta_y: 7.35f32, anchor_delta_x: -10.35f32, anchor_delta_y: 0f32 },
                ShapeRecord::CurvedEdge { control_delta_x: -10.35f32, control_delta_y: 0f32, anchor_delta_x: -7.35f32, anchor_delta_y: -7.35f32 },
                ShapeRecord::CurvedEdge { control_delta_x: -7.3f32, control_delta_y: -7.3f32, anchor_delta_x: 0f32, anchor_delta_y: -10.35f32 },
                ShapeRecord::CurvedEdge { control_delta_x: 0f32, control_delta_y: -10.35f32, anchor_delta_x: 7.3f32, anchor_delta_y: -7.35f32 },
                ShapeRecord::CurvedEdge { control_delta_x: 7.35f32, control_delta_y: -7.3f32, anchor_delta_x: 10.35f32, anchor_delta_y: 0f32 },
                ShapeRecord::CurvedEdge { control_delta_x: 10.35f32, control_delta_y: 0f32, anchor_delta_x: 7.3f32, anchor_delta_y: 7.3f32 },
                ShapeRecord::CurvedEdge { control_delta_x: 7.35f32, control_delta_y: 7.35f32, anchor_delta_x: 0f32, anchor_delta_y: 10.35f32 }
            ]
        }),
        read_tag_bytes_from_file("tests/swfs/defineshape3.swf", TagCode::DefineShape3)
    ),

    (
        8,
        Tag::DefineShape(Shape {
            version: 4,
            id: 1,
            shape_bounds: Rectangle { x_min: -10f32, x_max: 260f32, y_min: -10f32, y_max: 110f32 },
            edge_bounds: Rectangle { x_min: 0f32, x_max: 250f32, y_min: 0f32, y_max: 100f32 },
            has_fill_winding_rule: false,
            has_non_scaling_strokes: true,
            has_scaling_strokes: false,
            styles: ShapeStyles {
                fill_styles: vec![
                    FillStyle::Color(Color { r: 255, g: 0, b: 0, a: 255 }),
                    FillStyle::FocalGradient {
                        gradient: Gradient {
                            matrix: Matrix { translate_x: 49.55f32, translate_y: 46.55f32, scale_x: 0.06199646f32, scale_y: 0.06199646f32, rotate_skew_0: 0f32, rotate_skew_1: 0f32 },
                            spread: GradientSpread::Pad,
                            interpolation: GradientInterpolation::LinearRGB,
                            records: vec![
                                GradientRecord { ratio: 0, color: Color { r: 255, g: 0, b: 0, a: 255 } },
                                GradientRecord { ratio: 255, color: Color { r: 0, g: 0, b: 0, a: 0 } }
                            ]
                        },
                        focal_point: 0.56640625f32
                    }
                ],
                line_styles: vec![
                    LineStyle { 
                        width: 400,
                        color: Color { r: 0, g: 153, b: 0, a: 255 },
                        start_cap: LineCapStyle::None,
                        end_cap: LineCapStyle::None,
                        join_style: LineJoinStyle::Bevel,
                        fill_style: None,
                        allow_scale_x: false,
                        allow_scale_y: false,
                        is_pixel_hinted: true,
                        allow_close: true
                    },
                    LineStyle {
                        width: 400,
                        color: Color { r: 0, g: 0, b: 0, a: 0 },
                        start_cap: LineCapStyle::Round,
                        end_cap: LineCapStyle::Round,
                        join_style: LineJoinStyle::Round,
                        fill_style: Some(
                            FillStyle::LinearGradient(Gradient {
                                matrix: Matrix { translate_x: 50f32, translate_y: 50f32, scale_x: 0.07324219f32, scale_y: 0.07324219f32, rotate_skew_0: 0f32, rotate_skew_1: 0f32 },
                                spread: GradientSpread::Pad,
                                interpolation: GradientInterpolation::RGB,
                                records: vec![
                                    GradientRecord { ratio: 0, color: Color { r: 255, g: 255, b: 255, a: 255 } },
                                    GradientRecord { ratio: 255, color: Color { r: 0, g: 0, b: 0, a: 255 } }
                                ]
                            })),
                            allow_scale_x: true,
                            allow_scale_y: false,
                            is_pixel_hinted: true,
                            allow_close: true
                        },
                        LineStyle {
                            width: 400,
                            color: Color { r: 0, g: 153, b: 0, a: 255 },
                            start_cap: LineCapStyle::Round,
                            end_cap: LineCapStyle::Round,
                            join_style: LineJoinStyle::Miter(56f32),
                            fill_style: None,
                            allow_scale_x: true,
                            allow_scale_y: false,
                            is_pixel_hinted: true,
                            allow_close: true
                        }
                    ]
            },
            shape: vec![
                ShapeRecord::StyleChange(StyleChangeData {
                    move_delta_x: 150f32,
                    move_delta_y: 0f32,
                    fill_style_0: None,
                    fill_style_1: Some(1),
                    line_style: Some(1),
                    new_styles: None
                }),
                ShapeRecord::StraightEdge { delta_x: 100f32, delta_y: 0f32 },
                ShapeRecord::StraightEdge { delta_x: 0f32, delta_y: 100f32 },
                ShapeRecord::StyleChange(StyleChangeData {
                    move_delta_x: 0f32,
                    move_delta_y: 0f32,
                    fill_style_0: None,
                    fill_style_1: None,
                    line_style: Some(3),
                    new_styles: None
                }),
                ShapeRecord::StraightEdge { delta_x: -100f32, delta_y: 0f32 },
                ShapeRecord::StraightEdge { delta_x: 0f32, delta_y: -100f32 },
                ShapeRecord::StyleChange(StyleChangeData {
                    move_delta_x: 100f32,
                    move_delta_y: 50f32,
                    fill_style_0: None,
                    fill_style_1: Some(2),
                    line_style: Some(2),
                    new_styles: None
                }),
                ShapeRecord::CurvedEdge { control_delta_x: 0f32, control_delta_y: 20.65f32, anchor_delta_x: -14.65f32, anchor_delta_y: 14.6f32 },
                ShapeRecord::CurvedEdge { control_delta_x: -14.7f32, control_delta_y: 14.75f32, anchor_delta_x: -20.65f32, anchor_delta_y: 0f32 },
                ShapeRecord::CurvedEdge { control_delta_x: -20.7f32, control_delta_y: 0f32, anchor_delta_x: -14.65f32, anchor_delta_y: -14.75f32 },
                ShapeRecord::CurvedEdge { control_delta_x: -14.65f32, control_delta_y: -14.6f32, anchor_delta_x: 0f32, anchor_delta_y: -20.65f32 },
                ShapeRecord::CurvedEdge { control_delta_x: 0f32, control_delta_y: -20.7f32, anchor_delta_x: 14.65f32, anchor_delta_y: -14.7f32 },
                ShapeRecord::CurvedEdge { control_delta_x: 14.65f32, control_delta_y: -14.6f32, anchor_delta_x: 20.7f32, anchor_delta_y: 0f32 },
                ShapeRecord::CurvedEdge { control_delta_x: 20.65f32, control_delta_y: 0f32, anchor_delta_x: 14.7f32, anchor_delta_y: 14.6f32 },
                ShapeRecord::CurvedEdge { control_delta_x: 14.65f32, control_delta_y: 14.7f32, anchor_delta_x: 0f32, anchor_delta_y: 20.7f32 }
            ]
        }),
        read_tag_bytes_from_file("tests/swfs/defineshape4.swf", TagCode::DefineShape4)
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
        read_tag_bytes_from_file("tests/swfs/define_sprite.swf", TagCode::DefineSprite)
    ),

    (
        5,
        Tag::DoAction(
            vec![150, 10, 0, 0, 84, 101, 115, 116, 105, 110, 103, 33, 0, 38, 0],
        ),
        read_tag_bytes_from_file("tests/swfs/doaction.swf", TagCode::DoAction)
    ),

    (
        6,
        Tag::DoInitAction {
            id: 2,
            action_data: vec![150, 6, 0, 0, 116, 101, 115, 116, 0, 38, 0],
        },
        read_tag_bytes_from_file("tests/swfs/doinitaction.swf", TagCode::DoInitAction)
    ),

    (
        6,
        Tag::EnableDebugger("$1$ve$EG3LE6bumvJ2pR8F5qXny/".to_string()),
        read_tag_bytes_from_file("tests/swfs/enabledebugger2.swf", TagCode::EnableDebugger2)
    ),

    (
        10,
        Tag::EnableTelemetry {
            password_hash: vec![]
        },
        read_tag_bytes_from_file("tests/swfs/enabletelemetry.swf", TagCode::EnableTelemetry)
    ),

    (
        10,
        Tag::EnableTelemetry {
            password_hash: vec![
                207, 128, 205, 138, 237, 72, 45, 93, 21, 39, 215, 220, 114, 252, 239, 248,
                78, 99, 38, 89, 40, 72, 68, 125, 45, 192, 176, 232, 125, 252, 154, 144
            ]
        },
        read_tag_bytes_from_file("tests/swfs/enabletelemetry-password.swf", TagCode::EnableTelemetry)
    ),

    (
        6,
        Tag::ExportAssets(vec![
            ExportedAsset { id: 2, name: "Test💯".to_string() },
        ]),
        read_tag_bytes_from_file("tests/swfs/exportassets.swf", TagCode::ExportAssets)
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
        vec![0b01_000100, 0b00010001, 0b00101000, 0, 0, 0]
    ),
        
    (
        3,
        Tag::FrameLabel { label: "test".to_string(), is_anchor: false },
        read_tag_bytes_from_file_with_index("tests/swfs/framelabel.swf", TagCode::FrameLabel, 0)
    ),

    (
        6, // Anchor tags supported in SWF version 6 and later.
        Tag::FrameLabel { label: "anchor_tag".to_string(), is_anchor: true },
        read_tag_bytes_from_file_with_index("tests/swfs/framelabel.swf", TagCode::FrameLabel, 1)
    ),

    (
        7,
        Tag::ImportAssets {
            url: "exportassets.swf".to_string(),
            imports: vec![ExportedAsset { id: 1, name: "Test💯".to_string() }],
        },
        read_tag_bytes_from_file("tests/swfs/importassets.swf", TagCode::ImportAssets)
    ),

    (
        8,
        Tag::ImportAssets {
            url: "exportassets.swf".to_string(),
            imports: vec![ExportedAsset { id: 1, name: "Test💯".to_string() }],
        },
        read_tag_bytes_from_file("tests/swfs/importassets2.swf", TagCode::ImportAssets2)
    ),

    (
        1,
        Tag::Metadata("aa!".to_string()),
        vec![0b01_000100, 0b000_10011, 'a' as u8, 'a' as u8, '!' as u8, 0]
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
        })),
        read_tag_bytes_from_file("tests/swfs/define_shape.swf", TagCode::PlaceObject2)
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
    ),

    (
        8,
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
    ),

    (
        5, // Password supported in SWF version 5 or later.
        Tag::Protect(Some("$1$d/$yMscKH17OJ0paJT.e67iz0".to_string())),
        read_tag_bytes_from_file(
            "tests/swfs/protect.swf",
            TagCode::Protect
        )
    ),

    (
        1,
        Tag::SetBackgroundColor(Color { r: 64, g: 150, b: 255, a: 255 }),
        vec![0b01_000011, 0b00000010, 64, 150, 255]
    ),

    (
        7,
        Tag::SetTabIndex { depth: 2, tab_index: 1 },
        vec![0b10_000100, 0b000_10000, 2, 0, 1, 0],
    ),

    (
        7,
        Tag::ScriptLimits { max_recursion_depth: 256, timeout_in_seconds: 42 },
        read_tag_bytes_from_file("tests/swfs/scriptlimits.swf", TagCode::ScriptLimits)
    ),

    (1, Tag::ShowFrame, vec![0b01_000000, 0]),

    (
        9,
        Tag::SymbolClass(vec![
            SymbolClassLink { id: 2, class_name: "foo.Test".to_string() },
            SymbolClassLink { id: 0, class_name: "DocumentTest".to_string() },
        ]),
        read_tag_bytes_from_file("tests/swfs/symbolclass.swf", TagCode::SymbolClass)
    ),

    (
        4,
        Tag::StartSound {
            id: 1,
            sound_info: Box::new(SoundInfo {
                event: SoundEvent::Event,
                in_sample: None,
                out_sample: None,
                num_loops: 1,
                envelope: None,
            }),
        },
        read_tag_bytes_from_file("tests/swfs/definesound.swf", TagCode::StartSound)
    ),

    (1, Tag::Unknown { tag_code: 512, data: vec![] }, vec![0b00_000000, 0b10000000]),
    (1, Tag::Unknown { tag_code: 513, data: vec![1, 2] },  vec![0b01_000010, 0b10000000, 1, 2]),
    (
        1,
        Tag::Unknown { tag_code: 513, data: vec![0; 64] }, 
        vec![0b01_111111, 0b10000000, 64, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
    ),
] }