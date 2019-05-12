pub fn calculate_shape_bounds(shape_records: &[swf::ShapeRecord]) -> swf::Rectangle {
    let mut bounds = swf::Rectangle {
        x_min: 999.0,
        x_max: -999.0,
        y_min: 999.0,
        y_max: -999.0,
    };
    let mut x = 0.0;
    let mut y = 0.0;
    for record in shape_records {
        match record {
            swf::ShapeRecord::StyleChange(style_change) => {
                if let Some((move_x, move_y)) = style_change.move_to {
                    x = move_x;
                    y = move_y;
                    bounds.x_min = f32::min(bounds.x_min, x);
                    bounds.x_max = f32::max(bounds.x_max, x);
                    bounds.y_min = f32::min(bounds.y_min, y);
                    bounds.y_max = f32::max(bounds.y_max, y);
                }
            }
            swf::ShapeRecord::StraightEdge { delta_x, delta_y } => {
                x += delta_x;
                y += delta_y;
                bounds.x_min = f32::min(bounds.x_min, x);
                bounds.x_max = f32::max(bounds.x_max, x);
                bounds.y_min = f32::min(bounds.y_min, y);
                bounds.y_max = f32::max(bounds.y_max, y);
            }
            swf::ShapeRecord::CurvedEdge {
                control_delta_x,
                control_delta_y,
                anchor_delta_x,
                anchor_delta_y,
            } => {
                x += control_delta_x;
                y += control_delta_y;
                bounds.x_min = f32::min(bounds.x_min, x);
                bounds.x_max = f32::max(bounds.x_max, x);
                bounds.y_min = f32::min(bounds.y_min, y);
                bounds.y_max = f32::max(bounds.y_max, y);
                x += anchor_delta_x;
                y += anchor_delta_y;
                bounds.x_min = f32::min(bounds.x_min, x);
                bounds.x_max = f32::max(bounds.x_max, x);
                bounds.y_min = f32::min(bounds.y_min, y);
                bounds.y_max = f32::max(bounds.y_max, y);
            }
        }
    }
    if bounds.x_max < bounds.x_min || bounds.y_max < bounds.y_min {
        bounds = swf::Rectangle {
            x_min: 0.0,
            x_max: 0.0,
            y_min: 0.0,
            y_max: 0.0,
        };
    }
    bounds
}
