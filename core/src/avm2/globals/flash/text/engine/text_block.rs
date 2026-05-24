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
use crate::display_object::{EditText, FteTextLine, TDisplayObject};
use crate::html::TextFormat;
use crate::string::WStr;

pub fn create_text_line<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .expect("TextBlock native method receiver must be an object");

    let content = this.get_slot(block_slots::_CONTENT);
    if matches!(content, Value::Null) {
        return Ok(Value::Null);
    }
    let previous_text_line = args.try_get_object(0);
    let text = match previous_text_line {
        Some(_) => {
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
                return Ok(Value::Null);
            } else {
                txt.coerce_to_string(activation)?
            }
        }
    };

    let movie = activation.caller_movie_or_root();
    let fallback = EditText::new_fte(
        activation.context,
        movie.clone(),
        0.0,
        0.0,
        args.get_f64(1),
        15.0,
    );
    fallback.set_text(text.as_wstr(), activation.context);
    let content_obj = content.as_object().unwrap();
    let element_format = content_obj
        .get_slot(element_slots::_ELEMENT_FORMAT)
        .as_object();
    apply_format(activation, fallback, text.as_wstr(), element_format)?;

    let fte = FteTextLine::new(activation.context, movie, Some(fallback));
    let class = activation.avm2().classes().textline;
    let instance = initialize_for_allocator(activation.context, fte.into(), class);

    instance.set_slot(line_slots::_TEXT_BLOCK, this.into(), activation)?;
    instance.set_slot(line_slots::_SPECIFIED_WIDTH, args.get_value(1), activation)?;
    instance.set_slot(
        line_slots::_RAW_TEXT_LENGTH,
        Value::from_usize_lossy(text.len()),
        activation,
    )?;

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
        display_object.set_new_text_format(format);
    } else {
        display_object.set_is_device_font(activation.context, true);
    }

    display_object.set_word_wrap(true, activation.context);
    let measured_text = display_object.measure_text(activation.context);
    display_object.set_height(activation.context, measured_text.1.to_pixels());

    Ok(())
}
