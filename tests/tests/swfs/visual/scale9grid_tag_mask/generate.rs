use std::fs::File;
use swf::*;

// A gridded, scaled clip-depth masker beside an ungridded twin: both reveals must match,
// since maskers render plainly. The third column places the gridded character visible,
// where slicing does engage.

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

fn styled_start() -> ShapeRecord {
    ShapeRecord::StyleChange(Box::new(StyleChangeData {
        move_to: Some(Point::new(px(0.0), px(0.0))),
        fill_style_0: None,
        fill_style_1: Some(1),
        line_style: None,
        new_styles: None,
    }))
}

fn notch_shape(id: CharacterId) -> Shape {
    let mut shape = vec![styled_start()];
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

fn sheet_shape(id: CharacterId) -> Shape {
    let shape = vec![
        styled_start(),
        edge(190.0, 0.0),
        edge(0.0, 70.0),
        edge(-190.0, 0.0),
        edge(0.0, -70.0),
    ];
    Shape {
        version: 3,
        id,
        shape_bounds: rect(0.0, 0.0, 190.0, 70.0),
        edge_bounds: rect(0.0, 0.0, 190.0, 70.0),
        flags: ShapeFlag::empty(),
        styles: ShapeStyles {
            fill_styles: vec![FillStyle::Color(Color { r: 255, g: 0, b: 255, a: 255 })],
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
    tags.push(Tag::DefineShape(Box::new(sheet_shape(200))));

    for col in 0..3u16 {
        let shape_id = 210 + col;
        tags.push(Tag::DefineShape(Box::new(notch_shape(shape_id))));
        if col != 1 {
            tags.push(Tag::DefineScalingGrid {
                id: shape_id,
                splitter_rect: rect(20.0, 20.0, 40.0, 40.0),
            });
        }
        let base = col * 10;
        let x = 10.0 + col as f32 * 195.0;
        let scaled = Matrix {
            a: Fixed16::from_f32(3.0),
            tx: px(x),
            ty: px(5.0),
            ..Matrix::IDENTITY
        };
        if col < 2 {
            let mut masker = place(shape_id, base + 1);
            masker.matrix = Some(scaled);
            masker.clip_depth = Some(base + 2);
            tags.push(Tag::PlaceObject(masker));

            let mut sheet = place(200, base + 2);
            sheet.matrix = Some(Matrix {
                tx: px(x),
                ty: px(5.0),
                ..Matrix::IDENTITY
            });
            tags.push(Tag::PlaceObject(sheet));
        } else {
            let mut p = place(shape_id, base + 1);
            p.matrix = Some(scaled);
            tags.push(Tag::PlaceObject(p));
        }
    }
    tags.push(Tag::ShowFrame);

    let header = Header {
        compression: Compression::None,
        version: 10,
        stage_size: rect(0.0, 0.0, 600.0, 80.0),
        frame_rate: Fixed8::from_f32(24.0),
        num_frames: 1,
    };
    swf::write_swf(&header, &tags, File::create("test.swf").expect("create")).expect("write");
}
