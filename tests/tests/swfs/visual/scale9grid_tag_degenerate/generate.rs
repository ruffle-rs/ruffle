use std::fs::File;
use swf::*;

// Degenerate grid rectangles the setter can never produce. Each row places the same art
// twice, left with the grid and right without: Flash Player refuses these rectangles and
// scales plainly, so the two columns must match. The rectangle entirely past the bounds
// on the high side lives in scale9grid_tag.
const ROWS: [[f32; 4]; 8] = [
    [30.0, 20.0, 30.0, 40.0],
    [20.0, 30.0, 40.0, 30.0],
    [30.0, 30.0, 30.0, 30.0],
    [40.0, 40.0, 20.0, 20.0],
    [0.0, 0.0, 60.0, 60.0],
    [0.0, 20.0, 40.0, 40.0],
    [-40.0, -40.0, -20.0, -20.0],
    [-10.0, 20.0, 40.0, 40.0],
];

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

fn notch_shape(id: CharacterId) -> Shape {
    let mut shape = vec![ShapeRecord::StyleChange(Box::new(StyleChangeData {
        move_to: Some(Point::new(px(0.0), px(0.0))),
        fill_style_0: None,
        fill_style_1: Some(1),
        line_style: None,
        new_styles: None,
    }))];
    shape.extend([
        edge(60.0, 0.0),
        edge(0.0, 60.0),
        edge(-20.0, 0.0),
        edge(0.0, -20.0),
        edge(-20.0, 0.0),
        edge(0.0, 20.0),
        edge(-20.0, 0.0),
        edge(0.0, -60.0),
    ]);
    Shape {
        version: 3,
        id,
        shape_bounds: rect(0.0, 0.0, 60.0, 60.0),
        edge_bounds: rect(0.0, 0.0, 60.0, 60.0),
        flags: ShapeFlag::empty(),
        styles: ShapeStyles {
            fill_styles: vec![FillStyle::Color(Color { r: 255, g: 0, b: 0, a: 255 })],
            line_styles: vec![],
        },
        shape,
    }
}

fn place(id: CharacterId, depth: Depth) -> Box<PlaceObject<'static>> {
    Box::new(PlaceObject {
        version: 2,
        action: PlaceObjectAction::Place(id),
        depth,
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
    })
}

fn main() {
    let mut tags: Vec<Tag> = vec![
        Tag::FileAttributes(FileAttributes::IS_ACTION_SCRIPT_3),
        Tag::SetBackgroundColor(Color { r: 255, g: 255, b: 255, a: 255 }),
    ];
    for (row, splitter) in ROWS.iter().enumerate() {
        for col in 0..2u16 {
            let shape_id = (row as CharacterId + 1) * 10 + col;
            tags.push(Tag::DefineShape(Box::new(notch_shape(shape_id))));
            if col == 0 {
                let [x0, y0, x1, y1] = *splitter;
                tags.push(Tag::DefineScalingGrid {
                    id: shape_id,
                    splitter_rect: rect(x0, y0, x1, y1),
                });
            }
            let mut p = place(shape_id, (row as Depth) * 2 + col + 1);
            p.matrix = Some(Matrix {
                a: Fixed16::from_f32(3.0),
                tx: px(10.0 + col as f32 * 195.0),
                ty: px(10.0 + row as f32 * 70.0),
                ..Matrix::IDENTITY
            });
            tags.push(Tag::PlaceObject(p));
        }
    }
    tags.push(Tag::ShowFrame);

    let header = Header {
        compression: Compression::None,
        version: 10,
        stage_size: rect(0.0, 0.0, 400.0, 570.0),
        frame_rate: Fixed8::from_f32(24.0),
        num_frames: 1,
    };
    swf::write_swf(&header, &tags, File::create("test.swf").expect("create")).expect("write");
}
