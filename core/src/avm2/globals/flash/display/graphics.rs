//! `flash.display.Graphics` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::object::{stage_allocator, Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::Multiname;
use crate::avm2::Namespace;
use crate::avm2::QName;
use crate::display_object::TDisplayObject;
use crate::drawing::Drawing;
use crate::string::WStr;
use gc_arena::{GcCell, MutationContext};
use ruffle_render::shape_utils::DrawCommand;
use std::f64::consts::FRAC_1_SQRT_2;
use swf::{Color, FillStyle, Fixed8, LineCapStyle, LineJoinStyle, LineStyle, Twips};

/// Implements `flash.display.Graphics`'s instance constructor.
fn instance_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Graphics cannot be constructed directly.".into())
}

/// Implements `flash.display.Graphics`'s native instance constructor.
fn native_instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        activation.super_init(this, &[])?;
    }

    Ok(Value::Undefined)
}

/// Implements `flash.display.Graphics`'s class constructor.
fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Convert an RGB `color` and `alpha` argument pair into a `swf::Color`.
/// `alpha` is normalized from 0.0 - 1.0.
fn color_from_args(rgb: u32, alpha: f64) -> Color {
    Color::from_rgb(rgb, (alpha * 255.0) as u8)
}

/// Implements `Graphics.beginFill`.
fn begin_fill<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this.and_then(|t| t.as_display_object()) {
        let color = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_u32(activation)?;
        let alpha = args
            .get(1)
            .cloned()
            .unwrap_or_else(|| 1.0.into())
            .coerce_to_number(activation)?;

        if let Some(mut draw) = this.as_drawing(activation.context.gc_context) {
            draw.set_fill_style(Some(FillStyle::Color(color_from_args(color, alpha))));
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Graphics.clear`
fn clear<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this.and_then(|t| t.as_display_object()) {
        if let Some(mut draw) = this.as_drawing(activation.context.gc_context) {
            draw.clear()
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Graphics.curveTo`.
fn curve_to<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this.and_then(|t| t.as_display_object()) {
        let x1 = Twips::from_pixels(
            args.get(0)
                .cloned()
                .unwrap_or(Value::Undefined)
                .coerce_to_number(activation)?,
        );
        let y1 = Twips::from_pixels(
            args.get(1)
                .cloned()
                .unwrap_or(Value::Undefined)
                .coerce_to_number(activation)?,
        );
        let x2 = Twips::from_pixels(
            args.get(2)
                .cloned()
                .unwrap_or(Value::Undefined)
                .coerce_to_number(activation)?,
        );
        let y2 = Twips::from_pixels(
            args.get(3)
                .cloned()
                .unwrap_or(Value::Undefined)
                .coerce_to_number(activation)?,
        );

        if let Some(mut draw) = this.as_drawing(activation.context.gc_context) {
            draw.draw_command(DrawCommand::CurveTo { x1, y1, x2, y2 });
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Graphics.endFill`.
fn end_fill<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this.and_then(|t| t.as_display_object()) {
        if let Some(mut draw) = this.as_drawing(activation.context.gc_context) {
            draw.set_fill_style(None);
        }
    }

    Ok(Value::Undefined)
}

fn caps_to_cap_style<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    caps: Value<'gc>,
) -> Result<LineCapStyle, Error> {
    if let Value::Null = caps {
        return Ok(LineCapStyle::None);
    }

    let caps = caps.coerce_to_string(activation)?;
    if &caps == b"none" {
        Ok(LineCapStyle::None)
    } else if &caps == b"square" {
        Ok(LineCapStyle::Square)
    } else {
        Ok(LineCapStyle::Round)
    }
}

fn joints_to_join_style<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    joints: Value<'gc>,
    miter_limit: f64,
) -> Result<LineJoinStyle, Error> {
    if let Value::Null = joints {
        return Ok(LineJoinStyle::Round);
    }

    let joints = joints.coerce_to_string(activation)?;
    if &joints == b"miter" {
        Ok(LineJoinStyle::Miter(Fixed8::from_f64(miter_limit)))
    } else if &joints == b"bevel" {
        Ok(LineJoinStyle::Bevel)
    } else {
        Ok(LineJoinStyle::Round)
    }
}

fn scale_mode_to_allow_scale_bits(scale_mode: &WStr) -> Result<(bool, bool), Error> {
    if scale_mode == b"none" {
        Ok((false, false))
    } else if scale_mode == b"horizontal" {
        Ok((true, false))
    } else if scale_mode == b"vertical" {
        Ok((false, true))
    } else {
        Ok((true, true))
    }
}

/// Implements `Graphics.lineStyle`.
fn line_style<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this.and_then(|t| t.as_display_object()) {
        let thickness = args
            .get(0)
            .cloned()
            .unwrap_or_else(|| f64::NAN.into())
            .coerce_to_number(activation)?;

        if thickness.is_nan() {
            if let Some(mut draw) = this.as_drawing(activation.context.gc_context) {
                draw.set_line_style(None);
            }
        } else {
            let color = args
                .get(1)
                .cloned()
                .unwrap_or_else(|| 0.into())
                .coerce_to_u32(activation)?;
            let alpha = args
                .get(2)
                .cloned()
                .unwrap_or_else(|| 1.0.into())
                .coerce_to_number(activation)?;
            let is_pixel_hinted = args
                .get(3)
                .cloned()
                .unwrap_or_else(|| false.into())
                .coerce_to_boolean();
            let scale_mode = args
                .get(4)
                .cloned()
                .unwrap_or_else(|| "normal".into())
                .coerce_to_string(activation)?;
            let caps = caps_to_cap_style(activation, args.get(5).cloned().unwrap_or(Value::Null))?;
            let joints = args.get(6).cloned().unwrap_or(Value::Null);
            let miter_limit = args
                .get(7)
                .cloned()
                .unwrap_or_else(|| 3.0.into())
                .coerce_to_number(activation)?;

            let width = Twips::from_pixels(thickness.clamp(0.0, 255.0));
            let color = color_from_args(color, alpha);
            let join_style = joints_to_join_style(activation, joints, miter_limit)?;
            let (allow_scale_x, allow_scale_y) = scale_mode_to_allow_scale_bits(&scale_mode)?;

            let line_style = LineStyle::new()
                .with_width(width)
                .with_color(color)
                .with_start_cap(caps)
                .with_end_cap(caps)
                .with_join_style(join_style)
                .with_allow_scale_x(allow_scale_x)
                .with_allow_scale_y(allow_scale_y)
                .with_is_pixel_hinted(is_pixel_hinted)
                .with_allow_close(false);

            if let Some(mut draw) = this.as_drawing(activation.context.gc_context) {
                draw.set_line_style(Some(line_style));
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Graphics.lineTo`.
fn line_to<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this.and_then(|t| t.as_display_object()) {
        let x = Twips::from_pixels(
            args.get(0)
                .cloned()
                .unwrap_or(Value::Undefined)
                .coerce_to_number(activation)?,
        );
        let y = Twips::from_pixels(
            args.get(1)
                .cloned()
                .unwrap_or(Value::Undefined)
                .coerce_to_number(activation)?,
        );

        if let Some(mut draw) = this.as_drawing(activation.context.gc_context) {
            draw.draw_command(DrawCommand::LineTo { x, y });
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Graphics.moveTo`.
fn move_to<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this.and_then(|t| t.as_display_object()) {
        let x = Twips::from_pixels(
            args.get(0)
                .cloned()
                .unwrap_or(Value::Undefined)
                .coerce_to_number(activation)?,
        );
        let y = Twips::from_pixels(
            args.get(1)
                .cloned()
                .unwrap_or(Value::Undefined)
                .coerce_to_number(activation)?,
        );

        if let Some(mut draw) = this.as_drawing(activation.context.gc_context) {
            draw.draw_command(DrawCommand::MoveTo { x, y });
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Graphics.drawRect`.
fn draw_rect<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this.and_then(|t| t.as_display_object()) {
        let x = Twips::from_pixels(
            args.get(0)
                .cloned()
                .unwrap_or(Value::Undefined)
                .coerce_to_number(activation)?,
        );
        let y = Twips::from_pixels(
            args.get(1)
                .cloned()
                .unwrap_or(Value::Undefined)
                .coerce_to_number(activation)?,
        );
        let width = Twips::from_pixels(
            args.get(2)
                .cloned()
                .unwrap_or(Value::Undefined)
                .coerce_to_number(activation)?,
        );
        let height = Twips::from_pixels(
            args.get(3)
                .cloned()
                .unwrap_or(Value::Undefined)
                .coerce_to_number(activation)?,
        );

        if let Some(mut draw) = this.as_drawing(activation.context.gc_context) {
            draw.draw_command(DrawCommand::MoveTo { x, y });
            draw.draw_command(DrawCommand::LineTo { x: x + width, y });
            draw.draw_command(DrawCommand::LineTo {
                x: x + width,
                y: y + height,
            });
            draw.draw_command(DrawCommand::LineTo { x, y: y + height });
            draw.draw_command(DrawCommand::LineTo { x, y });
        }
    }

    Ok(Value::Undefined)
}

/// Length between two points on a unit circle that are 45 degrees apart from
/// one another.
///
/// This constant is `H`, short for 'hypotenuse', because it is also the length
/// of the hypotenuse formed from the control point triangle of any quadratic
/// Bezier curve approximating a 45-degree unit circle arc.
///
/// The derivation of this constant - or a similar constant for any other arc
/// angle hypotenuse - is as follows:
///
/// 1. Call the arc angle `alpha`. In this special case, `alpha` is 45 degrees,
///    or one-quarter `PI`.
/// 2. Consider the triangle formed by the center of the circle and the two
///    points at the start and end of the arc. The two other angles will be
///    equal, and it and `alpha` sum to 180 degrees. We'll call this angle
///    `beta`, and it is equal to `alpha` minus 180 degrees, divided by 2.
/// 3. Using the law of sines, we know that the sine of `alpha` divided by `H`
///    is equal to the sine of `beta` divided by `r`, where `r` is the radius
///    of the circle. We can solve for `H` to get the result. Note that since
///    this is a unit circle, you won't see a radius term in this constant.
//const H:f64 = (PI * 0.25).sin() / (PI * 0.375).sin();

/// Length between two control points of a quadratic Bezier curve approximating
/// a 45-degree arc of a unit circle.
///
/// This constant is critical to calculating the off-curve point of the control
/// point triangle. We do so by taking the tangents at each on-curve point,
/// which point in the direction of the off-curve points. Then, we scale one of
/// those tangent vectors by `A_B` and add it to the on-curve point to get the
/// off-curve point, constructing our Bezier.
///
/// The derivation of this constant - or a similar constant for any other arc
/// angle Bezier - is as follows:
///
/// 1. Start with the value of `H` for the given arc angle `alpha`.
/// 2. Consider the triangle formed by the three control points of our desired
///    Bezier curve. We'll call the angle at the off-curve control point
///    `delta`, and the two other angles of this triangle are `gamma`.
/// 3. Because two of the lines of this triangle are tangent lines of the
///    circle, they will form a right angle with the normal, which is the same
///    as the line between the center of the circle and the point.
///    Coincidentally, this right angle is shared between `beta`, meaning that
///    we can subtract it from 90 degrees to obtain `gamma`. Or, after some
///    elementary algebra, just take half of `alpha`.
/// 4. We can then derive the value of `delta` by subtracting out the other two
///    `gamma`s from 180 degrees. This, again, can be simplified to just
///    180 degrees minus `alpha`.
/// 5. By the law of sines, the sine of `delta` divided by `H` is equal to
///    the sine of `gamma` divided by `A_B`. We can then rearrange this to get
///    `H` times the sine of `gamma`, divided by the sine of `delta`; which is
///    our `A_B` constant.
//const A_B:f64 = H * (PI * 0.125).sin() / (PI * 0.75).sin();

/// A list of five quadratic Bezier control points, intended to approximate the
/// bottom-right quadrant of a unit circle.
///
/// Through coordinate reflections we can obtain the rest of the circle; and
/// with translations and scaling we can obtain any ellipse on the plane.
///
/// Points are stored in counter-clockwise order from 0 degrees to 90 degrees.
const UNIT_CIRCLE_POINTS: [(f64, f64); 5] = [
    (1.0, 0.0),
    (1.0, 0.41421356237309503),
    (FRAC_1_SQRT_2, FRAC_1_SQRT_2),
    (0.4142135623730951, 1.0),
    (0.00000000000000006123233995736766, 1.0),
];

/* [
    ((PI * 0.0).cos(), (PI * 0.0).sin()),
    ((PI * 0.0).cos() + *A_B * (PI * 0.0).sin() * -1.0,
    (PI * 0.0).sin() + *A_B * (PI * 0.0).cos()),
    ((PI * 0.25).cos(), (PI * 0.25).sin()),
    ((PI * 0.25).cos() + *A_B * (PI * 0.25).sin() * -1.0,
    (PI * 0.25).sin() + *A_B * (PI * 0.25).cos()),
    ((PI * 0.5).cos(), (PI * 0.5).sin()),
]; */

/// Draw a roundrect.
fn draw_round_rect_internal(
    draw: &mut Drawing,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    mut ellipse_width: f64,
    mut ellipse_height: f64,
) {
    if ellipse_height.is_nan() {
        ellipse_height = ellipse_width;
    }

    //Clamp the ellipse sizes to the size of the rectangle.
    if ellipse_width > width {
        ellipse_width = width;
    }

    if ellipse_height > height {
        ellipse_height = height;
    }

    // We'll start from the bottom-right corner of the rectangle,
    // because that's what Flash Player does.
    let ucp = UNIT_CIRCLE_POINTS;

    let line_width = width - ellipse_width;
    let line_height = height - ellipse_height;

    let br_ellipse_center_x = x + ellipse_width / 2.0 + line_width;
    let br_ellipse_center_y = y + ellipse_height / 2.0 + line_height;

    let br_point_x = br_ellipse_center_x + ellipse_width / 2.0 * ucp[2].0;
    let br_point_y = br_ellipse_center_y + ellipse_height / 2.0 * ucp[2].1;

    draw.draw_command(DrawCommand::MoveTo {
        x: Twips::from_pixels(br_point_x),
        y: Twips::from_pixels(br_point_y),
    });

    let br_b_curve_x = br_ellipse_center_x + ellipse_width / 2.0 * ucp[3].0;
    let br_b_curve_y = br_ellipse_center_y + ellipse_height / 2.0 * ucp[3].1;

    let right_b_point_x = br_ellipse_center_x + ellipse_width / 2.0 * ucp[4].0;
    let right_b_point_y = br_ellipse_center_y + ellipse_height / 2.0 * ucp[4].1;

    draw.draw_command(DrawCommand::CurveTo {
        x1: Twips::from_pixels(br_b_curve_x),
        y1: Twips::from_pixels(br_b_curve_y),
        x2: Twips::from_pixels(right_b_point_x),
        y2: Twips::from_pixels(right_b_point_y),
    });

    // Oh, since we're drawing roundrects, we also need to draw lines
    // in between each ellipse. This is the bottom line.
    let tl_ellipse_center_x = x + ellipse_width / 2.0;
    let tl_ellipse_center_y = y + ellipse_height / 2.0;

    let left_b_point_x = tl_ellipse_center_x + ellipse_width / -2.0 * ucp[4].0;
    let left_b_point_y = br_ellipse_center_y + ellipse_height / 2.0 * ucp[4].1;

    draw.draw_command(DrawCommand::LineTo {
        x: Twips::from_pixels(left_b_point_x),
        y: Twips::from_pixels(left_b_point_y),
    });

    // Bottom-left ellipse
    let b_bl_curve_x = tl_ellipse_center_x + ellipse_width / -2.0 * ucp[3].0;
    let b_bl_curve_y = br_ellipse_center_y + ellipse_height / 2.0 * ucp[3].1;

    let bl_point_x = tl_ellipse_center_x + ellipse_width / -2.0 * ucp[2].0;
    let bl_point_y = br_ellipse_center_y + ellipse_height / 2.0 * ucp[2].1;

    draw.draw_command(DrawCommand::CurveTo {
        x1: Twips::from_pixels(b_bl_curve_x),
        y1: Twips::from_pixels(b_bl_curve_y),
        x2: Twips::from_pixels(bl_point_x),
        y2: Twips::from_pixels(bl_point_y),
    });

    let bl_l_curve_x = tl_ellipse_center_x + ellipse_width / -2.0 * ucp[1].0;
    let bl_l_curve_y = br_ellipse_center_y + ellipse_height / 2.0 * ucp[1].1;

    let bottom_l_point_x = tl_ellipse_center_x + ellipse_width / -2.0 * ucp[0].0;
    let bottom_l_point_y = br_ellipse_center_y + ellipse_height / 2.0 * ucp[0].1;

    draw.draw_command(DrawCommand::CurveTo {
        x1: Twips::from_pixels(bl_l_curve_x),
        y1: Twips::from_pixels(bl_l_curve_y),
        x2: Twips::from_pixels(bottom_l_point_x),
        y2: Twips::from_pixels(bottom_l_point_y),
    });

    // Left side
    let top_l_point_x = tl_ellipse_center_x + ellipse_width / -2.0 * ucp[0].0;
    let top_l_point_y = tl_ellipse_center_y + ellipse_height / -2.0 * ucp[0].1;

    draw.draw_command(DrawCommand::LineTo {
        x: Twips::from_pixels(top_l_point_x),
        y: Twips::from_pixels(top_l_point_y),
    });

    // Top-left ellipse
    let l_tl_curve_x = tl_ellipse_center_x + ellipse_width / -2.0 * ucp[1].0;
    let l_tl_curve_y = tl_ellipse_center_y + ellipse_height / -2.0 * ucp[1].1;

    let tl_point_x = tl_ellipse_center_x + ellipse_width / -2.0 * ucp[2].0;
    let tl_point_y = tl_ellipse_center_y + ellipse_height / -2.0 * ucp[2].1;

    draw.draw_command(DrawCommand::CurveTo {
        x1: Twips::from_pixels(l_tl_curve_x),
        y1: Twips::from_pixels(l_tl_curve_y),
        x2: Twips::from_pixels(tl_point_x),
        y2: Twips::from_pixels(tl_point_y),
    });

    let tl_t_curve_x = tl_ellipse_center_x + ellipse_width / -2.0 * ucp[3].0;
    let tl_t_curve_y = tl_ellipse_center_y + ellipse_height / -2.0 * ucp[3].1;

    let left_t_point_x = tl_ellipse_center_x + ellipse_width / -2.0 * ucp[4].0;
    let left_t_point_y = tl_ellipse_center_y + ellipse_height / -2.0 * ucp[4].1;

    draw.draw_command(DrawCommand::CurveTo {
        x1: Twips::from_pixels(tl_t_curve_x),
        y1: Twips::from_pixels(tl_t_curve_y),
        x2: Twips::from_pixels(left_t_point_x),
        y2: Twips::from_pixels(left_t_point_y),
    });

    // Top side
    let right_t_point_x = br_ellipse_center_x + ellipse_width / 2.0 * ucp[4].0;
    let right_t_point_y = tl_ellipse_center_y + ellipse_height / -2.0 * ucp[4].1;

    draw.draw_command(DrawCommand::LineTo {
        x: Twips::from_pixels(right_t_point_x),
        y: Twips::from_pixels(right_t_point_y),
    });

    // Top-right ellipse
    let t_tr_curve_x = br_ellipse_center_x + ellipse_width / 2.0 * ucp[3].0;
    let t_tr_curve_y = tl_ellipse_center_y + ellipse_height / -2.0 * ucp[3].1;

    let tr_point_x = br_ellipse_center_x + ellipse_width / 2.0 * ucp[2].0;
    let tr_point_y = tl_ellipse_center_y + ellipse_height / -2.0 * ucp[2].1;

    draw.draw_command(DrawCommand::CurveTo {
        x1: Twips::from_pixels(t_tr_curve_x),
        y1: Twips::from_pixels(t_tr_curve_y),
        x2: Twips::from_pixels(tr_point_x),
        y2: Twips::from_pixels(tr_point_y),
    });

    let tr_r_curve_x = br_ellipse_center_x + ellipse_width / 2.0 * ucp[1].0;
    let tr_r_curve_y = tl_ellipse_center_y + ellipse_height / -2.0 * ucp[1].1;

    let top_r_point_x = br_ellipse_center_x + ellipse_width / 2.0 * ucp[0].0;
    let top_r_point_y = tl_ellipse_center_y + ellipse_height / -2.0 * ucp[0].1;

    draw.draw_command(DrawCommand::CurveTo {
        x1: Twips::from_pixels(tr_r_curve_x),
        y1: Twips::from_pixels(tr_r_curve_y),
        x2: Twips::from_pixels(top_r_point_x),
        y2: Twips::from_pixels(top_r_point_y),
    });

    // Right side & other half of bottom-right ellipse
    let bottom_r_point_x = br_ellipse_center_x + ellipse_width / 2.0 * ucp[0].0;
    let bottom_r_point_y = br_ellipse_center_y + ellipse_height / 2.0 * ucp[0].1;

    draw.draw_command(DrawCommand::LineTo {
        x: Twips::from_pixels(bottom_r_point_x),
        y: Twips::from_pixels(bottom_r_point_y),
    });

    let r_br_curve_x = br_ellipse_center_x + ellipse_width / 2.0 * ucp[1].0;
    let r_br_curve_y = br_ellipse_center_y + ellipse_height / 2.0 * ucp[1].1;

    draw.draw_command(DrawCommand::CurveTo {
        x1: Twips::from_pixels(r_br_curve_x),
        y1: Twips::from_pixels(r_br_curve_y),
        x2: Twips::from_pixels(br_point_x),
        y2: Twips::from_pixels(br_point_y),
    });
}

/// Implements `Graphics.drawRoundRect`.
fn draw_round_rect<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this.and_then(|t| t.as_display_object()) {
        let x = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_number(activation)?;
        let y = args
            .get(1)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_number(activation)?;
        let width = args
            .get(2)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_number(activation)?;
        let height = args
            .get(3)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_number(activation)?;
        let ellipse_width = args
            .get(4)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_number(activation)?;
        let ellipse_height = args
            .get(5)
            .cloned()
            .unwrap_or(Value::Number(f64::NAN))
            .coerce_to_number(activation)?;

        if let Some(mut draw) = this.as_drawing(activation.context.gc_context) {
            draw_round_rect_internal(
                &mut draw,
                x,
                y,
                width,
                height,
                ellipse_width,
                ellipse_height,
            );
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Graphics.drawCircle`.
fn draw_circle<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this.and_then(|t| t.as_display_object()) {
        let x = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_number(activation)?;
        let y = args
            .get(1)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_number(activation)?;
        let radius = args
            .get(2)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_number(activation)?;

        if let Some(mut draw) = this.as_drawing(activation.context.gc_context) {
            draw_round_rect_internal(
                &mut draw,
                x - radius,
                y - radius,
                radius * 2.0,
                radius * 2.0,
                radius * 2.0,
                radius * 2.0,
            );
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Graphics.drawEllipse`.
fn draw_ellipse<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this.and_then(|t| t.as_display_object()) {
        let x = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_number(activation)?;
        let y = args
            .get(1)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_number(activation)?;
        let width = args
            .get(2)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_number(activation)?;
        let height = args
            .get(3)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_number(activation)?;

        if let Some(mut draw) = this.as_drawing(activation.context.gc_context) {
            draw_round_rect_internal(&mut draw, x, y, width, height, width, height)
        }
    }

    Ok(Value::Undefined)
}

/// Construct `Graphics`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.display"), "Graphics"),
        Some(Multiname::public("Object")),
        Method::from_builtin(instance_init, "<Graphics instance initializer>", mc),
        Method::from_builtin(class_init, "<Graphics class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    write.set_attributes(ClassAttributes::SEALED);
    write.set_instance_allocator(stage_allocator);
    write.set_native_instance_init(Method::from_builtin(
        native_instance_init,
        "<Graphics native instance initializer>",
        mc,
    ));

    const PUBLIC_INSTANCE_METHODS: &[(&str, NativeMethodImpl)] = &[
        ("beginFill", begin_fill),
        ("clear", clear),
        ("curveTo", curve_to),
        ("endFill", end_fill),
        ("lineStyle", line_style),
        ("lineTo", line_to),
        ("moveTo", move_to),
        ("drawRect", draw_rect),
        ("drawRoundRect", draw_round_rect),
        ("drawCircle", draw_circle),
        ("drawEllipse", draw_ellipse),
    ];
    write.define_public_builtin_instance_methods(mc, PUBLIC_INSTANCE_METHODS);

    class
}
