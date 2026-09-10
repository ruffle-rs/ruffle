use std::fs::File;
use swf::avm1::types::{Action, Push, Value};
use swf::*;

const CASES: [(&str, Option<[f32; 4]>, bool); 4] = [
    ("b0", Some([20.0, 20.0, 20.0, 20.0]), false),
    ("b1", Some([20.0, 20.0, -10.0, 20.0]), false),
    ("b2", Some([20.0, 20.0, 20.0, 20.0]), true),
    ("b3", None, false),
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
            fill_styles: vec![FillStyle::Color(Color {
                r: 255,
                g: 0,
                b: 0,
                a: 255,
            })],
            line_styles: vec![],
        },
        shape,
    }
}

/// `trace(name + ": " + name.scale9Grid)`, exercising the getter after each case.
fn trace_grid(actions: &mut Vec<Action<'static>>, name: &'static str, label: &'static str) {
    actions.push(Action::Push(Push {
        values: vec![
            Value::Str(SwfStr::from_utf8_str(label)),
            Value::Str(SwfStr::from_utf8_str(name)),
        ],
    }));
    actions.push(Action::GetVariable);
    actions.push(Action::Push(Push {
        values: vec![Value::Str(SwfStr::from_utf8_str("scale9Grid"))],
    }));
    actions.push(Action::GetMember);
    actions.push(Action::Add2);
    actions.push(Action::Trace);
}

fn set_grid(actions: &mut Vec<Action<'static>>, name: &'static str, grid: Option<[f32; 4]>) {
    actions.push(Action::Push(Push {
        values: vec![Value::Str(SwfStr::from_utf8_str(name))],
    }));
    actions.push(Action::GetVariable);
    actions.push(Action::Push(Push {
        values: vec![Value::Str(SwfStr::from_utf8_str("scale9Grid"))],
    }));
    match grid {
        Some([x, y, w, h]) => {
            for (field, v) in [("x", x), ("y", y), ("width", w), ("height", h)] {
                actions.push(Action::Push(Push {
                    values: vec![
                        Value::Str(SwfStr::from_utf8_str(field)),
                        Value::Double(v as f64),
                    ],
                }));
            }
            actions.push(Action::Push(Push {
                values: vec![Value::Int(4)],
            }));
            actions.push(Action::InitObject);
        }
        None => actions.push(Action::Push(Push {
            values: vec![Value::Null],
        })),
    }
    actions.push(Action::SetMember);
}

fn main() {
    let mut tags: Vec<Tag> = vec![Tag::SetBackgroundColor(Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    })];
    let mut actions: Vec<Action> = Vec::new();

    for (i, &(name, grid, then_null)) in CASES.iter().enumerate() {
        let art = 100 + i as CharacterId;
        let button = 10 + i as CharacterId;
        tags.push(Tag::DefineShape(Box::new(notch_shape(art))));
        tags.push(Tag::DefineButton2(Box::new(Button {
            id: button,
            is_track_as_menu: false,
            records: vec![ButtonRecord {
                states: ButtonState::UP
                    | ButtonState::OVER
                    | ButtonState::DOWN
                    | ButtonState::HIT_TEST,
                id: art,
                depth: 1,
                matrix: Matrix::IDENTITY,
                color_transform: ColorTransform::IDENTITY,
                filters: vec![],
                blend_mode: BlendMode::Normal,
            }],
            actions: vec![],
        })));
        tags.push(Tag::PlaceObject(Box::new(PlaceObject {
            version: 2,
            action: PlaceObjectAction::Place(button),
            depth: i as Depth + 1,
            matrix: Some(Matrix {
                a: Fixed16::from_f32(3.0),
                tx: px(10.0),
                ty: px(10.0 + i as f32 * 70.0),
                ..Matrix::IDENTITY
            }),
            color_transform: None,
            ratio: None,
            name: Some(SwfStr::from_utf8_str(name)),
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
        })));
        if grid.is_some() {
            set_grid(&mut actions, name, grid);
        }
        if then_null {
            set_grid(&mut actions, name, None);
        }
    }

    // Getter round-trip: the accepted grid reads back, a refused or nulled one is undefined.
    for &(name, _, _) in CASES.iter() {
        let label: &'static str = match name {
            "b0" => "b0: ",
            "b1" => "b1: ",
            "b2" => "b2: ",
            _ => "b3: ",
        };
        trace_grid(&mut actions, name, label);
    }

    let mut body = Vec::new();
    let mut writer = swf::avm1::write::Writer::new(&mut body, 8);
    for action in &actions {
        writer.write_action(action).expect("action");
    }
    body.push(0);
    tags.push(Tag::DoAction(&body));
    tags.push(Tag::ShowFrame);

    let header = Header {
        compression: Compression::None,
        version: 8,
        stage_size: rect(0.0, 0.0, 200.0, 290.0),
        frame_rate: Fixed8::from_f32(24.0),
        num_frames: 1,
    };
    swf::write_swf(&header, &tags, File::create("test.swf").expect("create")).expect("write");
}
