use crate::avm2::activation::Activation;
use crate::avm2::error::Error;
use crate::avm2::globals::flash::display::display_object::initialize_for_allocator;
use crate::avm2::object::{Object, TObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::Multiname;
use crate::avm2_stub_method;
use crate::display_object::{EditText, TDisplayObject};
use crate::html::TextFormat;
use crate::string::WStr;

pub fn create_text_line<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(activation, "flash.text.TextBlock", "createTextLine");

    let previous_text_line = args.try_get_object(activation, 0);
    let width = args.get_f64(activation, 1)?;

    let content = this.get_public_property("content", activation)?;

    let content = if matches!(content, Value::Null) {
        return Ok(Value::Null);
    } else {
        content.as_object().unwrap()
    };

    let text = match previous_text_line {
        Some(_) => {
            // Some SWFs rely on eventually getting `null` from createLineText.
            this.set_property(
                &Multiname::new(
                    activation.avm2().flash_text_engine_internal,
                    "_textLineCreationResult",
                ),
                "complete".into(),
                activation,
            )?;
            return Ok(Value::Null);
        }
        // Get the content element's text property.
        // TODO: GraphicElement and GroupElement
        None => content
            .get_public_property("text", activation)
            .and_then(|o| {
                if matches!(o, Value::Null) {
                    Ok("".into())
                } else {
                    o.coerce_to_string(activation)
                }
            })
            .unwrap_or("".into()),
    };

    let class = activation.avm2().classes().textline;
    let movie = activation.context.swf.clone();

    // FIXME: TextLine should be its own DisplayObject
    let display_object: EditText =
        EditText::new(&mut activation.context, movie, 0.0, 0.0, width, 15.0).into();

    display_object.set_text(text.as_wstr(), &mut activation.context);

    let element_format = content
        .get_public_property("elementFormat", activation)?
        .as_object();

    let new_height = apply_format(activation, display_object, text.as_wstr(), element_format)?;

    display_object.set_height(activation.gc(), new_height);

    let instance = initialize_for_allocator(activation, display_object.into(), class)?;
    class.call_native_init(instance.into(), &[], activation)?;

    instance.set_property(
        &Multiname::new(activation.avm2().flash_text_engine_internal, "_textBlock"),
        this.into(),
        activation,
    )?;

    instance.set_property(
        &Multiname::new(
            activation.avm2().flash_text_engine_internal,
            "_specifiedWidth",
        ),
        args.get_value(1),
        activation,
    )?;

    instance.set_property(
        &Multiname::new(
            activation.avm2().flash_text_engine_internal,
            "_rawTextLength",
        ),
        text.len().into(),
        activation,
    )?;

    this.set_property(
        &Multiname::new(
            activation.avm2().flash_text_engine_internal,
            "_textLineCreationResult",
        ),
        "success".into(),
        activation,
    )?;

    this.set_property(
        &Multiname::new(activation.avm2().flash_text_engine_internal, "_firstLine"),
        instance.into(),
        activation,
    )?;

    Ok(instance.into())
}

fn apply_format<'gc>(
    activation: &mut Activation<'_, 'gc>,
    display_object: EditText<'gc>,
    text: &WStr,
    element_format: Option<Object<'gc>>,
) -> Result<f64, Error<'gc>> {
    if let Some(element_format) = element_format {
        let color = element_format
            .get_public_property("color", activation)?
            .coerce_to_u32(activation)?;
        let size = element_format
            .get_public_property("fontSize", activation)?
            .coerce_to_number(activation)?;

        let format = TextFormat {
            color: Some(swf::Color::from_rgb(color, 0xFF)),
            size: Some(size),
            ..TextFormat::default()
        };

        display_object.set_text_format(0, text.len(), format.clone(), &mut activation.context);
        display_object.set_new_text_format(format, &mut activation.context);

        return Ok(size * 1.2 + 3.0);
    }

    Ok(15.0)
}
