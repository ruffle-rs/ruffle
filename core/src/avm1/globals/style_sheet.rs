use std::fmt;

use crate::avm1::object::{Object, TObject};
use crate::avm1::property_decl::define_properties_on;
use crate::avm1::{
    property_decl::Declaration, ArrayObject, ExecutionReason, NativeObject, ScriptObject,
};
use crate::avm1::{Activation, Error, Value};
use crate::backend::navigator::Request;
use crate::html::{transform_dashes_to_camel_case, CssStream, StyleSheet, TextFormat};
use crate::string::{AvmString, StringContext};
use gc_arena::{Collect, Gc, Mutation};
use ruffle_macros::istr;
use ruffle_wstr::{WStr, WString};

/// A `StyleSheet` object that is tied to a style sheet.
#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct StyleSheetObject<'gc>(StyleSheet<'gc>);

impl fmt::Debug for StyleSheetObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("StyleSheetObject")
            .field("style_sheet", &self.0)
            .finish()
    }
}

impl<'gc> StyleSheetObject<'gc> {
    pub fn new(mc: &Mutation<'gc>) -> Self {
        Self(StyleSheet::new(mc))
    }

    pub fn set_style(self, selector: WString, format: TextFormat) {
        self.0.set_style(selector, format);
    }

    pub fn style_sheet(self) -> StyleSheet<'gc> {
        self.0
    }
}

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "setStyle" => method(set_style; DONT_ENUM | DONT_DELETE | READ_ONLY | VERSION_7);
    "clear" => method(clear; DONT_ENUM | DONT_DELETE | READ_ONLY | VERSION_7);
    "getStyleNames" => method(get_style_names; DONT_ENUM | DONT_DELETE | READ_ONLY | VERSION_7);
    "load" => method(load; DONT_ENUM | DONT_DELETE | READ_ONLY | VERSION_7);
    "getStyle" => method(get_style; DONT_ENUM | DONT_DELETE | READ_ONLY | VERSION_7);
    "transform" => method(transform; DONT_ENUM | DONT_DELETE | READ_ONLY | VERSION_7);
    "parseCSS" => method(parse_css; DONT_ENUM | DONT_DELETE | READ_ONLY | VERSION_7);
    "parse" => method(parse_css; DONT_ENUM | DONT_DELETE | READ_ONLY | VERSION_7);
};

fn shallow_copy<'gc>(
    activation: &mut Activation<'_, 'gc>,
    value: Value<'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    if let Value::Object(object) = value {
        let object_proto = activation.context.avm1.prototypes().object;
        let result = ScriptObject::new(activation.strings(), Some(object_proto));

        for key in object.get_keys(activation, false) {
            result.set(key, object.get_stored(key, activation)?, activation)?;
        }

        Ok(result.into())
    } else {
        Ok(Value::Null)
    }
}

fn set_style<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if !this.has_property(activation, istr!("_styles")) {
        this.set(
            istr!("_styles"),
            ArrayObject::empty(activation).into(),
            activation,
        )?;
    }
    if !this.has_property(activation, istr!("_css")) {
        this.set(
            istr!("_css"),
            ArrayObject::empty(activation).into(),
            activation,
        )?;
    }
    let css = this
        .get_stored(istr!("_css"), activation)?
        .coerce_to_object(activation);
    let styles = this
        .get_stored(istr!("_styles"), activation)?
        .coerce_to_object(activation);
    let name = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?;
    let object = args.get(1).unwrap_or(&Value::Undefined);

    css.set(name, shallow_copy(activation, *object)?, activation)?;
    let text_format = this.call_method(
        istr!("transform"),
        &[shallow_copy(activation, *object)?],
        activation,
        ExecutionReason::Special,
    )?;
    styles.set(name, text_format, activation)?;

    if let NativeObject::StyleSheet(style_sheet) = this.native() {
        if let Value::Object(text_format) = text_format {
            if let NativeObject::TextFormat(text_format) = text_format.native() {
                style_sheet.set_style(
                    name.as_wstr().to_ascii_lowercase(),
                    text_format.borrow().clone(),
                );
            }
        }
    }

    Ok(Value::Undefined)
}

fn get_style<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let css = this
        .get_stored(istr!("_css"), activation)?
        .coerce_to_object(activation);
    let name = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?;
    let style = css.get(name, activation)?;
    shallow_copy(activation, style)
}

fn get_style_names<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let css = this
        .get_stored(istr!("_css"), activation)?
        .coerce_to_object(activation);
    Ok(ArrayObject::builder(activation)
        .with(
            css.get_keys(activation, false)
                .into_iter()
                .map(Value::String),
        )
        .into())
}

fn load<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let url = match args.get(0) {
        Some(val) => val.coerce_to_string(activation)?,
        None => return Ok(false.into()),
    };

    let request = Request::get(url.to_utf8_lossy().into_owned());

    let future = activation.context.load_manager.load_stylesheet(
        activation.context.player.clone(),
        this,
        request,
    );
    activation.context.navigator.spawn_future(future);

    Ok(true.into())
}

fn transform<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let mut text_format = TextFormat {
        display: Some(crate::html::TextDisplay::Block),
        kerning: Some(false),
        ..Default::default()
    };

    let Some(style_object) = args.get(0) else {
        return Ok(Value::Null);
    };

    let style_object = match style_object {
        Value::Undefined | Value::Null => {
            return Ok(Value::Null);
        }
        Value::Object(object) => Some(object),
        _ => None,
    };

    if let Some(style_object) = style_object {
        fn get_style<'gc>(
            style_object: &Object<'gc>,
            name: &'static str,
            activation: &mut Activation<'_, 'gc>,
        ) -> Option<Value<'gc>> {
            let name = AvmString::new_utf8(activation.gc(), name);

            match style_object.get_stored(name, activation).ok()? {
                Value::Undefined => None,
                value => Some(value),
            }
        }

        fn parse_color(input: AvmString<'_>) -> Option<swf::Color> {
            let stripped = input.strip_prefix(WStr::from_units(b"#"))?;

            if stripped.len() != 6 {
                return None;
            }

            if let Ok(number) = u32::from_str_radix(&stripped.to_string(), 16) {
                Some(swf::Color::from_rgba(number))
            } else {
                None
            }
        }

        fn parse_suffixed_number_i32<'gc>(
            activation: &mut Activation<'_, 'gc>,
            input: &Value<'gc>,
        ) -> Result<i32, Error<'gc>> {
            let kerning = super::parse_int_internal(activation, input, None)?;
            kerning.coerce_to_i32(activation)
        }

        fn parse_suffixed_number_f64<'gc>(
            activation: &mut Activation<'_, 'gc>,
            input: &Value<'gc>,
        ) -> Result<f64, Error<'gc>> {
            let kerning = super::parse_int_internal(activation, input, None)?;
            kerning.coerce_to_f64(activation)
        }

        if let Some(Value::String(color)) = get_style(style_object, "color", activation) {
            text_format.color = parse_color(color);
        }

        if let Some(display) = get_style(style_object, "display", activation) {
            let display = display.coerce_to_string(activation)?;
            if &display == b"none" {
                text_format.display = Some(crate::html::TextDisplay::None);
            } else if &display == b"inline" {
                text_format.display = Some(crate::html::TextDisplay::Inline);
            } else {
                text_format.display = Some(crate::html::TextDisplay::Block);
            }
        }

        if let Some(family) = get_style(style_object, "fontFamily", activation) {
            if family.as_bool(activation.swf_version()) {
                let font_list =
                    crate::html::parse_font_list(family.coerce_to_string(activation)?.as_wstr());
                let font_list = AvmString::new(activation.gc(), font_list);
                text_format.font = Some(font_list.as_wstr().to_owned());
            }
        }

        if let Some(size) = get_style(style_object, "fontSize", activation) {
            let size = parse_suffixed_number_i32(activation, &size)?;
            if size > 0 {
                text_format.size = Some(size as f64);
            }
        }

        if let Ok(style) = style_object.get_stored(istr!("fontStyle"), activation) {
            let style = style.coerce_to_string(activation)?;
            if &style == b"normal" {
                text_format.italic = Some(false);
            } else if &style == b"italic" {
                text_format.italic = Some(true);
            }
        }

        if let Ok(weight) = style_object.get_stored(istr!("fontWeight"), activation) {
            let weight = weight.coerce_to_string(activation)?;
            if &weight == b"normal" {
                text_format.bold = Some(false);
            } else if &weight == b"bold" {
                text_format.bold = Some(true);
            }
        }

        if let Some(kerning) = get_style(style_object, "kerning", activation) {
            let kerning = match kerning {
                Value::String(string) if &string == b"true" => true,
                kerning => parse_suffixed_number_i32(activation, &kerning)? != 0,
            };
            text_format.kerning = Some(kerning);
        }

        if let Some(leading) = get_style(style_object, "leading", activation) {
            if leading.as_bool(activation.swf_version()) {
                let leading = parse_suffixed_number_i32(activation, &leading)?;
                text_format.leading = Some(leading as f64);
            }
        }

        if let Some(letter_spacing) = get_style(style_object, "letterSpacing", activation) {
            if letter_spacing.as_bool(activation.swf_version()) {
                let letter_spacing = parse_suffixed_number_f64(activation, &letter_spacing)?;
                text_format.letter_spacing = Some(letter_spacing);
            }
        }

        if let Some(left_margin) = get_style(style_object, "marginLeft", activation) {
            if left_margin.as_bool(activation.swf_version()) {
                let left_margin = parse_suffixed_number_i32(activation, &left_margin)?;
                text_format.left_margin = Some(left_margin.max(0) as f64);
            }
        }

        if let Some(right_margin) = get_style(style_object, "marginRight", activation) {
            if right_margin.as_bool(activation.swf_version()) {
                let right_margin = parse_suffixed_number_i32(activation, &right_margin)?;
                text_format.right_margin = Some(right_margin.max(0) as f64);
            }
        }

        if let Some(align) = get_style(style_object, "textAlign", activation) {
            let align = align.coerce_to_string(activation)?.to_ascii_lowercase();
            if &align == b"left" {
                text_format.align = Some(swf::TextAlign::Left);
            } else if &align == b"center" {
                text_format.align = Some(swf::TextAlign::Center);
            } else if &align == b"right" {
                text_format.align = Some(swf::TextAlign::Right);
            } else if &align == b"justify" {
                text_format.align = Some(swf::TextAlign::Justify);
            }
        }

        if let Some(decoration) = get_style(style_object, "textDecoration", activation) {
            let decoration = decoration.coerce_to_string(activation)?;
            if &decoration == b"none" {
                text_format.underline = Some(false);
            } else if &decoration == b"underline" {
                text_format.underline = Some(true);
            }
        }

        if let Some(indent) = get_style(style_object, "textIndent", activation) {
            if indent.as_bool(activation.swf_version()) {
                let indent = parse_suffixed_number_i32(activation, &indent)?;
                text_format.indent = Some(indent as f64);
            }
        }
    }

    let proto = activation.context.avm1.prototypes().text_format;
    let object = ScriptObject::new(activation.strings(), Some(proto));
    object.set_native(
        activation.gc(),
        NativeObject::TextFormat(Gc::new(activation.gc(), text_format.into())),
    );
    Ok(object.into())
}

fn parse_css<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let source = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?;

    if let Ok(css) = CssStream::new(&source).parse() {
        for (selector, properties) in css.into_iter() {
            if !selector.is_empty() {
                let proto = activation.context.avm1.prototypes().object;
                let object = ScriptObject::new(activation.strings(), Some(proto));

                for (key, value) in properties.into_iter() {
                    object.set(
                        AvmString::new(activation.gc(), transform_dashes_to_camel_case(key)),
                        AvmString::new(activation.gc(), value).into(),
                        activation,
                    )?;
                }

                set_style(
                    activation,
                    this,
                    &[
                        AvmString::new(activation.gc(), selector).into(),
                        object.into(),
                    ],
                )?;
            }
        }
        Ok(Value::Bool(true))
    } else {
        Ok(Value::Bool(false))
    }
}

fn clear<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    this.set(
        istr!("_styles"),
        ArrayObject::empty(activation).into(),
        activation,
    )?;
    this.set(
        istr!("_css"),
        ArrayObject::empty(activation).into(),
        activation,
    )?;
    Ok(Value::Undefined)
}

pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let style_sheet = StyleSheetObject::new(activation.gc());
    this.set_native(activation.gc(), NativeObject::StyleSheet(style_sheet));

    Ok(this.into())
}

pub fn create_proto<'gc>(
    context: &mut StringContext<'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let style_sheet_proto = ScriptObject::new(context, Some(proto));
    define_properties_on(PROTO_DECLS, context, style_sheet_proto, fn_proto);
    style_sheet_proto.into()
}
