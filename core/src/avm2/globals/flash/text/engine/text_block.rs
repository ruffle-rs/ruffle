use ruffle_macros::istr;

use crate::avm2::activation::Activation;
use crate::avm2::error::Error;
use crate::avm2::globals::flash::display::display_object::initialize_for_allocator;
use crate::avm2::globals::methods::flash_text_engine_content_element as element_methods;
use crate::avm2::globals::slots::flash_text_engine_content_element as element_slots;
use crate::avm2::globals::slots::flash_text_engine_element_format as format_slots;
use crate::avm2::globals::slots::flash_text_engine_font_description as font_desc_slots;
use crate::avm2::globals::slots::flash_text_engine_text_block as block_slots;
use crate::avm2::globals::slots::flash_text_engine_text_line as line_slots;
use crate::avm2::object::{Object, TObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2_stub_method;
use crate::display_object::{EditText, TDisplayObject};
use crate::html::TextFormat;
use crate::string::WStr;

pub fn create_text_line<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    avm2_stub_method!(activation, "flash.text.TextBlock", "createTextLine");

    let previous_text_line = args.try_get_object(activation, 0);
    let width = args.get_f64(activation, 1)?;

    let content = this.get_slot(block_slots::_CONTENT);

    let content = if matches!(content, Value::Null) {
        return Ok(Value::Null);
    } else {
        content
    };

    let text = match previous_text_line {
        Some(_) => {
            // Some SWFs rely on eventually getting `null` from createLineText.
            // TODO: Support multiple lines
            this.set_slot(
                block_slots::_TEXT_LINE_CREATION_RESULT,
                istr!("complete").into(),
                activation,
            )?;
            return Ok(Value::Null);
        }
        // Get the content element's text property (it's a getter).
        // TODO: GraphicElement?
        None => {
            let txt = content
                .call_method(element_methods::GET_TEXT, &[], activation)
                .unwrap_or_else(|_| istr!("").into());

            if matches!(txt, Value::Null) {
                // FP returns a null TextLine when `o` is null- note that
                // `o` is already coerced to a String because of the AS bindings.
                return Ok(Value::Null);
            } else {
                txt.coerce_to_string(activation)
                    .expect("Guaranteed by AS bindings")
            }
        }
    };

    let class = activation.avm2().classes().textline;
    let movie = activation.caller_movie_or_root();

    // FIXME: TextLine should be its own DisplayObject
    let display_object: EditText =
        EditText::new_fte(activation.context, movie, 0.0, 0.0, width, 15.0);

    display_object.set_text(text.as_wstr(), activation.context);

    // FIXME: This needs to use `intrinsic_bounds` to measure the width
    // of the provided text, and set the width of the EditText to that.
    // Some games depend on this (e.g. Realm Grinder).

    let content = content.as_object().unwrap();
    let element_format = content.get_slot(element_slots::_ELEMENT_FORMAT).as_object();

    apply_format(activation, display_object, text.as_wstr(), element_format)?;

    let instance = initialize_for_allocator(activation, display_object.into(), class)?;
    class.call_init(instance.into(), &[], activation)?;

    instance.set_slot(line_slots::_TEXT_BLOCK, this.into(), activation)?;

    instance.set_slot(line_slots::_SPECIFIED_WIDTH, args.get_value(1), activation)?;

    instance.set_slot(line_slots::_RAW_TEXT_LENGTH, text.len().into(), activation)?;

    this.set_slot(
        block_slots::_TEXT_LINE_CREATION_RESULT,
        istr!("success").into(),
        activation,
    )?;

    this.set_slot(block_slots::_FIRST_LINE, instance.into(), activation)?;

    Ok(instance.into())
}

fn apply_format<'gc>(
    activation: &mut Activation<'_, 'gc>,
    display_object: EditText<'gc>,
    text: &WStr,
    element_format: Option<Object<'gc>>,
) -> Result<(), Error<'gc>> {
    if let Some(element_format) = element_format {
        // TODO: Support more ElementFormat properties
        let color = element_format
            .get_slot(format_slots::_COLOR)
            .coerce_to_u32(activation)?;
        let size = element_format
            .get_slot(format_slots::_FONT_SIZE)
            .coerce_to_number(activation)?;

        let (font, bold, italic, is_device_font) = if let Value::Object(font_description) =
            element_format.get_slot(format_slots::_FONT_DESCRIPTION)
        {
            (
                Some(
                    font_description
                        .get_slot(font_desc_slots::_FONT_NAME)
                        .coerce_to_string(activation)?
                        .as_wstr()
                        .into(),
                ),
                Some(
                    &font_description
                        .get_slot(font_desc_slots::_FONT_WEIGHT)
                        .coerce_to_string(activation)?
                        == b"bold",
                ),
                Some(
                    &font_description
                        .get_slot(font_desc_slots::_FONT_POSTURE)
                        .coerce_to_string(activation)?
                        == b"italic",
                ),
                &font_description
                    .get_slot(font_desc_slots::_FONT_LOOKUP)
                    .coerce_to_string(activation)?
                    == b"device",
            )
        } else {
            (None, None, None, true)
        };

        let format = TextFormat {
            color: Some(swf::Color::from_rgb(color, 0xFF)),
            size: Some(size),
            font,
            bold,
            italic,
            ..TextFormat::default()
        };

        display_object.set_is_device_font(activation.context, is_device_font);
        display_object.set_text_format(0, text.len(), format.clone(), activation.context);
        display_object.set_new_text_format(format, activation.context);
    } else {
        display_object.set_is_device_font(activation.context, true);
    }

    display_object.set_word_wrap(true, activation.context);

    let measured_text = display_object.measure_text(activation.context);

    display_object.set_height(activation.context, measured_text.1.to_pixels());

    Ok(())
}
