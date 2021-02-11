//! `flash.display.Graphics` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::method::Method;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{Object, TObject};
use crate::avm2::traits::Trait;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::display_object::TDisplayObject;
use crate::shape_utils::DrawCommand;
use gc_arena::{GcCell, MutationContext};
use swf::{Color, FillStyle, LineCapStyle, LineJoinStyle, LineStyle, Twips};

/// Implements `flash.display.Graphics`'s instance constructor.
pub fn instance_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Graphics cannot be constructed directly.".into())
}

/// Implements `flash.display.Graphics`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Convert a `color`/`alpha` argument pair into a `swf::Color`.
fn color_from_args(color: u32, alpha: f64) -> Color {
    let r = (color & 0xFF0000 >> 16) as u8;
    let g = (color & 0x00FF00 >> 8) as u8;
    let b = (color & 0x0000FF) as u8;
    let a = (alpha * 255.0) as u8;

    Color { r, g, b, a }
}

/// Implements `Graphics.beginFill`.
pub fn begin_fill<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(dobj) = this.as_display_object() {
            if let Some(mc) = dobj.as_movie_clip() {
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

                mc.set_fill_style(
                    &mut activation.context,
                    Some(FillStyle::Color(color_from_args(color, alpha))),
                );
            }
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
    if let Some(this) = this {
        if let Some(dobj) = this.as_display_object() {
            if let Some(mc) = dobj.as_movie_clip() {
                mc.clear(&mut activation.context)
            }
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
    if let Some(this) = this {
        if let Some(dobj) = this.as_display_object() {
            if let Some(mc) = dobj.as_movie_clip() {
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

                mc.draw_command(
                    &mut activation.context,
                    DrawCommand::CurveTo { x1, y1, x2, y2 },
                );
            }
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
    if let Some(this) = this {
        if let Some(dobj) = this.as_display_object() {
            if let Some(mc) = dobj.as_movie_clip() {
                mc.set_fill_style(&mut activation.context, None);
            }
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
    miter_limit: f32,
) -> Result<LineJoinStyle, Error> {
    let joints_string = joints.coerce_to_string(activation);
    let joints_str = joints_string.as_deref();

    match (joints, joints_str) {
        (Value::Null, _) | (_, Ok("round")) => Ok(LineJoinStyle::Round),
        (_, Ok("miter")) => Ok(LineJoinStyle::Miter(miter_limit)),
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
    if let Some(this) = this {
        if let Some(dobj) = this.as_display_object() {
            if let Some(mc) = dobj.as_movie_clip() {
                let thickness = args
                    .get(0)
                    .cloned()
                    .unwrap_or_else(|| f64::NAN.into())
                    .coerce_to_number(activation)?;
                if thickness.is_nan() {
                    mc.set_line_style(&mut activation.context, None);
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
                    let caps =
                        caps_to_cap_style(activation, args.get(5).cloned().unwrap_or(Value::Null))?;
                    let joints = args.get(6).cloned().unwrap_or(Value::Null);
                    let miter_limit = args
                        .get(7)
                        .cloned()
                        .unwrap_or_else(|| 3.0.into())
                        .coerce_to_number(activation)?;

                    let width = Twips::from_pixels(thickness.min(255.0).max(0.0));
                    let color = color_from_args(color, alpha);
                    let join_style = joints_to_join_style(activation, joints, miter_limit as f32)?;
                    let (allow_scale_x, allow_scale_y) =
                        scale_mode_to_allow_scale_bits(&scale_mode)?;

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

                    mc.set_line_style(&mut activation.context, Some(line_style));
                }
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
    if let Some(this) = this {
        if let Some(dobj) = this.as_display_object() {
            if let Some(mc) = dobj.as_movie_clip() {
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

                mc.draw_command(&mut activation.context, DrawCommand::LineTo { x, y });
            }
        }
    }

    Ok(Value::Undefined)
}

/// Construct `Graphics`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.display"), "Graphics"),
        Some(QName::new(Namespace::public(), "Object").into()),
        Method::from_builtin(instance_init),
        Method::from_builtin(class_init),
        mc,
    );

    let mut write = class.write(mc);

    write.define_instance_trait(Trait::from_method(
        QName::new(Namespace::public(), "beginFill"),
        Method::from_builtin(begin_fill),
    ));
    write.define_instance_trait(Trait::from_method(
        QName::new(Namespace::public(), "clear"),
        Method::from_builtin(clear),
    ));
    write.define_instance_trait(Trait::from_method(
        QName::new(Namespace::public(), "curveTo"),
        Method::from_builtin(curve_to),
    ));
    write.define_instance_trait(Trait::from_method(
        QName::new(Namespace::public(), "endFill"),
        Method::from_builtin(end_fill),
    ));
    write.define_instance_trait(Trait::from_method(
        QName::new(Namespace::public(), "lineStyle"),
        Method::from_builtin(line_style),
    ));
    write.define_instance_trait(Trait::from_method(
        QName::new(Namespace::public(), "lineTo"),
        Method::from_builtin(line_to),
    ));

    class
}
