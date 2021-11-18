//! `flash.display.Graphics` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{stage_allocator, Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::display_object::TDisplayObject;
use crate::shape_utils::DrawCommand;
use gc_arena::{GcCell, MutationContext};
use std::f64::consts::PI;
use swf::{Color, FillStyle, Fixed8, LineCapStyle, LineJoinStyle, LineStyle, Twips};

/// Implements `flash.display.Graphics`'s instance constructor.
pub fn instance_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Graphics cannot be constructed directly.".into())
}

/// Implements `flash.display.Graphics`'s native instance constructor.
pub fn native_instance_init<'gc>(
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
pub fn class_init<'gc>(
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
pub fn begin_fill<'gc>(
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
pub fn clear<'gc>(
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
pub fn curve_to<'gc>(
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
pub fn end_fill<'gc>(
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
    let caps_string = caps.coerce_to_string(activation);
    let caps_str = caps_string.as_deref();

    match (caps, caps_str) {
        (Value::Null, _) | (_, Ok("none")) => Ok(LineCapStyle::None),
        (_, Ok("round")) => Ok(LineCapStyle::Round),
        (_, Ok("square")) => Ok(LineCapStyle::Square),
        (_, Ok(_)) => Err("ArgumentError: caps is invalid".into()),
        (_, Err(_)) => Err(caps_string.unwrap_err()),
    }
}

fn joints_to_join_style<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    joints: Value<'gc>,
    miter_limit: f64,
) -> Result<LineJoinStyle, Error> {
    let joints_string = joints.coerce_to_string(activation);
    let joints_str = joints_string.as_deref();

    match (joints, joints_str) {
        (Value::Null, _) | (_, Ok("round")) => Ok(LineJoinStyle::Round),
        (_, Ok("miter")) => Ok(LineJoinStyle::Miter(Fixed8::from_f64(miter_limit))),
        (_, Ok("bevel")) => Ok(LineJoinStyle::Bevel),
        (_, Ok(_)) => Err("ArgumentError: joints is invalid".into()),
        (_, Err(_)) => Err(joints_string.unwrap_err()),
    }
}

fn scale_mode_to_allow_scale_bits(scale_mode: &str) -> Result<(bool, bool), Error> {
    match scale_mode {
        "normal" => Ok((true, true)),
        "none" => Ok((false, false)),
        "horizontal" => Ok((true, false)),
        "vertical" => Ok((false, true)),
        _ => Err("ArgumentError: scaleMode parameter is invalid".into()),
    }
}

/// Implements `Graphics.lineStyle`.
pub fn line_style<'gc>(
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

            let width = Twips::from_pixels(thickness.min(255.0).max(0.0));
            let color = color_from_args(color, alpha);
            let join_style = joints_to_join_style(activation, joints, miter_limit)?;
            let (allow_scale_x, allow_scale_y) = scale_mode_to_allow_scale_bits(&scale_mode)?;

            let line_style = LineStyle {
                width,
                color,
                start_cap: caps,
                end_cap: caps,
                join_style,
                fill_style: None,
                allow_scale_x,
                allow_scale_y,
                is_pixel_hinted,
                allow_close: true,
            };

            if let Some(mut draw) = this.as_drawing(activation.context.gc_context) {
                draw.set_line_style(Some(line_style));
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Graphics.lineTo`.
pub fn line_to<'gc>(
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
pub fn move_to<'gc>(
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
pub fn draw_rect<'gc>(
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

/// Implements `Graphics.drawRoundRect`.
pub fn draw_round_rect<'gc>(
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
        let mut ellipse_width = args
            .get(4)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_number(activation)?;
        let mut ellipse_height = args
            .get(5)
            .cloned()
            .unwrap_or(Value::Number(f64::NAN))
            .coerce_to_number(activation)?;

        if let Some(mut draw) = this.as_drawing(activation.context.gc_context) {
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

            // We always draw ellipses in 45-degree chunks, so let's
            // precalculate some common math. For efficiency, we'll work on a
            // unit circle at the origin and then translate and scale to get
            // four ellipse pieces on our roundrect. This lets us not have to
            // care about scaling our constants by a potentially varying
            // radius.
            //
            // We first consider the triangle formed by the control points of
            // our desired Bezier curve. We need to know the length of the
            // sides, which we don't know yet, but can derive from a second
            // triangle formed by the two control points on the circle and it's
            // center. The triangle in the circle shares a side with the Bezier
            // control point triangle, and that side just so happens to always
            // be the hypotenuse of the control points. Thanks to the law of
            // sines and some basic algebra, the length of this hypotenuse will
            // be the sine of our arc angle, divided by the sine of one of the
            // angles of the triangle in the circle, which is always 180 minus
            // half our angle.
            let h = (PI * 0.25).sin() / (PI * 0.375).sin();

            // Now we need the length of the other two sides of the control
            // point triangle. For reasons I'll explain shortly, both sides are
            // the same length, and the angles are related to the same angles
            // we used to calculate the hypotenuse.
            let a_b = h * (PI * 0.125).sin() / (PI * 0.75).sin();

            // Now that we know these lengths, we can start calculating the
            // actual control points. Since we're using quadratic Beziers, we
            // need two points on the circle, and one control point that we'll
            // calculate by intersecting the tangent lines at the two points
            // on the circle.
            //
            // Remember, we're still in unit circle mode.

            let unit_r_point_x = (PI * 0.0).cos();
            let unit_r_point_y = (PI * 0.0).sin();

            let unit_br_point_x = (PI * 0.25).cos();
            let unit_br_point_y = (PI * 0.25).sin();

            // Here's the actual fun part. We can calculate the tangent vector
            // of a point on the unit circle by swizzling it's coordinates, and
            // flipping the sign of the former Y coordinate. That tangent points
            // in the same direction as the line in our control point triangle,
            // so we just need to scale it by the length of that side of the
            // triangle and add it to the point. We now have our Bezier curve
            // point.

            let unit_r_br_curve_x = unit_r_point_x + a_b * unit_r_point_y * -1.0;
            let unit_r_br_curve_y = unit_r_point_y + a_b * unit_r_point_x;

            // We need another Bezier segment, of course...

            let unit_b_point_x = (PI * 0.5).cos();
            let unit_b_point_y = (PI * 0.5).sin();

            let unit_br_b_curve_x = unit_br_point_x + a_b * unit_br_point_y * -1.0;
            let unit_br_b_curve_y = unit_br_point_y + a_b * unit_br_point_x;

            // At this point we have 90 degrees of a circle in Bezier points.
            // The remaining points can be derived by coordinate flips,
            // folllowed by scaling to get an ellipse, and translation to the
            // actual ellipse centers. All the scary trig is over with.
            //
            // We'll start from the bottom-right corner of the rectangle,
            // because that's what Flash Player does.

            let line_width = width - ellipse_width;
            let line_height = height - ellipse_height;

            let br_ellipse_center_x = x + ellipse_width / 2.0 + line_width;
            let br_ellipse_center_y = y + ellipse_height / 2.0 + line_height;

            let br_point_x = br_ellipse_center_x + ellipse_width / 2.0 * unit_br_point_x;
            let br_point_y = br_ellipse_center_y + ellipse_height / 2.0 * unit_br_point_y;

            draw.draw_command(DrawCommand::MoveTo {
                x: Twips::from_pixels(br_point_x),
                y: Twips::from_pixels(br_point_y),
            });

            let br_b_curve_x = br_ellipse_center_x + ellipse_width / 2.0 * unit_br_b_curve_x;
            let br_b_curve_y = br_ellipse_center_y + ellipse_height / 2.0 * unit_br_b_curve_y;

            let right_b_point_x = br_ellipse_center_x + ellipse_width / 2.0 * unit_b_point_x;
            let right_b_point_y = br_ellipse_center_y + ellipse_height / 2.0 * unit_b_point_y;

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

            let left_b_point_x = tl_ellipse_center_x + ellipse_width / -2.0 * unit_b_point_x;
            let left_b_point_y = br_ellipse_center_y + ellipse_height / 2.0 * unit_b_point_y;

            draw.draw_command(DrawCommand::LineTo {
                x: Twips::from_pixels(left_b_point_x),
                y: Twips::from_pixels(left_b_point_y),
            });

            // Bottom-left ellipse
            let b_bl_curve_x = tl_ellipse_center_x + ellipse_width / -2.0 * unit_br_b_curve_x;
            let b_bl_curve_y = br_ellipse_center_y + ellipse_height / 2.0 * unit_br_b_curve_y;

            let bl_point_x = tl_ellipse_center_x + ellipse_width / -2.0 * unit_br_point_x;
            let bl_point_y = br_ellipse_center_y + ellipse_height / 2.0 * unit_br_point_y;

            draw.draw_command(DrawCommand::CurveTo {
                x1: Twips::from_pixels(b_bl_curve_x),
                y1: Twips::from_pixels(b_bl_curve_y),
                x2: Twips::from_pixels(bl_point_x),
                y2: Twips::from_pixels(bl_point_y),
            });

            let bl_l_curve_x = tl_ellipse_center_x + ellipse_width / -2.0 * unit_r_br_curve_x;
            let bl_l_curve_y = br_ellipse_center_y + ellipse_height / 2.0 * unit_r_br_curve_y;

            let bottom_l_point_x = tl_ellipse_center_x + ellipse_width / -2.0 * unit_r_point_x;
            let bottom_l_point_y = br_ellipse_center_y + ellipse_height / 2.0 * unit_r_point_y;

            draw.draw_command(DrawCommand::CurveTo {
                x1: Twips::from_pixels(bl_l_curve_x),
                y1: Twips::from_pixels(bl_l_curve_y),
                x2: Twips::from_pixels(bottom_l_point_x),
                y2: Twips::from_pixels(bottom_l_point_y),
            });

            // Left side
            let top_l_point_x = tl_ellipse_center_x + ellipse_width / -2.0 * unit_r_point_x;
            let top_l_point_y = tl_ellipse_center_y + ellipse_height / -2.0 * unit_r_point_y;

            draw.draw_command(DrawCommand::LineTo {
                x: Twips::from_pixels(top_l_point_x),
                y: Twips::from_pixels(top_l_point_y),
            });

            // Top-left ellipse
            let l_tl_curve_x = tl_ellipse_center_x + ellipse_width / -2.0 * unit_r_br_curve_x;
            let l_tl_curve_y = tl_ellipse_center_y + ellipse_height / -2.0 * unit_r_br_curve_y;

            let tl_point_x = tl_ellipse_center_x + ellipse_width / -2.0 * unit_br_point_x;
            let tl_point_y = tl_ellipse_center_y + ellipse_height / -2.0 * unit_br_point_y;

            draw.draw_command(DrawCommand::CurveTo {
                x1: Twips::from_pixels(l_tl_curve_x),
                y1: Twips::from_pixels(l_tl_curve_y),
                x2: Twips::from_pixels(tl_point_x),
                y2: Twips::from_pixels(tl_point_y),
            });

            let tl_t_curve_x = tl_ellipse_center_x + ellipse_width / -2.0 * unit_br_b_curve_x;
            let tl_t_curve_y = tl_ellipse_center_y + ellipse_height / -2.0 * unit_br_b_curve_y;

            let left_t_point_x = tl_ellipse_center_x + ellipse_width / -2.0 * unit_b_point_x;
            let left_t_point_y = tl_ellipse_center_y + ellipse_height / -2.0 * unit_b_point_y;

            draw.draw_command(DrawCommand::CurveTo {
                x1: Twips::from_pixels(tl_t_curve_x),
                y1: Twips::from_pixels(tl_t_curve_y),
                x2: Twips::from_pixels(left_t_point_x),
                y2: Twips::from_pixels(left_t_point_y),
            });

            // Top side
            let right_t_point_x = br_ellipse_center_x + ellipse_width / 2.0 * unit_b_point_x;
            let right_t_point_y = tl_ellipse_center_y + ellipse_height / -2.0 * unit_b_point_y;

            draw.draw_command(DrawCommand::LineTo {
                x: Twips::from_pixels(right_t_point_x),
                y: Twips::from_pixels(right_t_point_y),
            });

            // Top-right ellipse
            let t_tr_curve_x = br_ellipse_center_x + ellipse_width / 2.0 * unit_br_b_curve_x;
            let t_tr_curve_y = tl_ellipse_center_y + ellipse_height / -2.0 * unit_br_b_curve_y;

            let tr_point_x = br_ellipse_center_x + ellipse_width / 2.0 * unit_br_point_x;
            let tr_point_y = tl_ellipse_center_y + ellipse_height / -2.0 * unit_br_point_y;

            draw.draw_command(DrawCommand::CurveTo {
                x1: Twips::from_pixels(t_tr_curve_x),
                y1: Twips::from_pixels(t_tr_curve_y),
                x2: Twips::from_pixels(tr_point_x),
                y2: Twips::from_pixels(tr_point_y),
            });

            let tr_r_curve_x = br_ellipse_center_x + ellipse_width / 2.0 * unit_r_br_curve_x;
            let tr_r_curve_y = tl_ellipse_center_y + ellipse_height / -2.0 * unit_r_br_curve_y;

            let top_r_point_x = br_ellipse_center_x + ellipse_width / 2.0 * unit_r_point_x;
            let top_r_point_y = tl_ellipse_center_y + ellipse_height / -2.0 * unit_r_point_y;

            draw.draw_command(DrawCommand::CurveTo {
                x1: Twips::from_pixels(tr_r_curve_x),
                y1: Twips::from_pixels(tr_r_curve_y),
                x2: Twips::from_pixels(top_r_point_x),
                y2: Twips::from_pixels(top_r_point_y),
            });

            // Right side & other half of bottom-right ellipse
            let bottom_r_point_x = br_ellipse_center_x + ellipse_width / 2.0 * unit_r_point_x;
            let bottom_r_point_y = br_ellipse_center_y + ellipse_height / 2.0 * unit_r_point_y;

            draw.draw_command(DrawCommand::LineTo {
                x: Twips::from_pixels(bottom_r_point_x),
                y: Twips::from_pixels(bottom_r_point_y),
            });

            let r_br_curve_x = br_ellipse_center_x + ellipse_width / 2.0 * unit_r_br_curve_x;
            let r_br_curve_y = br_ellipse_center_y + ellipse_height / 2.0 * unit_r_br_curve_y;

            draw.draw_command(DrawCommand::CurveTo {
                x1: Twips::from_pixels(r_br_curve_x),
                y1: Twips::from_pixels(r_br_curve_y),
                x2: Twips::from_pixels(br_point_x),
                y2: Twips::from_pixels(br_point_y),
            });
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Graphics.drawCircle`.
pub fn draw_circle<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
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

    draw_round_rect(
        activation,
        this,
        &[
            (x - radius).into(),
            (y - radius).into(),
            (radius * 2.0).into(),
            (radius * 2.0).into(),
            (radius * 2.0).into(),
            (radius * 2.0).into(),
        ],
    )
}

/// Implements `Graphics.drawEllipse`.
pub fn draw_ellipse<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
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

    draw_round_rect(
        activation,
        this,
        &[
            x.into(),
            y.into(),
            width.into(),
            height.into(),
            width.into(),
            height.into(),
        ],
    )
}

/// Construct `Graphics`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.display"), "Graphics"),
        Some(QName::new(Namespace::public(), "Object").into()),
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
