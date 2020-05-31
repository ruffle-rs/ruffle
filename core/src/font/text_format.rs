//! Classes that store formatting options
use crate::avm1::{Avm1, Object, ScriptObject, TObject, Value};
use crate::context::UpdateContext;

type Error = Box<dyn std::error::Error>;

/// A set of text formatting options to be applied to some part, or the whole
/// of, a given text field.
///
/// Any property set to `None` is treated as undefined, which has different
/// meanings based on the context by which the `TextFormat` is used. For
/// example, when getting the format of a particular region of text, `None`
/// means that multiple regions of text apply. When setting the format of a
/// particular region of text, `None` means that the existing setting for that
/// property will be retained.
#[derive(Clone, Debug)]
pub struct TextFormat {
    font: Option<String>,
    size: Option<f64>,
    color: Option<swf::Color>,
    align: Option<swf::TextAlign>,
    bold: Option<bool>,
    italic: Option<bool>,
    underline: Option<bool>,
    left_margin: Option<f64>,
    right_margin: Option<f64>,
    indent: Option<f64>,
    block_indent: Option<f64>,
    kerning: Option<bool>,
    leading: Option<f64>,
    letter_spacing: Option<f64>,
    tab_stops: Option<Vec<f64>>,
    bullet: Option<bool>,
    url: Option<String>,
    target: Option<String>,
}

impl Default for TextFormat {
    fn default() -> Self {
        Self {
            font: None,
            size: None,
            color: None,
            align: None,
            bold: None,
            italic: None,
            underline: None,
            left_margin: None,
            right_margin: None,
            indent: None,
            block_indent: None,
            kerning: None,
            leading: None,
            letter_spacing: None,
            tab_stops: None,
            bullet: None,
            url: None,
            target: None,
        }
    }
}

fn getstr_from_avm1_object<'gc>(
    object: Object<'gc>,
    name: &str,
    avm1: &mut Avm1<'gc>,
    uc: &mut UpdateContext<'_, 'gc, '_>,
) -> Result<Option<String>, Error> {
    Ok(match object.get(name, avm1, uc)? {
        Value::Undefined => None,
        Value::Null => None,
        v => Some(v.coerce_to_string(avm1, uc)?),
    })
}

fn getfloat_from_avm1_object<'gc>(
    object: Object<'gc>,
    name: &str,
    avm1: &mut Avm1<'gc>,
    uc: &mut UpdateContext<'_, 'gc, '_>,
) -> Result<Option<f64>, Error> {
    Ok(match object.get(name, avm1, uc)? {
        Value::Undefined => None,
        Value::Null => None,
        v => Some(v.as_number(avm1, uc)?),
    })
}

fn getbool_from_avm1_object<'gc>(
    object: Object<'gc>,
    name: &str,
    avm1: &mut Avm1<'gc>,
    uc: &mut UpdateContext<'_, 'gc, '_>,
) -> Result<Option<bool>, Error> {
    Ok(match object.get(name, avm1, uc)? {
        Value::Undefined => None,
        Value::Null => None,
        v => Some(v.as_bool(avm1.current_swf_version())),
    })
}

impl TextFormat {
    /// Construct a `TextFormat` from an object that is
    pub fn from_avm1_object<'gc>(
        object1: Object<'gc>,
        avm1: &mut Avm1<'gc>,
        uc: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<Self, Error> {
        Ok(Self {
            font: getstr_from_avm1_object(object1, "font", avm1, uc)?,
            size: getfloat_from_avm1_object(object1, "size", avm1, uc)?,
            color: getfloat_from_avm1_object(object1, "color", avm1, uc)?
                .map(|v| swf::Color::from_rgb(v as u32, 0xFF)),
            align: getstr_from_avm1_object(object1, "align", avm1, uc)?.and_then(|v| {
                match v.to_lowercase().as_str() {
                    "left" => Some(swf::TextAlign::Left),
                    "center" => Some(swf::TextAlign::Center),
                    "right" => Some(swf::TextAlign::Right),
                    "justify" => Some(swf::TextAlign::Justify),
                    _ => None,
                }
            }),
            bold: getbool_from_avm1_object(object1, "bold", avm1, uc)?,
            italic: getbool_from_avm1_object(object1, "italic", avm1, uc)?,
            underline: getbool_from_avm1_object(object1, "underline", avm1, uc)?,
            left_margin: getfloat_from_avm1_object(object1, "leftMargin", avm1, uc)?,
            right_margin: getfloat_from_avm1_object(object1, "rightMargin", avm1, uc)?,
            indent: getfloat_from_avm1_object(object1, "indent", avm1, uc)?,
            block_indent: getfloat_from_avm1_object(object1, "blockIndent", avm1, uc)?,
            kerning: getbool_from_avm1_object(object1, "kerning", avm1, uc)?,
            leading: getfloat_from_avm1_object(object1, "leading", avm1, uc)?,
            letter_spacing: getfloat_from_avm1_object(object1, "letterSpacing", avm1, uc)?,
            tab_stops: None,
            bullet: getbool_from_avm1_object(object1, "bullet", avm1, uc)?,
            url: getstr_from_avm1_object(object1, "url", avm1, uc)?,
            target: getstr_from_avm1_object(object1, "target", avm1, uc)?,
        })
    }

    /// Construct a `TextFormat` AVM1 object from this text format object.
    pub fn as_avm1_object<'gc>(
        &self,
        avm1: &mut Avm1<'gc>,
        uc: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<Object<'gc>, Error> {
        let object = ScriptObject::object(uc.gc_context, Some(avm1.prototypes().text_format));

        object.set(
            "font",
            self.font.clone().map(|v| v.into()).unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "size",
            self.size.map(|v| v.into()).unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "color",
            self.color
                .clone()
                .map(|v| (((v.r as u32) << 16) + ((v.g as u32) << 8) + v.b as u32).into())
                .unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "align",
            self.align
                .map(|v| {
                    match v {
                        swf::TextAlign::Left => "left",
                        swf::TextAlign::Center => "center",
                        swf::TextAlign::Right => "right",
                        swf::TextAlign::Justify => "justify",
                    }
                    .into()
                })
                .unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "bold",
            self.bold.map(|v| v.into()).unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "italic",
            self.italic.map(|v| v.into()).unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "underline",
            self.underline.map(|v| v.into()).unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "leftMargin",
            self.left_margin.map(|v| v.into()).unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "rightMargin",
            self.right_margin.map(|v| v.into()).unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "indent",
            self.indent.map(|v| v.into()).unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "blockIndent",
            self.block_indent.map(|v| v.into()).unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "kerning",
            self.kerning.map(|v| v.into()).unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "leading",
            self.leading.map(|v| v.into()).unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "letterSpacing",
            self.letter_spacing.map(|v| v.into()).unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "bullet",
            self.bullet.map(|v| v.into()).unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "url",
            self.url.clone().map(|v| v.into()).unwrap_or(Value::Null),
            avm1,
            uc,
        )?;
        object.set(
            "target",
            self.target.clone().map(|v| v.into()).unwrap_or(Value::Null),
            avm1,
            uc,
        )?;

        Ok(object.into())
    }
}
