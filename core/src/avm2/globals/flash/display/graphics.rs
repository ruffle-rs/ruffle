//! `flash.display.Graphics` builtin/prototype

// See: https://github.com/rust-lang/rust-clippy/issues/12917
#![allow(clippy::doc_lazy_continuation)]

use crate::avm2::activation::Activation;
use crate::avm2::error::{make_error_2004, make_error_2008, Error2004Type};
use crate::avm2::globals::flash::geom::transform::object_to_matrix;
use crate::avm2::object::{Object, TObject, VectorObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::vector::VectorStorage;
use crate::avm2::{ArrayStorage, Error};
use crate::avm2_stub_method;
use crate::display_object::TDisplayObject;
use crate::drawing::Drawing;
use crate::string::{AvmString, WStr};
use ruffle_render::shape_utils::{DrawCommand, FillRule, GradientType};
use std::f64::consts::FRAC_1_SQRT_2;
use swf::{
    Color, FillStyle, Fixed16, Fixed8, Gradient, GradientInterpolation, GradientRecord,
    GradientSpread, LineCapStyle, LineJoinStyle, LineStyle, Matrix, Point, Twips,
};

/// Convert an RGB `color` and `alpha` argument pair into a `swf::Color`.
/// `alpha` is normalized from 0.0 - 1.0.
fn color_from_args(rgb: u32, alpha: f64) -> Color {
    Color::from_rgb(rgb, (alpha * 255.0) as u8)
}

/// Implements `Graphics.beginFill`.
pub fn begin_fill<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_display_object() {
        let color = args.get_u32(activation, 0)?;
        let alpha = args.get_f64(activation, 1)?;

        if let Some(mut draw) = this.as_drawing(activation.context.gc_context) {
            draw.set_fill_style(Some(FillStyle::Color(color_from_args(color, alpha))));
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Graphics.beginBitmapFill`.
pub fn begin_bitmap_fill<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_display_object() {
        let bitmap = args
            .get_object(activation, 0, "bitmap")?
            .as_bitmap_data()
            .expect("Bitmap argument is ensured to be a BitmapData from actionscript");
        let matrix = if let Some(matrix) = args.try_get_object(activation, 1) {
            Matrix::from(object_to_matrix(matrix, activation)?)
        } else {
            // Users can explicitly pass in `null` to mean identity matrix
            Matrix::IDENTITY
        };
        let is_repeating = args.get_bool(2);
        let is_smoothed = args.get_bool(3);

        let handle =
            bitmap.bitmap_handle(activation.context.gc_context, activation.context.renderer);

        let bitmap = ruffle_render::bitmap::BitmapInfo {
            handle,
            width: bitmap.width() as u16,
            height: bitmap.height() as u16,
        };
        let scale_matrix = Matrix::scale(
            (Twips::TWIPS_PER_PIXEL as i16).into(),
            (Twips::TWIPS_PER_PIXEL as i16).into(),
        );

        if let Some(mut draw) = this.as_drawing(activation.context.gc_context) {
            let id = draw.add_bitmap(bitmap);
            draw.set_fill_style(Some(FillStyle::Bitmap {
                id,
                matrix: matrix * scale_matrix,
                is_smoothed,
                is_repeating,
            }));
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Graphics.beginGradientFill`.
pub fn begin_gradient_fill<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_display_object() {
        let gradient_type = args.get_string(activation, 0)?;
        let gradient_type = parse_gradient_type(activation, gradient_type)?;
        let colors = args.get_object(activation, 1, "colors")?;
        let alphas = args.get_object(activation, 2, "alphas")?;
        let ratios = args.get_object(activation, 3, "ratios")?;
        let records = build_gradient_records(
            activation,
            &colors.as_array_storage().expect("Guaranteed by AS"),
            &alphas.as_array_storage().expect("Guaranteed by AS"),
            &ratios.as_array_storage().expect("Guaranteed by AS"),
        )?;
        let matrix = if let Some(matrix) = args.try_get_object(activation, 4) {
            Matrix::from(object_to_matrix(matrix, activation)?)
        } else {
            // Users can explicitly pass in `null` to mean identity matrix
            Matrix::IDENTITY
        };
        let spread = args.get_string(activation, 5);
        let spread = parse_spread_method(spread?);
        let interpolation = args.get_string(activation, 6);
        let interpolation = parse_interpolation_method(interpolation?);
        let focal_point = args.get_f64(activation, 7)?;

        if let Some(mut draw) = this.as_drawing(activation.context.gc_context) {
            match gradient_type {
                GradientType::Linear => {
                    draw.set_fill_style(Some(FillStyle::LinearGradient(Gradient {
                        matrix,
                        spread,
                        interpolation,
                        records,
                    })))
                }
                GradientType::Radial if focal_point == 0.0 => {
                    draw.set_fill_style(Some(FillStyle::RadialGradient(Gradient {
                        matrix,
                        spread,
                        interpolation,
                        records,
                    })))
                }
                _ => draw.set_fill_style(Some(FillStyle::FocalGradient {
                    gradient: Gradient {
                        matrix,
                        spread,
                        interpolation,
                        records,
                    },
                    focal_point: Fixed8::from_f64(focal_point),
                })),
            }
        }
    }
    Ok(Value::Undefined)
}

fn build_gradient_records<'gc>(
    activation: &mut Activation<'_, 'gc>,
    colors: &ArrayStorage<'gc>,
    alphas: &ArrayStorage<'gc>,
    ratios: &ArrayStorage<'gc>,
) -> Result<Vec<GradientRecord>, Error<'gc>> {
    let length = colors.length().min(alphas.length()).min(ratios.length());
    let mut records = Vec::with_capacity(length);
    for i in 0..length {
        let color = colors
            .get(i)
            .expect("Length should be guaranteed")
            .coerce_to_u32(activation)?;
        let alpha = alphas
            .get(i)
            .expect("Length should be guaranteed")
            .coerce_to_number(activation)? as f32;
        let ratio = ratios
            .get(i)
            .expect("Length should be guaranteed")
            .coerce_to_u32(activation)?;
        records.push(GradientRecord {
            ratio: ratio.clamp(0, 255) as u8,
            color: Color::from_rgb(color, (alpha * 255.0) as u8),
        })
    }
    Ok(records)
}

fn parse_gradient_type<'gc>(
    activation: &mut Activation<'_, 'gc>,
    gradient_type: AvmString<'gc>,
) -> Result<GradientType, Error<'gc>> {
    if &gradient_type == b"linear" {
        Ok(GradientType::Linear)
    } else if &gradient_type == b"radial" {
        Ok(GradientType::Radial)
    } else {
        Err(make_error_2008(activation, "type"))
    }
}

fn parse_interpolation_method(gradient_type: AvmString) -> GradientInterpolation {
    if &gradient_type == b"linearRGB" {
        GradientInterpolation::LinearRgb
    } else {
        GradientInterpolation::Rgb
    }
}

fn parse_spread_method(spread_method: AvmString) -> GradientSpread {
    if &spread_method == b"repeat" {
        GradientSpread::Repeat
    } else if &spread_method == b"reflect" {
        GradientSpread::Reflect
    } else {
        GradientSpread::Pad
    }
}

/// Implements `Graphics.clear`
pub fn clear<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_display_object() {
        if let Some(mut draw) = this.as_drawing(activation.context.gc_context) {
            draw.clear()
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Graphics.curveTo`.
pub fn curve_to<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_display_object() {
        let control_x = args.get_f64(activation, 0)?;
        let control_y = args.get_f64(activation, 1)?;
        let anchor_x = args.get_f64(activation, 2)?;
        let anchor_y = args.get_f64(activation, 3)?;

        if let Some(mut draw) = this.as_drawing(activation.context.gc_context) {
            draw.draw_command(DrawCommand::QuadraticCurveTo {
                control: Point::from_pixels(control_x, control_y),
                anchor: Point::from_pixels(anchor_x, anchor_y),
            });
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Graphics.endFill`.
pub fn end_fill<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_display_object() {
        if let Some(mut draw) = this.as_drawing(activation.context.gc_context) {
            draw.set_fill_style(None);
        }
    }

    Ok(Value::Undefined)
}

fn caps_to_cap_style(caps: Option<AvmString>) -> LineCapStyle {
    if let Some(caps) = caps {
        if &caps == b"none" {
            LineCapStyle::None
        } else if &caps == b"square" {
            LineCapStyle::Square
        } else {
            LineCapStyle::Round
        }
    } else {
        LineCapStyle::Round
    }
}

fn joints_to_join_style(joints: Option<AvmString>, miter_limit: f64) -> LineJoinStyle {
    if let Some(joints) = joints {
        if &joints == b"miter" {
            LineJoinStyle::Miter(Fixed8::from_f64(miter_limit))
        } else if &joints == b"bevel" {
            LineJoinStyle::Bevel
        } else {
            LineJoinStyle::Round
        }
    } else {
        LineJoinStyle::Round
    }
}

fn scale_mode_to_allow_scale_bits<'gc>(scale_mode: &WStr) -> Result<(bool, bool), Error<'gc>> {
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
pub fn line_style<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_display_object() {
        let thickness = args.get_f64(activation, 0)?;

        if thickness.is_nan() {
            if let Some(mut draw) = this.as_drawing(activation.context.gc_context) {
                draw.set_line_style(None);
            }
        } else {
            let color = args.get_u32(activation, 1)?;
            let alpha = args.get_f64(activation, 2)?;
            let is_pixel_hinted = args.get_bool(3);
            let scale_mode = args.get_string(activation, 4)?;
            let caps = caps_to_cap_style(args.try_get_string(activation, 5)?);
            let joints = args.try_get_string(activation, 6)?;
            let miter_limit = args.get_f64(activation, 7)?;

            let width = Twips::from_pixels(thickness.clamp(0.0, 255.0));
            let color = color_from_args(color, alpha);
            let join_style = joints_to_join_style(joints, miter_limit);
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
pub fn line_to<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_display_object() {
        let x = Twips::from_pixels(args.get_f64(activation, 0)?);
        let y = Twips::from_pixels(args.get_f64(activation, 1)?);

        if let Some(mut draw) = this.as_drawing(activation.context.gc_context) {
            draw.draw_command(DrawCommand::LineTo(Point::new(x, y)));
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Graphics.moveTo`.
pub fn move_to<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_display_object() {
        let x = Twips::from_pixels(args.get_f64(activation, 0)?);
        let y = Twips::from_pixels(args.get_f64(activation, 1)?);

        if let Some(mut draw) = this.as_drawing(activation.context.gc_context) {
            draw.draw_command(DrawCommand::MoveTo(Point::new(x, y)));
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Graphics.drawRect`.
pub fn draw_rect<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_display_object() {
        let x = Twips::from_pixels(args.get_f64(activation, 0)?);
        let y = Twips::from_pixels(args.get_f64(activation, 1)?);
        let width = Twips::from_pixels(args.get_f64(activation, 2)?);
        let height = Twips::from_pixels(args.get_f64(activation, 3)?);

        if let Some(mut draw) = this.as_drawing(activation.context.gc_context) {
            draw.draw_command(DrawCommand::MoveTo(Point::new(x, y)));
            draw.draw_command(DrawCommand::LineTo(Point::new(x + width, y)));
            draw.draw_command(DrawCommand::LineTo(Point::new(x + width, y + height)));
            draw.draw_command(DrawCommand::LineTo(Point::new(x, y + height)));
            draw.draw_command(DrawCommand::LineTo(Point::new(x, y)));
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
    let br_point = Point::from_pixels(br_point_x, br_point_y);

    draw.draw_command(DrawCommand::MoveTo(br_point));

    let br_b_curve_x = br_ellipse_center_x + ellipse_width / 2.0 * ucp[3].0;
    let br_b_curve_y = br_ellipse_center_y + ellipse_height / 2.0 * ucp[3].1;
    let br_b_curve = Point::from_pixels(br_b_curve_x, br_b_curve_y);

    let right_b_point_x = br_ellipse_center_x + ellipse_width / 2.0 * ucp[4].0;
    let right_b_point_y = br_ellipse_center_y + ellipse_height / 2.0 * ucp[4].1;
    let right_b_point = Point::from_pixels(right_b_point_x, right_b_point_y);

    draw.draw_command(DrawCommand::QuadraticCurveTo {
        control: br_b_curve,
        anchor: right_b_point,
    });

    // Oh, since we're drawing roundrects, we also need to draw lines
    // in between each ellipse. This is the bottom line.
    let tl_ellipse_center_x = x + ellipse_width / 2.0;
    let tl_ellipse_center_y = y + ellipse_height / 2.0;

    let left_b_point_x = tl_ellipse_center_x + ellipse_width / -2.0 * ucp[4].0;
    let left_b_point_y = br_ellipse_center_y + ellipse_height / 2.0 * ucp[4].1;
    let left_b_point = Point::from_pixels(left_b_point_x, left_b_point_y);

    draw.draw_command(DrawCommand::LineTo(left_b_point));

    // Bottom-left ellipse
    let b_bl_curve_x = tl_ellipse_center_x + ellipse_width / -2.0 * ucp[3].0;
    let b_bl_curve_y = br_ellipse_center_y + ellipse_height / 2.0 * ucp[3].1;
    let b_bl_curve = Point::from_pixels(b_bl_curve_x, b_bl_curve_y);

    let bl_point_x = tl_ellipse_center_x + ellipse_width / -2.0 * ucp[2].0;
    let bl_point_y = br_ellipse_center_y + ellipse_height / 2.0 * ucp[2].1;
    let bl_point = Point::from_pixels(bl_point_x, bl_point_y);

    draw.draw_command(DrawCommand::QuadraticCurveTo {
        control: b_bl_curve,
        anchor: bl_point,
    });

    let bl_l_curve_x = tl_ellipse_center_x + ellipse_width / -2.0 * ucp[1].0;
    let bl_l_curve_y = br_ellipse_center_y + ellipse_height / 2.0 * ucp[1].1;
    let bl_l_curve = Point::from_pixels(bl_l_curve_x, bl_l_curve_y);

    let bottom_l_point_x = tl_ellipse_center_x + ellipse_width / -2.0 * ucp[0].0;
    let bottom_l_point_y = br_ellipse_center_y + ellipse_height / 2.0 * ucp[0].1;
    let bottom_l_point = Point::from_pixels(bottom_l_point_x, bottom_l_point_y);

    draw.draw_command(DrawCommand::QuadraticCurveTo {
        control: bl_l_curve,
        anchor: bottom_l_point,
    });

    // Left side
    let top_l_point_x = tl_ellipse_center_x + ellipse_width / -2.0 * ucp[0].0;
    let top_l_point_y = tl_ellipse_center_y + ellipse_height / -2.0 * ucp[0].1;
    let top_l_point = Point::from_pixels(top_l_point_x, top_l_point_y);

    draw.draw_command(DrawCommand::LineTo(top_l_point));

    // Top-left ellipse
    let l_tl_curve_x = tl_ellipse_center_x + ellipse_width / -2.0 * ucp[1].0;
    let l_tl_curve_y = tl_ellipse_center_y + ellipse_height / -2.0 * ucp[1].1;
    let l_tl_curve = Point::from_pixels(l_tl_curve_x, l_tl_curve_y);

    let tl_point_x = tl_ellipse_center_x + ellipse_width / -2.0 * ucp[2].0;
    let tl_point_y = tl_ellipse_center_y + ellipse_height / -2.0 * ucp[2].1;
    let tl_point = Point::from_pixels(tl_point_x, tl_point_y);

    draw.draw_command(DrawCommand::QuadraticCurveTo {
        control: l_tl_curve,
        anchor: tl_point,
    });

    let tl_t_curve_x = tl_ellipse_center_x + ellipse_width / -2.0 * ucp[3].0;
    let tl_t_curve_y = tl_ellipse_center_y + ellipse_height / -2.0 * ucp[3].1;
    let tl_t_curve = Point::from_pixels(tl_t_curve_x, tl_t_curve_y);

    let left_t_point_x = tl_ellipse_center_x + ellipse_width / -2.0 * ucp[4].0;
    let left_t_point_y = tl_ellipse_center_y + ellipse_height / -2.0 * ucp[4].1;
    let left_t_point = Point::from_pixels(left_t_point_x, left_t_point_y);

    draw.draw_command(DrawCommand::QuadraticCurveTo {
        control: tl_t_curve,
        anchor: left_t_point,
    });

    // Top side
    let right_t_point_x = br_ellipse_center_x + ellipse_width / 2.0 * ucp[4].0;
    let right_t_point_y = tl_ellipse_center_y + ellipse_height / -2.0 * ucp[4].1;
    let right_t_point = Point::from_pixels(right_t_point_x, right_t_point_y);

    draw.draw_command(DrawCommand::LineTo(right_t_point));

    // Top-right ellipse
    let t_tr_curve_x = br_ellipse_center_x + ellipse_width / 2.0 * ucp[3].0;
    let t_tr_curve_y = tl_ellipse_center_y + ellipse_height / -2.0 * ucp[3].1;
    let t_tr_curve = Point::from_pixels(t_tr_curve_x, t_tr_curve_y);

    let tr_point_x = br_ellipse_center_x + ellipse_width / 2.0 * ucp[2].0;
    let tr_point_y = tl_ellipse_center_y + ellipse_height / -2.0 * ucp[2].1;
    let tr_point = Point::from_pixels(tr_point_x, tr_point_y);

    draw.draw_command(DrawCommand::QuadraticCurveTo {
        control: t_tr_curve,
        anchor: tr_point,
    });

    let tr_r_curve_x = br_ellipse_center_x + ellipse_width / 2.0 * ucp[1].0;
    let tr_r_curve_y = tl_ellipse_center_y + ellipse_height / -2.0 * ucp[1].1;
    let tr_r_curve = Point::from_pixels(tr_r_curve_x, tr_r_curve_y);

    let top_r_point_x = br_ellipse_center_x + ellipse_width / 2.0 * ucp[0].0;
    let top_r_point_y = tl_ellipse_center_y + ellipse_height / -2.0 * ucp[0].1;
    let top_r_point = Point::from_pixels(top_r_point_x, top_r_point_y);

    draw.draw_command(DrawCommand::QuadraticCurveTo {
        control: tr_r_curve,
        anchor: top_r_point,
    });

    // Right side & other half of bottom-right ellipse
    let bottom_r_point_x = br_ellipse_center_x + ellipse_width / 2.0 * ucp[0].0;
    let bottom_r_point_y = br_ellipse_center_y + ellipse_height / 2.0 * ucp[0].1;
    let bottom_r_point = Point::from_pixels(bottom_r_point_x, bottom_r_point_y);

    draw.draw_command(DrawCommand::LineTo(bottom_r_point));

    let r_br_curve_x = br_ellipse_center_x + ellipse_width / 2.0 * ucp[1].0;
    let r_br_curve_y = br_ellipse_center_y + ellipse_height / 2.0 * ucp[1].1;
    let r_br_curve = Point::from_pixels(r_br_curve_x, r_br_curve_y);

    draw.draw_command(DrawCommand::QuadraticCurveTo {
        control: r_br_curve,
        anchor: br_point,
    });
}

/// Implements `Graphics.drawRoundRect`.
pub fn draw_round_rect<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_display_object() {
        let x = args.get_f64(activation, 0)?;
        let y = args.get_f64(activation, 1)?;
        let width = args.get_f64(activation, 2)?;
        let height = args.get_f64(activation, 3)?;
        let ellipse_width = args.get_f64(activation, 4)?;
        let ellipse_height = args.get_f64(activation, 5)?;

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
pub fn draw_circle<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_display_object() {
        let x = args.get_f64(activation, 0)?;
        let y = args.get_f64(activation, 1)?;
        let radius = args.get_f64(activation, 2)?;

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
pub fn draw_ellipse<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_display_object() {
        let x = args.get_f64(activation, 0)?;
        let y = args.get_f64(activation, 1)?;
        let width = args.get_f64(activation, 2)?;
        let height = args.get_f64(activation, 3)?;

        if let Some(mut draw) = this.as_drawing(activation.context.gc_context) {
            draw_round_rect_internal(&mut draw, x, y, width, height, width, height)
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Graphics.lineGradientStyle`
pub fn line_gradient_style<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_display_object() {
        let gradient_type = args.get_string(activation, 0);
        let gradient_type = parse_gradient_type(activation, gradient_type?)?;
        let colors = args.get_object(activation, 1, "colors")?;
        let alphas = args.get_object(activation, 2, "alphas")?;
        let ratios = args.get_object(activation, 3, "ratios")?;
        let records = build_gradient_records(
            activation,
            &colors.as_array_storage().expect("Guaranteed by AS"),
            &alphas.as_array_storage().expect("Guaranteed by AS"),
            &ratios.as_array_storage().expect("Guaranteed by AS"),
        )?;
        let matrix = if let Some(matrix) = args.try_get_object(activation, 4) {
            Matrix::from(object_to_matrix(matrix, activation)?)
        } else {
            // Users can explicitly pass in `null` to mean identity matrix
            Matrix::IDENTITY
        };
        let spread = args.get_string(activation, 5);
        let spread = parse_spread_method(spread?);
        let interpolation = args.get_string(activation, 6);
        let interpolation = parse_interpolation_method(interpolation?);
        let focal_point = args.get_f64(activation, 7)?;

        if let Some(mut draw) = this.as_drawing(activation.context.gc_context) {
            match gradient_type {
                GradientType::Linear => {
                    draw.set_line_fill_style(FillStyle::LinearGradient(Gradient {
                        matrix,
                        spread,
                        interpolation,
                        records,
                    }))
                }
                GradientType::Radial if focal_point == 0.0 => {
                    draw.set_line_fill_style(FillStyle::RadialGradient(Gradient {
                        matrix,
                        spread,
                        interpolation,
                        records,
                    }))
                }
                _ => draw.set_line_fill_style(FillStyle::FocalGradient {
                    gradient: Gradient {
                        matrix,
                        spread,
                        interpolation,
                        records,
                    },
                    focal_point: Fixed8::from_f64(focal_point),
                }),
            }
        }
    }
    Ok(Value::Undefined)
}

/// Implements `Graphics.cubicCurveTo`
pub fn cubic_curve_to<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_display_object() {
        let control_a_x = args.get_f64(activation, 0)?;
        let control_a_y = args.get_f64(activation, 1)?;
        let control_b_x = args.get_f64(activation, 2)?;
        let control_b_y = args.get_f64(activation, 3)?;
        let anchor_x = args.get_f64(activation, 4)?;
        let anchor_y = args.get_f64(activation, 5)?;

        if let Some(mut draw) = this.as_drawing(activation.context.gc_context) {
            draw.draw_command(DrawCommand::CubicCurveTo {
                control_a: Point::from_pixels(control_a_x, control_a_y),
                control_b: Point::from_pixels(control_b_x, control_b_y),
                anchor: Point::from_pixels(anchor_x, anchor_y),
            });
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Graphics.copyFrom`
pub fn copy_from<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_display_object() {
        let source = args
            .get_object(activation, 0, "sourceGraphics")?
            .as_display_object()
            .expect("Bad sourceGraphics");

        let source = source
            .as_drawing(activation.context.gc_context)
            .expect("Missing drawing for sourceGraphics");

        let mut target_drawing = this
            .as_drawing(activation.context.gc_context)
            .expect("Missing drawing for target");

        target_drawing.copy_from(&source);
    }
    Ok(Value::Undefined)
}

/// Implements `Graphics.drawPath`
pub fn draw_path<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_display_object().unwrap();
    let mut drawing = this.as_drawing(activation.context.gc_context).unwrap();
    let commands = args.get_object(activation, 0, "commands")?;
    let data = args.get_object(activation, 1, "data")?;
    let winding = args.get_string(activation, 2)?;

    // FIXME - implement fill behavior described in the Flash docs
    // (which is different from just running each command sequentially on `Graphics`)
    avm2_stub_method!(
        activation,
        "flash.display.Graphics",
        "drawPath",
        "fill behavior"
    );

    let commands = commands
        .as_vector_storage()
        .expect("commands is not a Vector");
    let data = data.as_vector_storage().expect("data is not a Vector");

    process_commands(activation, &mut drawing, &commands, &data, winding)?;

    Ok(Value::Undefined)
}

/// Implements `Graphics.drawRoundRectComplex`
pub fn draw_round_rect_complex<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(activation, "flash.display.Graphics", "drawRoundRectComplex");
    Ok(Value::Undefined)
}

/// Implements `Graphics.drawTriangles`
pub fn draw_triangles<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_display_object() {
        if let Some(mut drawing) = this.as_drawing(activation.context.gc_context) {
            let vertices = args.get_object(activation, 0, "vertices")?;

            let indices = args.try_get_object(activation, 1);

            let uvt_data = args.try_get_object(activation, 2);

            let culling = {
                let culling = args.get_string(activation, 3)?;
                TriangleCulling::from_string(culling)
                    .ok_or_else(|| make_error_2004(activation, Error2004Type::ArgumentError))?
            };

            draw_triangles_internal(
                activation,
                &mut drawing,
                &vertices,
                indices.as_ref(),
                uvt_data.as_ref(),
                culling,
            )?;
        }
    }

    Ok(Value::Undefined)
}

#[derive(Debug, Clone, Copy)]
enum TriangleCulling {
    None,
    Positive,
    Negative,
}

impl TriangleCulling {
    fn from_string(value: AvmString) -> Option<Self> {
        if &value == b"none" {
            Some(Self::None)
        } else if &value == b"positive" {
            Some(Self::Positive)
        } else if &value == b"negative" {
            Some(Self::Negative)
        } else {
            None
        }
    }

    fn cull(self, (a, b, c): Triangle) -> bool {
        fn triangle_orientation((a, b, c): Triangle) -> i64 {
            let ax = a.x.get() as i64;
            let ay = a.y.get() as i64;
            let bx = b.x.get() as i64;
            let by = b.y.get() as i64;
            let cx = c.x.get() as i64;
            let cy = c.y.get() as i64;
            (bx - ax) * (cy - ay) - (by - ay) * (cx - ax)
        }

        match self {
            Self::None => false,
            Self::Positive => triangle_orientation((a, b, c)) >= 0,
            Self::Negative => triangle_orientation((a, b, c)) <= 0,
        }
    }
}

type Triangle = (Point<Twips>, Point<Twips>, Point<Twips>);

fn draw_triangles_internal<'gc>(
    activation: &mut Activation<'_, 'gc>,
    drawing: &mut Drawing,
    vertices: &Object<'gc>,
    indices: Option<&Object<'gc>>,
    uvt_data: Option<&Object<'gc>>,
    culling: TriangleCulling,
) -> Result<(), Error<'gc>> {
    // FIXME Triangles should be drawn using non-zero winding rule.
    //   When fixed, update output.expected.png of avm2/graphics_draw_triangles.
    avm2_stub_method!(
        activation,
        "flash.display.Graphics",
        "drawTriangles",
        "winding behavior"
    );

    if uvt_data.is_some() {
        avm2_stub_method!(
            activation,
            "flash.display.Graphics",
            "drawTriangles",
            "with uvt data"
        );
    }

    let vertices = vertices
        .as_vector_storage()
        .expect("vertices is not a Vector");

    if let Some(indices) = indices {
        if vertices.length() % 2 != 0 {
            return Err(make_error_2004(activation, Error2004Type::ArgumentError));
        }

        let indices = indices
            .as_vector_storage()
            .expect("indices is not a Vector");

        fn read_point<'gc>(
            vertices: &VectorStorage<'gc>,
            index: usize,
            activation: &mut Activation<'_, 'gc>,
        ) -> Result<Point<Twips>, Error<'gc>> {
            let x = {
                let x = vertices
                    .get(2 * index, activation)?
                    .coerce_to_number(activation)?;
                Twips::from_pixels(x)
            };
            let y = {
                let y = vertices
                    .get(2 * index + 1, activation)?
                    .coerce_to_number(activation)?;
                Twips::from_pixels(y)
            };

            Ok(Point::new(x, y))
        }

        fn next_triangle<'gc>(
            vertices: &VectorStorage<'gc>,
            indices: &mut impl Iterator<Item = Value<'gc>>,
            activation: &mut Activation<'_, 'gc>,
        ) -> Option<Triangle> {
            match (indices.next(), indices.next(), indices.next()) {
                (Some(i0), Some(i1), Some(i2)) => {
                    let i0 = i0.coerce_to_u32(activation).ok()? as usize;
                    let i1 = i1.coerce_to_u32(activation).ok()? as usize;
                    let i2 = i2.coerce_to_u32(activation).ok()? as usize;

                    let p0 = read_point(vertices, i0, activation).ok()?;
                    let p1 = read_point(vertices, i1, activation).ok()?;
                    let p2 = read_point(vertices, i2, activation).ok()?;

                    Some((p0, p1, p2))
                }
                _ => None,
            }
        }

        let indices = &mut indices.iter();

        while let Some(triangle) = next_triangle(&vertices, indices, activation) {
            draw_triangle_internal(triangle, drawing, culling);
        }
    } else {
        if vertices.length() % 6 != 0 {
            return Err(make_error_2004(activation, Error2004Type::ArgumentError));
        }

        let mut vertices = vertices.iter();

        fn read_point<'gc>(
            vertices: &mut impl Iterator<Item = Value<'gc>>,
            activation: &mut Activation<'_, 'gc>,
        ) -> Result<Option<Point<Twips>>, Error<'gc>> {
            let x = {
                let x = vertices.next();
                let x = match x {
                    Some(x) => x.coerce_to_number(activation)?,
                    None => return Ok(None),
                };
                Twips::from_pixels(x)
            };
            let y = {
                let y = vertices.next();
                let y = match y {
                    Some(y) => y.coerce_to_number(activation)?,
                    None => return Ok(None),
                };
                Twips::from_pixels(y)
            };

            Ok(Some(Point::new(x, y)))
        }

        fn next_triangle<'gc>(
            vertices: &mut impl Iterator<Item = Value<'gc>>,
            activation: &mut Activation<'_, 'gc>,
        ) -> Result<Option<Triangle>, Error<'gc>> {
            match (
                read_point(vertices, activation)?,
                read_point(vertices, activation)?,
                read_point(vertices, activation)?,
            ) {
                (Some(p0), Some(p1), Some(p2)) => Ok(Some((p0, p1, p2))),
                _ => Ok(None),
            }
        }

        while let Some(triangle) = next_triangle(&mut vertices, activation)? {
            draw_triangle_internal(triangle, drawing, culling);
        }
    }

    Ok(())
}

#[inline]
fn draw_triangle_internal((a, b, c): Triangle, drawing: &mut Drawing, culling: TriangleCulling) {
    if culling.cull((a, b, c)) {
        return;
    }

    drawing.draw_command(DrawCommand::MoveTo(a));

    drawing.draw_command(DrawCommand::LineTo(b));
    drawing.draw_command(DrawCommand::LineTo(c));
    drawing.draw_command(DrawCommand::LineTo(a));
}

/// Implements `Graphics.drawGraphicsData`
pub fn draw_graphics_data<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(vector) = args
        .get_object(activation, 0, "graphicsData")?
        .as_vector_storage()
    {
        //assert_eq!(vector.value_type(), Some(activation.avm2().classes().igraphicsdata));

        let this = this.as_display_object().expect("Bad this");

        if let Some(mut drawing) = this.as_drawing(activation.context.gc_context) {
            for elem in vector.iter() {
                let obj = elem.coerce_to_object(activation)?;

                handle_igraphics_data(activation, &mut drawing, &obj)?;
            }
        };
    }

    Ok(Value::Undefined)
}

/// Implements `Graphics.lineBitmapStyle`
pub fn line_bitmap_style<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_display_object() {
        let bitmap = args
            .get_object(activation, 0, "bitmap")?
            .as_bitmap_data()
            .expect("Bitmap argument is ensured to be a BitmapData from actionscript");
        let matrix = if let Some(matrix) = args.try_get_object(activation, 1) {
            Matrix::from(object_to_matrix(matrix, activation)?)
        } else {
            // Users can explicitly pass in `null` to mean identity matrix
            Matrix::IDENTITY
        };
        let is_repeating = args.get_bool(2);
        let is_smoothed = args.get_bool(3);

        let handle =
            bitmap.bitmap_handle(activation.context.gc_context, activation.context.renderer);

        let bitmap = ruffle_render::bitmap::BitmapInfo {
            handle,
            width: bitmap.width() as u16,
            height: bitmap.height() as u16,
        };
        let scale_matrix = Matrix::scale(
            Fixed16::from_f64(bitmap.width as f64),
            Fixed16::from_f64(bitmap.height as f64),
        );

        if let Some(mut draw) = this.as_drawing(activation.context.gc_context) {
            let id = draw.add_bitmap(bitmap);
            draw.set_line_fill_style(FillStyle::Bitmap {
                id,
                matrix: matrix * scale_matrix,
                is_smoothed,
                is_repeating,
            });
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Graphics.readGraphicsData`
pub fn read_graphics_data<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(activation, "flash.display.Graphics", "readGraphicsData");
    let value_type = activation
        .avm2()
        .classes()
        .igraphicsdata
        .inner_class_definition();
    let new_storage = VectorStorage::new(0, false, Some(value_type), activation);
    Ok(VectorObject::from_vector(new_storage, activation)?.into())
}

fn read_point<'gc>(
    activation: &mut Activation<'_, 'gc>,
    data: &VectorStorage<'gc>,
    data_index: &mut usize,
) -> Option<Point<Twips>> {
    let x = data
        .get(*data_index, activation)
        .ok()?
        .as_number(activation.context.gc_context)
        .expect("data is not a Vec.<Number>");

    let y = data
        .get(*data_index + 1, activation)
        .ok()?
        .as_number(activation.context.gc_context)
        .expect("data is not a Vec.<Number>");

    *data_index += 2;

    Some(Point {
        x: Twips::from_pixels(x),
        y: Twips::from_pixels(y),
    })
}

fn process_commands<'gc>(
    activation: &mut Activation<'_, 'gc>,
    drawing: &mut Drawing,
    commands: &VectorStorage<'gc>,
    data: &VectorStorage<'gc>,
    winding: AvmString,
) -> Result<(), Error<'gc>> {
    // Flash special cases this, and doesn't throw an error,
    // even if data has odd number of coordinates.
    if commands.length() == 0 {
        return Ok(());
    }

    // This error is always thrown at this point,
    // no matter if data is superfluous.
    if data.length() % 2 != 0 {
        return Err(make_error_2004(activation, Error2004Type::ArgumentError));
    }

    let rule = if winding == WStr::from_units(b"nonZero") {
        FillRule::NonZero
    } else if winding == WStr::from_units(b"evenOdd") {
        FillRule::EvenOdd
    } else {
        return Err(make_error_2008(activation, "winding"));
    };

    drawing.set_fill_rule(Some(rule));

    fn process_command<'gc>(
        activation: &mut Activation<'_, 'gc>,
        drawing: &mut Drawing,
        data: &VectorStorage<'gc>,
        command: i32,
        data_index: &mut usize,
    ) -> Option<()> {
        // Flash ignores commands which do not have data associated with them.
        match command {
            // NO_OP
            0 => {}
            // MOVE_TO
            1 => {
                let point = read_point(activation, data, data_index)?;
                drawing.draw_command(DrawCommand::MoveTo(point));
            }
            // LINE_TO
            2 => {
                let point = read_point(activation, data, data_index)?;
                drawing.draw_command(DrawCommand::LineTo(point));
            }
            // CURVE_TO
            3 => {
                let control = read_point(activation, data, data_index)?;
                let anchor = read_point(activation, data, data_index)?;
                drawing.draw_command(DrawCommand::QuadraticCurveTo { control, anchor });
            }
            // WIDE_MOVE_TO
            4 => {
                *data_index += 2;
                let point = read_point(activation, data, data_index)?;
                drawing.draw_command(DrawCommand::MoveTo(point));
            }
            // WIDE_LINE_TO
            5 => {
                *data_index += 2;
                let point = read_point(activation, data, data_index)?;
                drawing.draw_command(DrawCommand::LineTo(point));
            }
            // CUBIC_CURVE_TO
            6 => {
                let control_a = read_point(activation, data, data_index)?;
                let control_b = read_point(activation, data, data_index)?;
                let anchor = read_point(activation, data, data_index)?;
                drawing.draw_command(DrawCommand::CubicCurveTo {
                    control_a,
                    control_b,
                    anchor,
                });
            }
            _ => {
                // Unknown commands stop processing
                return None;
            }
        }
        Some(())
    }

    let mut data_index = 0;
    for i in 0..commands.length() {
        let command = commands
            .get(i, activation)
            .expect("missing command")
            .as_integer(activation.context.gc_context)
            .expect("commands is not a Vec.<int>");

        if process_command(activation, drawing, data, command, &mut data_index).is_none() {
            break;
        }
    }

    // Reset winding rule after drawing commands
    drawing.set_fill_rule(None);

    Ok(())
}

fn handle_igraphics_data<'gc>(
    activation: &mut Activation<'_, 'gc>,
    drawing: &mut Drawing,
    obj: &Object<'gc>,
) -> Result<(), Error<'gc>> {
    let class = obj.instance_class();

    if class
        == activation
            .avm2()
            .classes()
            .graphicsbitmapfill
            .inner_class_definition()
    {
        let style = handle_bitmap_fill(activation, drawing, obj)?;
        drawing.set_fill_style(Some(style));
    } else if class
        == activation
            .avm2()
            .classes()
            .graphicsendfill
            .inner_class_definition()
    {
        drawing.set_fill_style(None);
    } else if class
        == activation
            .avm2()
            .classes()
            .graphicsgradientfill
            .inner_class_definition()
    {
        let style = handle_gradient_fill(activation, obj)?;
        drawing.set_fill_style(Some(style));
    } else if class
        == activation
            .avm2()
            .classes()
            .graphicspath
            .inner_class_definition()
    {
        let commands = obj
            .get_public_property("commands", activation)?
            .coerce_to_object(activation)?;

        let data = obj
            .get_public_property("data", activation)?
            .coerce_to_object(activation)?;

        let winding = obj
            .get_public_property("winding", activation)?
            .coerce_to_string(activation)?;

        process_commands(
            activation,
            drawing,
            &commands
                .as_vector_storage()
                .expect("commands is not a Vector"),
            &data.as_vector_storage().expect("data is not a Vector"),
            winding,
        )?;
    } else if class
        == activation
            .avm2()
            .classes()
            .graphicssolidfill
            .inner_class_definition()
    {
        let style = handle_solid_fill(activation, obj)?;
        drawing.set_fill_style(Some(style));
    } else if class
        == activation
            .avm2()
            .classes()
            .graphicsshaderfill
            .inner_class_definition()
    {
        tracing::warn!("Graphics shader fill unimplemented {:?}", class);
        drawing.set_fill_style(None);
    } else if class
        == activation
            .avm2()
            .classes()
            .graphicsstroke
            .inner_class_definition()
    {
        let thickness = obj
            .get_public_property("thickness", activation)?
            .coerce_to_number(activation)?;

        if thickness.is_nan() {
            drawing.set_line_style(None);
        } else {
            let caps = {
                let caps = obj
                    .get_public_property("caps", activation)?
                    .coerce_to_string(activation);
                caps_to_cap_style(caps.ok())
            };
            let fill = {
                let fill = obj
                    .get_public_property("fill", activation)?
                    .coerce_to_object(activation)?;

                handle_igraphics_fill(activation, drawing, &fill)?
            };

            let joints = obj
                .get_public_property("joints", activation)?
                .coerce_to_string(activation)
                .ok();
            let miter_limit = obj
                .get_public_property("miterLimit", activation)?
                .coerce_to_number(activation)?;
            let pixel_hinting = obj
                .get_public_property("pixelHinting", activation)?
                .coerce_to_boolean();
            let scale_mode = obj
                .get_public_property("scaleMode", activation)?
                .coerce_to_string(activation)?;

            let width = Twips::from_pixels(thickness.clamp(0.0, 255.0));
            let join_style = joints_to_join_style(joints, miter_limit);
            let (allow_scale_x, allow_scale_y) = scale_mode_to_allow_scale_bits(&scale_mode)?;

            let mut line_style = LineStyle::new()
                .with_width(width)
                .with_start_cap(caps)
                .with_end_cap(caps)
                .with_join_style(join_style)
                .with_allow_scale_x(allow_scale_x)
                .with_allow_scale_y(allow_scale_y)
                .with_is_pixel_hinted(pixel_hinting)
                .with_allow_close(false);

            if let Some(fill) = fill {
                line_style = line_style.with_fill_style(fill);
            }

            drawing.set_line_style(Some(line_style));
        }
    } else if class
        == activation
            .avm2()
            .classes()
            .graphicstrianglepath
            .inner_class_definition()
    {
        handle_graphics_triangle_path(activation, drawing, obj)?;
    } else {
        panic!("Unknown graphics data class {:?}", class);
    }

    Ok(())
}

fn handle_graphics_triangle_path<'gc>(
    activation: &mut Activation<'_, 'gc>,
    drawing: &mut Drawing,
    obj: &Object<'gc>,
) -> Result<(), Error<'gc>> {
    let culling = {
        let culling = obj
            .get_public_property("culling", activation)?
            .coerce_to_string(activation)?;

        TriangleCulling::from_string(culling)
            .ok_or_else(|| make_error_2008(activation, "culling"))?
    };

    let vertices = obj.get_public_property("vertices", activation)?.as_object();
    let indices = obj.get_public_property("indices", activation)?.as_object();
    let uvt_data = obj.get_public_property("uvtData", activation)?.as_object();

    if let Some(vertices) = vertices {
        draw_triangles_internal(
            activation,
            drawing,
            &vertices,
            indices.as_ref(),
            uvt_data.as_ref(),
            culling,
        )?;
    }

    Ok(())
}

fn handle_igraphics_fill<'gc>(
    activation: &mut Activation<'_, 'gc>,
    drawing: &mut Drawing,
    obj: &Object<'gc>,
) -> Result<Option<FillStyle>, Error<'gc>> {
    let class = obj.instance_class();

    if class
        == activation
            .avm2()
            .classes()
            .graphicsbitmapfill
            .inner_class_definition()
    {
        let style = handle_bitmap_fill(activation, drawing, obj)?;
        Ok(Some(style))
    } else if class
        == activation
            .avm2()
            .classes()
            .graphicsendfill
            .inner_class_definition()
    {
        Ok(None)
    } else if class
        == activation
            .avm2()
            .classes()
            .graphicsgradientfill
            .inner_class_definition()
    {
        let style = handle_gradient_fill(activation, obj)?;
        Ok(Some(style))
    } else if class
        == activation
            .avm2()
            .classes()
            .graphicssolidfill
            .inner_class_definition()
    {
        let style = handle_solid_fill(activation, obj)?;
        Ok(Some(style))
    } else if class
        == activation
            .avm2()
            .classes()
            .graphicsshaderfill
            .inner_class_definition()
    {
        tracing::warn!("Graphics shader fill unimplemented {:?}", class);
        Ok(None)
    } else {
        tracing::warn!("Unknown graphics fill class {:?}", class);
        Ok(None)
    }
}

fn handle_solid_fill<'gc>(
    activation: &mut Activation<'_, 'gc>,
    obj: &Object<'gc>,
) -> Result<FillStyle, Error<'gc>> {
    let alpha = obj
        .get_public_property("alpha", activation)?
        .coerce_to_number(activation)
        .unwrap_or(1.0);

    let color = obj
        .get_public_property("color", activation)?
        .coerce_to_u32(activation)
        .unwrap_or(0);

    Ok(FillStyle::Color(color_from_args(color, alpha)))
}

fn handle_gradient_fill<'gc>(
    activation: &mut Activation<'_, 'gc>,
    obj: &Object<'gc>,
) -> Result<FillStyle, Error<'gc>> {
    let alphas = obj
        .get_public_property("alphas", activation)?
        .coerce_to_object(activation)?;

    let colors = obj
        .get_public_property("colors", activation)?
        .coerce_to_object(activation)?;

    let ratios = obj
        .get_public_property("ratios", activation)?
        .coerce_to_object(activation)?;

    let gradient_type = {
        let gradient_type = obj
            .get_public_property("type", activation)?
            .coerce_to_string(activation)?;
        parse_gradient_type(activation, gradient_type)?
    };

    let records = build_gradient_records(
        activation,
        &colors.as_array_storage().expect("Guaranteed by AS"),
        &alphas.as_array_storage().expect("Guaranteed by AS"),
        &ratios.as_array_storage().expect("Guaranteed by AS"),
    )?;

    let matrix = {
        let matrix = obj
            .get_public_property("matrix", activation)
            .ok()
            .and_then(|mat| mat.coerce_to_object(activation).ok());

        match matrix {
            Some(matrix) => Matrix::from(object_to_matrix(matrix, activation)?),
            None => Matrix::IDENTITY,
        }
    };

    let spread = {
        let spread_method = obj
            .get_public_property("spreadMethod", activation)?
            .coerce_to_string(activation)?;

        parse_spread_method(spread_method)
    };

    let interpolation = {
        let interpolation_method = obj
            .get_public_property("interpolationMethod", activation)?
            .coerce_to_string(activation)?;

        parse_interpolation_method(interpolation_method)
    };

    let focal_point = obj
        .get_public_property("focalPointRatio", activation)?
        .coerce_to_number(activation)?;

    let fill = match gradient_type {
        GradientType::Linear => FillStyle::LinearGradient(Gradient {
            matrix,
            spread,
            interpolation,
            records,
        }),
        GradientType::Radial if focal_point == 0.0 => FillStyle::RadialGradient(Gradient {
            matrix,
            spread,
            interpolation,
            records,
        }),
        _ => FillStyle::FocalGradient {
            gradient: Gradient {
                matrix,
                spread,
                interpolation,
                records,
            },
            focal_point: Fixed8::from_f64(focal_point),
        },
    };

    Ok(fill)
}

fn handle_bitmap_fill<'gc>(
    activation: &mut Activation<'_, 'gc>,
    drawing: &mut Drawing,
    obj: &Object<'gc>,
) -> Result<FillStyle, Error<'gc>> {
    let bitmap_data = obj
        .get_public_property("bitmapData", activation)?
        .coerce_to_object(activation)?
        .as_bitmap_data()
        .expect("Bitmap argument is ensured to be a BitmapData from actionscript");

    let matrix = obj
        .get_public_property("matrix", activation)
        .and_then(|prop| {
            let matrix = prop.coerce_to_object(activation)?;

            let matrix = Matrix::from(object_to_matrix(matrix, activation)?);

            Ok(matrix)
        })
        .unwrap_or(Matrix::IDENTITY);

    let is_repeating = obj
        .get_public_property("repeat", activation)?
        .coerce_to_boolean();

    let is_smoothed = obj
        .get_public_property("smooth", activation)?
        .coerce_to_boolean();

    let handle =
        bitmap_data.bitmap_handle(activation.context.gc_context, activation.context.renderer);

    let bitmap = ruffle_render::bitmap::BitmapInfo {
        handle,
        width: bitmap_data.width() as u16,
        height: bitmap_data.height() as u16,
    };

    let scale_matrix = Matrix::scale(
        Fixed16::from_f64(bitmap.width as f64),
        Fixed16::from_f64(bitmap.height as f64),
    );

    let id = drawing.add_bitmap(bitmap);

    let style = FillStyle::Bitmap {
        id,
        matrix: matrix * scale_matrix,
        is_smoothed,
        is_repeating,
    };

    Ok(style)
}
