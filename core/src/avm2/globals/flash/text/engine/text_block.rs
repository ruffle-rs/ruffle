use ruffle_macros::istr;

use crate::avm2::activation::Activation;
use crate::avm2::error::Error;
use crate::avm2::globals::flash::display::display_object::initialize_for_allocator;
use crate::avm2::globals::methods::flash_text_engine_content_element as element_methods;
use crate::avm2::globals::slots::flash_text_engine_text_block as block_slots;
use crate::avm2::globals::slots::flash_text_engine_text_line as line_slots;
use crate::avm2::object::{Object, TObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2_stub_method;
use crate::display_object::{EditText, TDisplayObject, TextLine};
use crate::fte::FontLookupValue;
use crate::fte::FontPostureValue;
use crate::fte::FontWeightValue;
use crate::html::TextFormat;
use crate::string::WStr;

pub fn create_text_line<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    avm2_stub_method!(activation, "flash.text.TextBlock", "createTextLine");

    let previous_text_line = args.try_get_object(0);
    let width = args.get_f64(1);

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

    let fallback = EditText::new_fte(activation.context, movie.clone(), 0.0, 0.0, width, 15.0);

    fallback.set_text(text.as_wstr(), activation.context);

    // FIXME: This needs to use `intrinsic_bounds` to measure the width
    // of the provided text, and set the width of the EditText to that.
    // Some games depend on this (e.g. Realm Grinder).

    let content_obj = content.as_object().unwrap();
    let element_format = content_obj
        .as_content_element_object()
        .and_then(|ce| ce.element_format())
        .map(|ef| ef.into());
    apply_format(activation, fallback, text.as_wstr(), element_format)?;

    let text_line = TextLine::new(activation.context, movie, fallback);
    let instance = initialize_for_allocator(activation.context, text_line.into(), class);

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
    if let Some(ef) = element_format.and_then(|o| o.as_element_format_object()) {
        // TODO: Support more ElementFormat properties
        let (font, bold, italic, is_device_font) = if let Some(fd) = ef.font_description() {
            (
                Some(fd.font_name().as_wstr().into()),
                Some(fd.font_weight() == FontWeightValue::Bold),
                Some(fd.font_posture() == FontPostureValue::Italic),
                fd.font_lookup() == FontLookupValue::Device,
            )
        } else {
            (None, None, None, true)
        };

        let format = TextFormat {
            color: Some(ef.color()),
            size: Some(ef.font_size()),
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
