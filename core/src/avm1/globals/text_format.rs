//! `TextFormat` impl

use crate::avm1::object::NativeObject;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Activation, ArrayObject, Error, Object, ScriptObject, TObject, Value};
use crate::avm1_stub;
use crate::display_object::{AutoSizeMode, EditText, TDisplayObject};
use crate::ecma_conversions::round_to_even;
use crate::html::TextFormat;
use crate::string::{AvmString, StringContext, WStr};
use gc_arena::Gc;

macro_rules! getter {
    ($name:ident) => {
        |activation, this, _args| {
            if let NativeObject::TextFormat(text_format) = this.native() {
                return Ok($name(activation, &text_format.borrow()));
            }
            Ok(Value::Undefined)
        }
    };
}

macro_rules! setter {
    ($name:ident) => {
        |activation, this, args| {
            if let NativeObject::TextFormat(text_format) = this.native() {
                let value = args.get(0).unwrap_or(&Value::Undefined);
                $name(activation, &mut text_format.borrow_mut(), value)?;
            }
            Ok(Value::Undefined)
        }
    };
}

macro_rules! method {
    ($name:ident) => {
        |activation, this, args| {
            if let NativeObject::TextFormat(text_format) = this.native() {
                return $name(activation, &text_format.borrow(), args);
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
    "getTextExtent" => method(method!(get_text_extent); DONT_ENUM | DONT_DELETE);
};

fn font<'gc>(activation: &mut Activation<'_, 'gc>, text_format: &TextFormat) -> Value<'gc> {
    text_format.font.as_ref().map_or(Value::Null, |font| {
        AvmString::new(activation.gc(), font.clone()).into()
    })
}

fn set_font<'gc>(
    activation: &mut Activation<'_, 'gc>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error<'gc>> {
    text_format.font = match value {
        Value::Undefined | Value::Null => None,
        value => Some(value.coerce_to_string(activation)?.as_wstr().into()),
    };
    Ok(())
}

fn size<'gc>(_activation: &mut Activation<'_, 'gc>, text_format: &TextFormat) -> Value<'gc> {
    text_format
        .size
        .as_ref()
        .map_or(Value::Null, |&size| size.into())
}

fn set_size<'gc>(
    activation: &mut Activation<'_, 'gc>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error<'gc>> {
    text_format.size = match value {
        Value::Undefined | Value::Null => None,
        value => Some(round_to_even(value.coerce_to_f64(activation)?).into()),
    };
    Ok(())
}

fn color<'gc>(_activation: &mut Activation<'_, 'gc>, text_format: &TextFormat) -> Value<'gc> {
    text_format
        .color
        .as_ref()
        .map_or(Value::Null, |color| color.to_rgba().into())
}

fn set_color<'gc>(
    activation: &mut Activation<'_, 'gc>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error<'gc>> {
    text_format.color = match value {
        Value::Undefined | Value::Null => None,
        value => Some(swf::Color::from_rgba(value.coerce_to_u32(activation)?)),
    };
    Ok(())
}

fn url<'gc>(activation: &mut Activation<'_, 'gc>, text_format: &TextFormat) -> Value<'gc> {
    text_format.url.as_ref().map_or(Value::Null, |url| {
        AvmString::new(activation.gc(), url.clone()).into()
    })
}

fn set_url<'gc>(
    activation: &mut Activation<'_, 'gc>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error<'gc>> {
    text_format.url = match value {
        Value::Undefined | Value::Null => None,
        value => Some(value.coerce_to_string(activation)?.as_wstr().into()),
    };
    Ok(())
}

fn target<'gc>(activation: &mut Activation<'_, 'gc>, text_format: &TextFormat) -> Value<'gc> {
    text_format.target.as_ref().map_or(Value::Null, |target| {
        AvmString::new(activation.gc(), target.clone()).into()
    })
}

fn set_target<'gc>(
    activation: &mut Activation<'_, 'gc>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error<'gc>> {
    text_format.target = match value {
        Value::Undefined | Value::Null => None,
        value => Some(value.coerce_to_string(activation)?.as_wstr().into()),
    };
    Ok(())
}

fn bold<'gc>(_activation: &mut Activation<'_, 'gc>, text_format: &TextFormat) -> Value<'gc> {
    text_format
        .bold
        .as_ref()
        .map_or(Value::Null, |&bold| bold.into())
}

fn set_bold<'gc>(
    activation: &mut Activation<'_, 'gc>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error<'gc>> {
    text_format.bold = match value {
        Value::Undefined | Value::Null => None,
        value => Some(value.as_bool(activation.swf_version())),
    };
    Ok(())
}

fn italic<'gc>(_activation: &mut Activation<'_, 'gc>, text_format: &TextFormat) -> Value<'gc> {
    text_format
        .italic
        .as_ref()
        .map_or(Value::Null, |&italic| italic.into())
}

fn set_italic<'gc>(
    activation: &mut Activation<'_, 'gc>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error<'gc>> {
    text_format.italic = match value {
        Value::Undefined | Value::Null => None,
        value => Some(value.as_bool(activation.swf_version())),
    };
    Ok(())
}

fn underline<'gc>(_activation: &mut Activation<'_, 'gc>, text_format: &TextFormat) -> Value<'gc> {
    text_format
        .underline
        .as_ref()
        .map_or(Value::Null, |&underline| underline.into())
}

fn set_underline<'gc>(
    activation: &mut Activation<'_, 'gc>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error<'gc>> {
    text_format.underline = match value {
        Value::Undefined | Value::Null => None,
        value => Some(value.as_bool(activation.swf_version())),
    };
    Ok(())
}

fn align<'gc>(_activation: &mut Activation<'_, 'gc>, text_format: &TextFormat) -> Value<'gc> {
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
    activation: &mut Activation<'_, 'gc>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error<'gc>> {
    if matches!(value, Value::Undefined | Value::Null) {
        text_format.align = None;
        return Ok(());
    }

    let value = value.coerce_to_string(activation)?;
    let align = if value.eq_ignore_case(WStr::from_units(b"left")) {
        swf::TextAlign::Left
    } else if value.eq_ignore_case(WStr::from_units(b"center")) {
        swf::TextAlign::Center
    } else if value.eq_ignore_case(WStr::from_units(b"right")) {
        swf::TextAlign::Right
    } else if value.eq_ignore_case(WStr::from_units(b"justify")) {
        swf::TextAlign::Justify
    } else {
        return Ok(());
    };
    text_format.align = Some(align);
    Ok(())
}

fn left_margin<'gc>(_activation: &mut Activation<'_, 'gc>, text_format: &TextFormat) -> Value<'gc> {
    text_format
        .left_margin
        .as_ref()
        .map_or(Value::Null, |&left_margin| left_margin.into())
}

fn set_left_margin<'gc>(
    activation: &mut Activation<'_, 'gc>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error<'gc>> {
    text_format.left_margin = match value {
        Value::Undefined | Value::Null => None,
        value => Some(round_to_even(value.coerce_to_f64(activation)?.max(0.0)).into()),
    };
    Ok(())
}

fn right_margin<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    text_format: &TextFormat,
) -> Value<'gc> {
    text_format
        .right_margin
        .as_ref()
        .map_or(Value::Null, |&right_margin| right_margin.into())
}

fn set_right_margin<'gc>(
    activation: &mut Activation<'_, 'gc>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error<'gc>> {
    text_format.right_margin = match value {
        Value::Undefined | Value::Null => None,
        value => Some(round_to_even(value.coerce_to_f64(activation)?.max(0.0)).into()),
    };
    Ok(())
}

fn indent<'gc>(_activation: &mut Activation<'_, 'gc>, text_format: &TextFormat) -> Value<'gc> {
    text_format
        .indent
        .as_ref()
        .map_or(Value::Null, |&indent| indent.into())
}

fn set_indent<'gc>(
    activation: &mut Activation<'_, 'gc>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error<'gc>> {
    text_format.indent = match value {
        Value::Undefined | Value::Null => None,
        value => Some(round_to_even(value.coerce_to_f64(activation)?).into()),
    };
    Ok(())
}

fn leading<'gc>(_activation: &mut Activation<'_, 'gc>, text_format: &TextFormat) -> Value<'gc> {
    text_format
        .leading
        .as_ref()
        .map_or(Value::Null, |&leading| leading.into())
}

fn set_leading<'gc>(
    activation: &mut Activation<'_, 'gc>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error<'gc>> {
    text_format.leading = match value {
        Value::Undefined | Value::Null => None,
        value => Some(round_to_even(value.coerce_to_f64(activation)?).into()),
    };
    Ok(())
}

fn block_indent<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    text_format: &TextFormat,
) -> Value<'gc> {
    text_format
        .block_indent
        .as_ref()
        .map_or(Value::Null, |&block_indent| block_indent.into())
}

fn set_block_indent<'gc>(
    activation: &mut Activation<'_, 'gc>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error<'gc>> {
    text_format.block_indent = match value {
        Value::Undefined | Value::Null => None,
        value => Some(round_to_even(value.coerce_to_f64(activation)?).into()),
    };
    Ok(())
}

fn tab_stops<'gc>(activation: &mut Activation<'_, 'gc>, text_format: &TextFormat) -> Value<'gc> {
    text_format
        .tab_stops
        .as_ref()
        .map_or(Value::Null, |tab_stops| {
            ArrayObject::new(
                activation.gc(),
                activation.context.avm1.prototypes().array,
                tab_stops.iter().map(|&x| x.into()),
            )
            .into()
        })
}

fn set_tab_stops<'gc>(
    activation: &mut Activation<'_, 'gc>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error<'gc>> {
    text_format.tab_stops = match value {
        Value::Object(object) => {
            let length = object.length(activation)?;
            let tab_stops: Result<Vec<_>, Error<'gc>> = (0..length)
                .map(|i| {
                    let element = object.get_element(activation, i);
                    Ok(round_to_even(element.coerce_to_f64(activation)?).into())
                })
                .collect();
            Some(tab_stops?)
        }
        _ => None,
    };
    Ok(())
}

fn bullet<'gc>(_activation: &mut Activation<'_, 'gc>, text_format: &TextFormat) -> Value<'gc> {
    text_format
        .bullet
        .as_ref()
        .map_or(Value::Null, |&bullet| bullet.into())
}

fn set_bullet<'gc>(
    activation: &mut Activation<'_, 'gc>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error<'gc>> {
    text_format.bullet = match value {
        Value::Undefined | Value::Null => None,
        value => Some(value.as_bool(activation.swf_version())),
    };
    Ok(())
}

fn display<'gc>(activation: &mut Activation<'_, 'gc>, _text_format: &TextFormat) -> Value<'gc> {
    avm1_stub!(activation, "TextFormat", "display");
    Value::Null
}

fn set_display<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _text_format: &mut TextFormat,
    _value: &Value<'gc>,
) -> Result<(), Error<'gc>> {
    avm1_stub!(activation, "TextFormat", "display");
    Ok(())
}

fn kerning<'gc>(_activation: &mut Activation<'_, 'gc>, text_format: &TextFormat) -> Value<'gc> {
    text_format
        .kerning
        .as_ref()
        .map_or(Value::Null, |&kerning| kerning.into())
}

fn set_kerning<'gc>(
    activation: &mut Activation<'_, 'gc>,
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
    _activation: &mut Activation<'_, 'gc>,
    text_format: &TextFormat,
) -> Value<'gc> {
    text_format
        .letter_spacing
        .as_ref()
        .map_or(Value::Null, |&letter_spacing| letter_spacing.into())
}

fn set_letter_spacing<'gc>(
    activation: &mut Activation<'_, 'gc>,
    text_format: &mut TextFormat,
    value: &Value<'gc>,
) -> Result<(), Error<'gc>> {
    text_format.letter_spacing = match value {
        Value::Undefined | Value::Null => None,
        value => Some(value.coerce_to_f64(activation)?),
    };
    Ok(())
}

fn get_text_extent<'gc>(
    activation: &mut Activation<'_, 'gc>,
    text_format: &TextFormat,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let movie = activation.base_clip().movie();
    let text = args
        .get(0)
        .cloned()
        .unwrap_or(Value::Undefined)
        .coerce_to_string(activation)?;
    let width = args
        .get(1)
        .cloned()
        .map(|v| v.coerce_to_f64(activation))
        .transpose()?;

    let temp_edittext = EditText::new(
        activation.context,
        movie,
        0.0,
        0.0,
        width.unwrap_or(0.0),
        0.0,
    );

    temp_edittext.set_autosize(AutoSizeMode::Left, activation.context);
    temp_edittext.set_word_wrap(width.is_some(), activation.context);
    temp_edittext.set_new_text_format(text_format.clone(), activation.context);
    temp_edittext.set_text(&text, activation.context);

    let result = ScriptObject::new(activation.gc(), None);
    let metrics = temp_edittext
        .layout_metrics(None)
        .expect("All text boxes should have at least one line at all times");

    result.set_data(
        "ascent".into(),
        metrics.ascent.to_pixels().into(),
        activation,
    )?;
    result.set_data(
        "descent".into(),
        metrics.descent.to_pixels().into(),
        activation,
    )?;
    result.set_data("width".into(), metrics.width.to_pixels().into(), activation)?;
    result.set_data(
        "height".into(),
        metrics.height.to_pixels().into(),
        activation,
    )?;
    result.set_data(
        "textFieldHeight".into(),
        temp_edittext.height().into(),
        activation,
    )?;
    result.set_data(
        "textFieldWidth".into(),
        temp_edittext.width().into(),
        activation,
    )?;

    Ok(result.into())
}

/// `TextFormat` constructor
pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let mut text_format = Default::default();
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
    this.set_native(
        activation.gc(),
        NativeObject::TextFormat(Gc::new(activation.gc(), text_format.into())),
    );
    Ok(this.into())
}

/// `TextFormat.prototype` constructor
pub fn create_proto<'gc>(
    context: &mut StringContext<'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let object = ScriptObject::new(context.gc(), Some(proto));
    define_properties_on(PROTO_DECLS, context, object, fn_proto);
    object.into()
}
