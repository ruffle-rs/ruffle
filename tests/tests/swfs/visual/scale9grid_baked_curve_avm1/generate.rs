use std::fs::File;
use swf::*;

// The AVM1 side of scale9grid_baked_curve: the same quadratic with its control point at
// x=10, inside the left corner region of grid (20,20)-(40,40), gridded by the tag alone.
// No FileAttributes tag, so the movie loads as AVM1; everything is placed by tags and no
// script runs. The AVM2 docs describe curve pre-splitting for authored symbols; this pins
// that the AVM1 runtime, like AVM2, remaps control points and never splits. Rows: tag
// grid at 3x, ungridded at 3x, authored 1x.

const PLAIN_ID: CharacterId = 1;
const GRIDDED_ID: CharacterId = 2;

fn px(v: f32) -> Twips {
    Twips::from_pixels(v as f64)
}

fn rect(x0: f32, y0: f32, x1: f32, y1: f32) -> Rectangle<Twips> {
    Rectangle {
        x_min: px(x0),
        y_min: px(y0),
        x_max: px(x1),
        y_max: px(y1),
    }
}

fn edge(dx: f32, dy: f32) -> ShapeRecord {
    ShapeRecord::StraightEdge {
        delta: PointDelta::new(px(dx), px(dy)),
    }
}

fn start_fill(x: f32, y: f32, fill: u32) -> ShapeRecord {
    ShapeRecord::StyleChange(Box::new(StyleChangeData {
        move_to: Some(Point::new(px(x), px(y))),
        fill_style_0: None,
        fill_style_1: Some(fill),
        line_style: None,
        new_styles: None,
    }))
}

fn rgb(r: u8, g: u8, b: u8) -> FillStyle {
    FillStyle::Color(Color { r, g, b, a: 255 })
}

fn curve_shape(id: CharacterId) -> Shape {
    let mut records = Vec::new();
    for (i, x) in [0.0f32, 20.0, 40.0].iter().enumerate() {
        records.push(start_fill(*x, 0.0, i as u32 + 1));
        records.extend([edge(20.0, 0.0), edge(0.0, 4.0), edge(-20.0, 0.0), edge(0.0, -4.0)]);
    }
    records.push(start_fill(0.0, 60.0, 4));
    records.push(ShapeRecord::CurvedEdge {
        control_delta: PointDelta::new(px(10.0), px(-60.0)),
        anchor_delta: PointDelta::new(px(50.0), px(60.0)),
    });
    records.push(edge(-60.0, 0.0));
    Shape {
        version: 3,
        id,
        shape_bounds: rect(0.0, 0.0, 60.0, 60.0),
        edge_bounds: rect(0.0, 0.0, 60.0, 60.0),
        flags: ShapeFlag::empty(),
        styles: ShapeStyles {
            fill_styles: vec![
                rgb(255, 0, 0),
                rgb(0, 255, 0),
                rgb(0, 0, 255),
                rgb(255, 255, 255),
            ],
            line_styles: vec![],
        },
        shape: records,
    }
}

fn place_at(id: CharacterId, depth: Depth, scale_x: f32, ty: f32) -> Tag<'static> {
    Tag::PlaceObject(Box::new(PlaceObject {
        version: 2,
        action: PlaceObjectAction::Place(id),
        depth,
        matrix: Some(Matrix {
            a: Fixed16::from_f32(scale_x),
            tx: px(10.0),
            ty: px(ty),
            ..Matrix::IDENTITY
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
        amf_data: None,
    }))
}

fn main() {
    let tags = vec![
        Tag::SetBackgroundColor(Color {
            r: 0x10,
            g: 0x10,
            b: 0x10,
            a: 255,
        }),
        Tag::DefineShape(Box::new(curve_shape(PLAIN_ID))),
        Tag::DefineShape(Box::new(curve_shape(GRIDDED_ID))),
        Tag::DefineScalingGrid {
            id: GRIDDED_ID,
            splitter_rect: rect(20.0, 20.0, 40.0, 40.0),
        },
        place_at(GRIDDED_ID, 1, 3.0, 10.0),
        place_at(PLAIN_ID, 2, 3.0, 80.0),
        place_at(PLAIN_ID, 3, 1.0, 150.0),
        Tag::ShowFrame,
    ];
    let header = Header {
        compression: Compression::None,
        version: 10,
        stage_size: rect(0.0, 0.0, 200.0, 220.0),
        frame_rate: Fixed8::from_f32(24.0),
        num_frames: 1,
    };
    let file = File::create("test.swf").expect("create");
    swf::write_swf(&header, &tags, file).expect("write");
}
