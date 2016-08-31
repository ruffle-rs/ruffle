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

type TagTestData = (Tag, Vec<u8>);

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
                num_fill_bits: 1,
                num_line_bits: 0,
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
        read_tag_bytes_from_file("test/swfs/define_shape.swf", TagCode::DefineShape)
    )
}