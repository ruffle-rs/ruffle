use std::fs::File;
use swf::*;

// Two gridded sprites: the lower one's shape is swapped by PlaceObject(Replace) on its
// second frame for art with identical bounds. A sliced tessellation served from a stale
// cache keeps drawing the first frame's art, which the compared image would show.

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

/// A 60x60 square with a 20px notch out of one edge: the bottom when `flipped` is false,
/// the top otherwise. Same bounds either way, so only the art distinguishes them.
fn notch_shape(id: CharacterId, flipped: bool, color: Color) -> Shape {
    let mut shape = vec![ShapeRecord::StyleChange(Box::new(StyleChangeData {
        move_to: Some(Point::new(px(0.0), px(0.0))),
        fill_style_0: None,
        fill_style_1: Some(1),
        line_style: None,
        new_styles: None,
    }))];
    if flipped {
        shape.extend([
            edge(20.0, 0.0),
            edge(0.0, 20.0),
            edge(20.0, 0.0),
            edge(0.0, -20.0),
            edge(20.0, 0.0),
            edge(0.0, 60.0),
            edge(-60.0, 0.0),
            edge(0.0, -60.0),
        ]);
    } else {
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
    }
    Shape {
        version: 3,
        id,
        shape_bounds: rect(0.0, 0.0, 60.0, 60.0),
        edge_bounds: rect(0.0, 0.0, 60.0, 60.0),
        flags: ShapeFlag::empty(),
        styles: ShapeStyles {
            fill_styles: vec![FillStyle::Color(color)],
            line_styles: vec![],
        },
        shape,
    }
}

fn place(depth: Depth, id: CharacterId) -> Tag<'static> {
    Tag::PlaceObject(Box::new(PlaceObject {
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
    }))
}

fn replace(depth: Depth, id: CharacterId) -> Tag<'static> {
    let mut tag = place(depth, id);
    if let Tag::PlaceObject(place) = &mut tag {
        place.action = PlaceObjectAction::Replace(id);
        place.matrix = None;
    }
    tag
}

fn main() {
    let red = Color {
        r: 192,
        g: 32,
        b: 32,
        a: 255,
    };
    let blue = Color {
        r: 32,
        g: 64,
        b: 192,
        a: 255,
    };

    let mut tags: Vec<Tag> = vec![Tag::SetBackgroundColor(Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    })];
    tags.push(Tag::DefineShape(Box::new(notch_shape(1, false, red))));
    tags.push(Tag::DefineShape(Box::new(notch_shape(2, true, blue))));

    // Steady control: the first art, never replaced.
    let steady: Vec<Tag> = vec![place(1, 1), Tag::ShowFrame];
    tags.push(Tag::DefineSprite(Sprite {
        id: 10,
        num_frames: 1,
        tags: steady,
    }));

    // Swapped: art 1 on the first frame, replaced by art 2 from the second on. The long
    // tail of ShowFrames keeps the sprite on the replaced art for every later frame.
    let mut swapped: Vec<Tag> = vec![place(1, 1), Tag::ShowFrame, replace(1, 2)];
    swapped.extend((0..59).map(|_| Tag::ShowFrame));
    tags.push(Tag::DefineSprite(Sprite {
        id: 11,
        num_frames: 60,
        tags: swapped,
    }));

    tags.push(Tag::DefineScalingGrid {
        id: 10,
        splitter_rect: rect(20.0, 20.0, 40.0, 40.0),
    });
    tags.push(Tag::DefineScalingGrid {
        id: 11,
        splitter_rect: rect(20.0, 20.0, 40.0, 40.0),
    });

    // Rows three and four replace the placed character itself, and the grid rides the
    // character: art 3 carries a tag grid, art 4 does not. Losing the grid must stop
    // slicing and gaining one must start it, whichever art was on the first frame.
    tags.push(Tag::DefineShape(Box::new(notch_shape(3, false, red))));
    tags.push(Tag::DefineShape(Box::new(notch_shape(4, true, blue))));
    tags.push(Tag::DefineScalingGrid {
        id: 3,
        splitter_rect: rect(20.0, 20.0, 40.0, 40.0),
    });
    for (sprite_id, first, second) in [(12, 3, 4), (13, 4, 3)] {
        // The grid gates on the shape's own matrix, so the scale goes on the inner
        // placement, and Replace keeps it.
        let mut scaled = place(1, first);
        if let Tag::PlaceObject(p) = &mut scaled {
            p.matrix = Some(Matrix {
                a: Fixed16::from_f32(3.0),
                ..Matrix::IDENTITY
            });
        }
        let mut inner: Vec<Tag> = vec![scaled, Tag::ShowFrame, replace(1, second)];
        inner.extend((0..59).map(|_| Tag::ShowFrame));
        tags.push(Tag::DefineSprite(Sprite {
            id: sprite_id,
            num_frames: 60,
            tags: inner,
        }));
    }

    for (i, id) in [10, 11, 12, 13].into_iter().enumerate() {
        let mut tag = place(i as Depth + 1, id);
        if let Tag::PlaceObject(place) = &mut tag {
            let a = if id < 12 { 3.0 } else { 1.0 };
            place.matrix = Some(Matrix {
                a: Fixed16::from_f32(a),
                tx: px(20.0),
                ty: px(20.0 + i as f32 * 70.0),
                ..Matrix::IDENTITY
            });
        }
        tags.push(tag);
    }
    for _ in 0..60 {
        tags.push(Tag::ShowFrame);
    }

    let header = Header {
        compression: Compression::None,
        version: 8,
        stage_size: rect(0.0, 0.0, 220.0, 300.0),
        frame_rate: Fixed8::from_f32(24.0),
        num_frames: 60,
    };
    swf::write_swf(&header, &tags, File::create("test.swf").expect("create")).expect("write");
}
