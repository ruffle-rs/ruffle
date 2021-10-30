//! `TextFormat` impl

use crate::avm1::object::text_format_object::TextFormatObject;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Activation, ArrayObject, AvmString, Error, Object, TObject, Value};
use crate::avm_warn;
use crate::html::TextFormat;
use gc_arena::MutationContext;

macro_rules! getter {
    ($name:ident) => {
        |activation, this, _args| {
            if let Some(text_format) = this.as_text_format_object() {
                return Ok($name(activation, &text_format.text_format()));
            }
            Ok(Value::Undefined)
        }
    };
}

macro_rules! setter {
    ($name:ident) => {
        |activation, this, args| {
            if let Some(text_format) = this.as_text_format_object() {
                let value = args.get(0).unwrap_or(&Value::Undefined);
                $name(
                    activation,
                    &mut text_format.text_format_mut(activation.context.gc_context),
                    value,
                )?;
            }
            Ok(Value::Undefined)
        }
    };
}

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "font" => property(getter!(font), setter!(set_font));
    "size" => property(getter!(size), setter!(set_size));
    "color" => property(getter!(color), setter!(set_color));
    "url" => property(getter!(url), setter!(set_url));
    "target" => property(getter!(target), setter!(set_target));
    "bold" => property(getter!(bold), setter!(set_bold));
    "italic" => property(getter!(italic), setter!(set_italic));
    "underline" => property(getter!(underline), setter!(set_underline));
    "align" => property(getter!(align), setter!(set_align));
    "leftMargin" => property(getter!(left_margin), setter!(set_left_margin));
    "rightMargin" => property(getter!(right_margin), setter!(set_right_margin));
    "indent" => property(getter!(indent), setter!(set_indent));
    "leading" => property(getter!(leading), setter!(set_leading));
    "blockIndent" => property(getter!(block_indent), setter!(set_block_indent));
    "tabStops" => property(getter!(tab_stops), setter!(set_tab_stops));
    "bullet" => property(getter!(bullet), setter!(set_bullet));
    "display" => property(getter!(display), setter!(set_display));
    "kerning" => property(getter!(kerning), setter!(set_kerning));
    "letterSpacing" => property(getter!(letter_spacing), setter!(set_letter_spacing));
};

fn font<'gc>(activation: &mut Activation<'_, 'gc, '_>, text_format: &TextFormat) -> Value<'gc> {
    text_format.font.as_ref().map_or(Value::Null, |font| {
        AvmString::new(activation.context.gc_context, font).into()
    })
}

fn set_font<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error<'gc>> {
    text_format.font = match value {
        Value::Undefined | Value::Null => None,
        value => Some(value.coerce_to_string(activation)?.to_string()),
    };
    Ok(())
}

fn size<'gc>(_activation: &mut Activation<'_, 'gc, '_>, text_format: &TextFormat) -> Value<'gc> {
    text_format
        .size
        .as_ref()
        .map_or(Value::Null, |&size| size.into())
}

fn set_size<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error<'gc>> {
    text_format.size = match value {
        Value::Undefined | Value::Null => None,
        // TODO: round up
        value => Some(value.coerce_to_i32(activation)?.into()),
    };
    Ok(())
}

fn color<'gc>(_activation: &mut Activation<'_, 'gc, '_>, text_format: &TextFormat) -> Value<'gc> {
    text_format
        .color
        .as_ref()
        .map_or(Value::Null, |color| color.to_rgba().into())
}

fn set_color<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error<'gc>> {
    text_format.color = match value {
        Value::Undefined | Value::Null => None,
        value => Some(swf::Color::from_rgba(value.coerce_to_u32(activation)?)),
    };
    Ok(())
}

fn url<'gc>(activation: &mut Activation<'_, 'gc, '_>, text_format: &TextFormat) -> Value<'gc> {
    text_format.url.as_ref().map_or(Value::Null, |url| {
        AvmString::new(activation.context.gc_context, url).into()
    })
}

fn set_url<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error<'gc>> {
    text_format.url = match value {
        Value::Undefined | Value::Null => None,
        value => Some(value.coerce_to_string(activation)?.to_string()),
    };
    Ok(())
}

fn target<'gc>(activation: &mut Activation<'_, 'gc, '_>, text_format: &TextFormat) -> Value<'gc> {
    text_format.target.as_ref().map_or(Value::Null, |target| {
        AvmString::new(activation.context.gc_context, target).into()
    })
}

fn set_target<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error<'gc>> {
    text_format.target = match value {
        Value::Undefined | Value::Null => None,
        value => Some(value.coerce_to_string(activation)?.to_string()),
    };
    Ok(())
}

fn bold<'gc>(_activation: &mut Activation<'_, 'gc, '_>, text_format: &TextFormat) -> Value<'gc> {
    text_format
        .bold
        .as_ref()
        .map_or(Value::Null, |&bold| bold.into())
}

fn set_bold<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error<'gc>> {
    text_format.bold = match value {
        Value::Undefined | Value::Null => None,
        value => Some(value.as_bool(activation.swf_version())),
    };
    Ok(())
}

fn italic<'gc>(_activation: &mut Activation<'_, 'gc, '_>, text_format: &TextFormat) -> Value<'gc> {
    text_format
        .italic
        .as_ref()
        .map_or(Value::Null, |&italic| italic.into())
}

fn set_italic<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error<'gc>> {
    text_format.italic = match value {
        Value::Undefined | Value::Null => None,
        value => Some(value.as_bool(activation.swf_version())),
    };
    Ok(())
}

fn underline<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    text_format: &TextFormat,
) -> Value<'gc> {
    text_format
        .underline
        .as_ref()
        .map_or(Value::Null, |&underline| underline.into())
}

fn set_underline<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error<'gc>> {
    text_format.underline = match value {
        Value::Undefined | Value::Null => None,
        value => Some(value.as_bool(activation.swf_version())),
    };
    Ok(())
}

fn align<'gc>(_activation: &mut Activation<'_, 'gc, '_>, text_format: &TextFormat) -> Value<'gc> {
    text_format
        .align
        .as_ref()
        .map_or(Value::Null, |align| match align {
            swf::TextAlign::Left => "left".into(),
            swf::TextAlign::Center => "center".into(),
            swf::TextAlign::Right => "right".into(),
            swf::TextAlign::Justify => "justify".into(),
        })
}

fn set_align<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error<'gc>> {
    text_format.align = match value {
        Value::Undefined | Value::Null => None,
        value => match value.coerce_to_string(activation)?.to_lowercase().as_str() {
            "left" => Some(swf::TextAlign::Left),
            "center" => Some(swf::TextAlign::Center),
            "right" => Some(swf::TextAlign::Right),
            "justify" => Some(swf::TextAlign::Justify),
            _ => None,
        },
    };
    Ok(())
}

fn left_margin<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    text_format: &TextFormat,
) -> Value<'gc> {
    text_format
        .left_margin
        .as_ref()
        .map_or(Value::Null, |&left_margin| left_margin.into())
}

fn set_left_margin<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error<'gc>> {
    text_format.left_margin = match value {
        Value::Undefined | Value::Null => None,
        // TODO: round up
        value => Some(value.coerce_to_i32(activation)?.into()),
    };
    Ok(())
}

fn right_margin<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    text_format: &TextFormat,
) -> Value<'gc> {
    text_format
        .right_margin
        .as_ref()
        .map_or(Value::Null, |&right_margin| right_margin.into())
}

fn set_right_margin<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error<'gc>> {
    text_format.right_margin = match value {
        Value::Undefined | Value::Null => None,
        // TODO: round up
        value => Some(value.coerce_to_i32(activation)?.into()),
    };
    Ok(())
}

fn indent<'gc>(_activation: &mut Activation<'_, 'gc, '_>, text_format: &TextFormat) -> Value<'gc> {
    text_format
        .indent
        .as_ref()
        .map_or(Value::Null, |&indent| indent.into())
}

fn set_indent<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error<'gc>> {
    text_format.indent = match value {
        Value::Undefined | Value::Null => None,
        // TODO: round up
        value => Some(value.coerce_to_i32(activation)?.into()),
    };
    Ok(())
}

fn leading<'gc>(_activation: &mut Activation<'_, 'gc, '_>, text_format: &TextFormat) -> Value<'gc> {
    text_format
        .leading
        .as_ref()
        .map_or(Value::Null, |&leading| leading.into())
}

fn set_leading<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error<'gc>> {
    text_format.leading = match value {
        Value::Undefined | Value::Null => None,
        // TODO: round up
        value => Some(value.coerce_to_i32(activation)?.into()),
    };
    Ok(())
}

fn block_indent<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    text_format: &TextFormat,
) -> Value<'gc> {
    text_format
        .block_indent
        .as_ref()
        .map_or(Value::Null, |&block_indent| block_indent.into())
}

fn set_block_indent<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error<'gc>> {
    text_format.block_indent = match value {
        Value::Undefined | Value::Null => None,
        // TODO: round up
        value => Some(value.coerce_to_i32(activation)?.into()),
    };
    Ok(())
}

fn tab_stops<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    text_format: &TextFormat,
) -> Value<'gc> {
    text_format
        .tab_stops
        .as_ref()
        .map_or(Value::Null, |tab_stops| {
            ArrayObject::new(
                activation.context.gc_context,
                activation.context.avm1.prototypes().array,
                tab_stops.iter().map(|&x| x.into()),
            )
            .into()
        })
}

fn set_tab_stops<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error<'gc>> {
    text_format.tab_stops = match value {
        Value::Object(object) => {
            let length = object.length(activation)?;
            let tab_stops: Result<Vec<_>, Error<'gc>> = (0..length)
                .map(|i| {
                    let element = object.get_element(activation, i);
                    // TODO: round up
                    Ok(element.coerce_to_i32(activation)?.into())
                })
                .collect();
            Some(tab_stops?)
        }
        _ => None,
    };
    Ok(())
}

fn bullet<'gc>(_activation: &mut Activation<'_, 'gc, '_>, text_format: &TextFormat) -> Value<'gc> {
    text_format
        .bullet
        .as_ref()
        .map_or(Value::Null, |&bullet| bullet.into())
}

fn set_bullet<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error<'gc>> {
    text_format.bullet = match value {
        Value::Undefined | Value::Null => None,
        value => Some(value.as_bool(activation.swf_version())),
    };
    Ok(())
}

fn display<'gc>(activation: &mut Activation<'_, 'gc, '_>, _text_format: &TextFormat) -> Value<'gc> {
    avm_warn!(activation, "TextFormat.display: Unimplemented");
    Value::Null
}

fn set_display<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _text_format: &mut TextFormat,
    _value: &Value<'gc>,
) -> Result<(), Error<'gc>> {
    avm_warn!(activation, "TextFormat.display: Unimplemented");
    Ok(())
}

fn kerning<'gc>(_activation: &mut Activation<'_, 'gc, '_>, text_format: &TextFormat) -> Value<'gc> {
    text_format
        .kerning
        .as_ref()
        .map_or(Value::Null, |&kerning| kerning.into())
}

fn set_kerning<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error<'gc>> {
    text_format.kerning = match value {
        Value::Undefined | Value::Null => None,
        value => Some(value.as_bool(activation.swf_version())),
    };
    Ok(())
}

fn letter_spacing<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    text_format: &TextFormat,
) -> Value<'gc> {
    text_format
        .letter_spacing
        .as_ref()
        .map_or(Value::Null, |&letter_spacing| letter_spacing.into())
}

fn set_letter_spacing<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error<'gc>> {
    text_format.letter_spacing = match value {
        Value::Undefined | Value::Null => None,
        value => Some(value.coerce_to_f64(activation)?),
    };
    Ok(())
}

/// `TextFormat` constructor
pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_text_format_object() {
        let mut text_format = TextFormat::default();
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
        this.set_text_format(activation.context.gc_context, text_format);
    }

    Ok(this.into())
}

/// `TextFormat.prototype` constructor
pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let text_format = TextFormatObject::empty_object(gc_context, Some(proto));
    let object = text_format.as_script_object().unwrap();
    define_properties_on(PROTO_DECLS, gc_context, object, fn_proto);
    text_format.into()
}
