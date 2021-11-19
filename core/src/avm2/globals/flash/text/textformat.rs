//! `flash.text.TextFormat` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{textformat_allocator, ArrayObject, Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::ecma_conversions::round_to_even;
use crate::html::TextFormat;
use crate::string::AvmString;
use gc_arena::{GcCell, MutationContext};

/// Implements `flash.text.TextFormat`'s instance constructor.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        activation.super_init(this, &[])?;

        if let Some(mut text_format) = this.as_text_format_mut(activation.context.gc_context) {
            set_font(
                activation,
                &mut text_format,
                args.get(0).unwrap_or(&Value::Undefined),
            )?;
            set_size(
                activation,
                &mut text_format,
                args.get(1).unwrap_or(&Value::Undefined),
            )?;
            set_color(
                activation,
                &mut text_format,
                args.get(2).unwrap_or(&Value::Undefined),
            )?;
            set_bold(
                activation,
                &mut text_format,
                args.get(3).unwrap_or(&Value::Undefined),
            )?;
            set_italic(
                activation,
                &mut text_format,
                args.get(4).unwrap_or(&Value::Undefined),
            )?;
            set_underline(
                activation,
                &mut text_format,
                args.get(5).unwrap_or(&Value::Undefined),
            )?;
            set_url(
                activation,
                &mut text_format,
                args.get(6).unwrap_or(&Value::Undefined),
            )?;
            set_target(
                activation,
                &mut text_format,
                args.get(7).unwrap_or(&Value::Undefined),
            )?;
            set_align(
                activation,
                &mut text_format,
                args.get(8).unwrap_or(&Value::Undefined),
            )?;
            set_left_margin(
                activation,
                &mut text_format,
                args.get(9).unwrap_or(&Value::Undefined),
            )?;
            set_right_margin(
                activation,
                &mut text_format,
                args.get(10).unwrap_or(&Value::Undefined),
            )?;
            set_indent(
                activation,
                &mut text_format,
                args.get(11).unwrap_or(&Value::Undefined),
            )?;
            set_leading(
                activation,
                &mut text_format,
                args.get(12).unwrap_or(&Value::Undefined),
            )?;
        }
    }

    Ok(Value::Undefined)
}

/// Implements `flash.text.TextFormat`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

macro_rules! getter {
    ($name:ident) => {
        |activation, this, _args| {
            if let Some(this) = this {
                if let Some(text_format) = this.as_text_format() {
                    return $name(activation, &text_format);
                }
            }
            Ok(Value::Undefined)
        }
    };
}

macro_rules! setter {
    ($name:ident) => {
        |activation, this, args| {
            if let Some(this) = this {
                if let Some(mut text_format) =
                    this.as_text_format_mut(activation.context.gc_context)
                {
                    let value = args.get(0).unwrap_or(&Value::Undefined);
                    $name(activation, &mut text_format, value)?;
                }
            }
            Ok(Value::Undefined)
        }
    };
}

fn align<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    text_format: &TextFormat,
) -> Result<Value<'gc>, Error> {
    Ok(text_format
        .align
        .as_ref()
        .map_or(Value::Null, |align| match align {
            swf::TextAlign::Left => "left".into(),
            swf::TextAlign::Center => "center".into(),
            swf::TextAlign::Right => "right".into(),
            swf::TextAlign::Justify => "justify".into(),
        }))
}

fn set_align<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error> {
    text_format.align = match value {
        Value::Undefined | Value::Null => None,
        value => match value.coerce_to_string(activation)?.as_str() {
            "left" => Some(swf::TextAlign::Left),
            "center" => Some(swf::TextAlign::Center),
            "right" => Some(swf::TextAlign::Right),
            "justify" => Some(swf::TextAlign::Justify),
            _ => return Err(
                "ArgumentError: Error #2008: Parameter align must be one of the accepted values."
                    .into(),
            ),
        },
    };
    Ok(())
}

fn block_indent<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    text_format: &TextFormat,
) -> Result<Value<'gc>, Error> {
    Ok(text_format
        .block_indent
        .as_ref()
        .map_or(Value::Null, |&block_indent| block_indent.into()))
}

fn set_block_indent<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error> {
    text_format.block_indent = match value {
        Value::Undefined | Value::Null => None,
        value => Some(round_to_even(value.coerce_to_number(activation)?).into()),
    };
    Ok(())
}

fn bold<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    text_format: &TextFormat,
) -> Result<Value<'gc>, Error> {
    Ok(text_format
        .bold
        .as_ref()
        .map_or(Value::Null, |&bold| bold.into()))
}

fn set_bold<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error> {
    text_format.bold = match value {
        Value::Undefined | Value::Null => None,
        value => Some(value.coerce_to_boolean()),
    };
    Ok(())
}

fn bullet<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    text_format: &TextFormat,
) -> Result<Value<'gc>, Error> {
    Ok(text_format
        .bullet
        .as_ref()
        .map_or(Value::Null, |&bullet| bullet.into()))
}

fn set_bullet<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error> {
    text_format.bullet = match value {
        Value::Undefined | Value::Null => None,
        value => Some(value.coerce_to_boolean()),
    };
    Ok(())
}

fn color<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    text_format: &TextFormat,
) -> Result<Value<'gc>, Error> {
    Ok(text_format
        .color
        .as_ref()
        .map_or(Value::Null, |color| (color.to_rgba() as i32).into()))
}

fn set_color<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error> {
    text_format.color = match value {
        Value::Undefined | Value::Null => None,
        value => Some(swf::Color::from_rgba(value.coerce_to_u32(activation)?)),
    };
    Ok(())
}

fn font<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    text_format: &TextFormat,
) -> Result<Value<'gc>, Error> {
    Ok(text_format.font.as_ref().map_or(Value::Null, |font| {
        AvmString::new(activation.context.gc_context, font).into()
    }))
}

fn set_font<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error> {
    text_format.font = match value {
        Value::Undefined | Value::Null => None,
        value => Some(value.coerce_to_string(activation)?.to_string()),
    };
    Ok(())
}

fn indent<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    text_format: &TextFormat,
) -> Result<Value<'gc>, Error> {
    Ok(text_format
        .indent
        .as_ref()
        .map_or(Value::Null, |&indent| indent.into()))
}

fn set_indent<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error> {
    text_format.indent = match value {
        Value::Undefined | Value::Null => None,
        value => Some(round_to_even(value.coerce_to_number(activation)?).into()),
    };
    Ok(())
}

fn italic<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    text_format: &TextFormat,
) -> Result<Value<'gc>, Error> {
    Ok(text_format
        .italic
        .as_ref()
        .map_or(Value::Null, |&italic| italic.into()))
}

fn set_italic<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error> {
    text_format.italic = match value {
        Value::Undefined | Value::Null => None,
        value => Some(value.coerce_to_boolean()),
    };
    Ok(())
}

fn kerning<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    text_format: &TextFormat,
) -> Result<Value<'gc>, Error> {
    Ok(text_format
        .kerning
        .as_ref()
        .map_or(Value::Null, |&kerning| kerning.into()))
}

fn set_kerning<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error> {
    text_format.kerning = match value {
        Value::Undefined | Value::Null => None,
        value => Some(value.coerce_to_boolean()),
    };
    Ok(())
}

fn leading<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    text_format: &TextFormat,
) -> Result<Value<'gc>, Error> {
    Ok(text_format
        .leading
        .as_ref()
        .map_or(Value::Null, |&leading| leading.into()))
}

fn set_leading<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error> {
    text_format.leading = match value {
        Value::Undefined | Value::Null => None,
        value => Some(round_to_even(value.coerce_to_number(activation)?).into()),
    };
    Ok(())
}

fn left_margin<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    text_format: &TextFormat,
) -> Result<Value<'gc>, Error> {
    Ok(text_format
        .left_margin
        .as_ref()
        .map_or(Value::Null, |&left_margin| left_margin.into()))
}

fn set_left_margin<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error> {
    text_format.left_margin = match value {
        Value::Undefined | Value::Null => None,
        value => Some(round_to_even(value.coerce_to_number(activation)?).into()),
    };
    Ok(())
}

fn letter_spacing<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    text_format: &TextFormat,
) -> Result<Value<'gc>, Error> {
    Ok(text_format
        .letter_spacing
        .as_ref()
        .map_or(Value::Null, |&letter_spacing| letter_spacing.into()))
}

fn set_letter_spacing<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error> {
    text_format.letter_spacing = match value {
        Value::Undefined | Value::Null => None,
        value => Some(value.coerce_to_number(activation)?),
    };
    Ok(())
}

fn right_margin<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    text_format: &TextFormat,
) -> Result<Value<'gc>, Error> {
    Ok(text_format
        .right_margin
        .as_ref()
        .map_or(Value::Null, |&right_margin| right_margin.into()))
}

fn set_right_margin<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error> {
    text_format.right_margin = match value {
        Value::Undefined | Value::Null => None,
        value => Some(round_to_even(value.coerce_to_number(activation)?).into()),
    };
    Ok(())
}

fn size<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    text_format: &TextFormat,
) -> Result<Value<'gc>, Error> {
    Ok(text_format
        .size
        .as_ref()
        .map_or(Value::Null, |&size| size.into()))
}

fn set_size<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error> {
    text_format.size = match value {
        Value::Undefined | Value::Null => None,
        value => Some(round_to_even(value.coerce_to_number(activation)?).into()),
    };
    Ok(())
}

fn tab_stops<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    text_format: &TextFormat,
) -> Result<Value<'gc>, Error> {
    text_format
        .tab_stops
        .as_ref()
        .map_or(Ok(Value::Null), |tab_stops| {
            let tab_stop_storage = tab_stops.iter().copied().collect();
            Ok(ArrayObject::from_storage(activation, tab_stop_storage)?.into())
        })
}

fn set_tab_stops<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error> {
    text_format.tab_stops = match value {
        Value::Undefined | Value::Null => None,
        value => {
            let object = value.coerce_to_object(activation)?;
            let length = object.as_array_storage().map_or(0, |v| v.length());

            let tab_stops: Result<Vec<_>, Error> = (0..length)
                .map(|i| {
                    let element = object.get_property(
                        object,
                        &QName::new(
                            Namespace::public(),
                            AvmString::new(activation.context.gc_context, format!("{}", i)),
                        )
                        .into(),
                        activation,
                    )?;
                    Ok(round_to_even(element.coerce_to_number(activation)?).into())
                })
                .collect();
            Some(tab_stops?)
        }
    };
    Ok(())
}

fn target<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    text_format: &TextFormat,
) -> Result<Value<'gc>, Error> {
    Ok(text_format.target.as_ref().map_or(Value::Null, |target| {
        AvmString::new(activation.context.gc_context, target).into()
    }))
}

fn set_target<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error> {
    text_format.target = match value {
        Value::Undefined | Value::Null => None,
        value => Some(value.coerce_to_string(activation)?.to_string()),
    };
    Ok(())
}

fn underline<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    text_format: &TextFormat,
) -> Result<Value<'gc>, Error> {
    Ok(text_format
        .underline
        .as_ref()
        .map_or(Value::Null, |&underline| underline.into()))
}

fn set_underline<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error> {
    text_format.underline = match value {
        Value::Undefined | Value::Null => None,
        value => Some(value.coerce_to_boolean()),
    };
    Ok(())
}

fn url<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    text_format: &TextFormat,
) -> Result<Value<'gc>, Error> {
    Ok(text_format.url.as_ref().map_or(Value::Null, |url| {
        AvmString::new(activation.context.gc_context, url).into()
    }))
}

fn set_url<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error> {
    text_format.url = match value {
        Value::Undefined | Value::Null => None,
        value => Some(value.coerce_to_string(activation)?.to_string()),
    };
    Ok(())
}

/// Construct `TextFormat`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.text"), "TextFormat"),
        Some(QName::new(Namespace::public(), "Object").into()),
        Method::from_builtin(instance_init, "<TextFormat instance initializer>", mc),
        Method::from_builtin(class_init, "<TextFormat class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);
    write.set_instance_allocator(textformat_allocator);

    write.set_attributes(ClassAttributes::SEALED);

    const PUBLIC_INSTANCE_PROPERTIES: &[(
        &str,
        Option<NativeMethodImpl>,
        Option<NativeMethodImpl>,
    )] = &[
        ("align", Some(getter!(align)), Some(setter!(set_align))),
        (
            "blockIndent",
            Some(getter!(block_indent)),
            Some(setter!(set_block_indent)),
        ),
        ("bold", Some(getter!(bold)), Some(setter!(set_bold))),
        ("bullet", Some(getter!(bullet)), Some(setter!(set_bullet))),
        ("color", Some(getter!(color)), Some(setter!(set_color))),
        ("font", Some(getter!(font)), Some(setter!(set_font))),
        ("indent", Some(getter!(indent)), Some(setter!(set_indent))),
        ("italic", Some(getter!(italic)), Some(setter!(set_italic))),
        (
            "kerning",
            Some(getter!(kerning)),
            Some(setter!(set_kerning)),
        ),
        (
            "leading",
            Some(getter!(leading)),
            Some(setter!(set_leading)),
        ),
        (
            "leftMargin",
            Some(getter!(left_margin)),
            Some(setter!(set_left_margin)),
        ),
        (
            "letterSpacing",
            Some(getter!(letter_spacing)),
            Some(setter!(set_letter_spacing)),
        ),
        (
            "rightMargin",
            Some(getter!(right_margin)),
            Some(setter!(set_right_margin)),
        ),
        ("size", Some(getter!(size)), Some(setter!(set_size))),
        (
            "tabStops",
            Some(getter!(tab_stops)),
            Some(setter!(set_tab_stops)),
        ),
        ("target", Some(getter!(target)), Some(setter!(set_target))),
        (
            "underline",
            Some(getter!(underline)),
            Some(setter!(set_underline)),
        ),
        ("url", Some(getter!(url)), Some(setter!(set_url))),
    ];
    write.define_public_builtin_instance_properties(mc, PUBLIC_INSTANCE_PROPERTIES);

    class
}
