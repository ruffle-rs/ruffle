use crate::avm2::{Activation, Error, Object, TObject, Value};
use crate::display_object::{TDisplayObject, Video};

/// Implements `flash.media.Video`'s `init` method, which is called from the constructor
pub fn init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        activation.super_init(this, &[])?;

        if this.as_display_object().is_none() {
            let width = args
                .get(0)
                .cloned()
                .unwrap_or(Value::Undefined)
                .coerce_to_i32(activation)?;
            let height = args
                .get(1)
                .cloned()
                .unwrap_or(Value::Undefined)
                .coerce_to_i32(activation)?;

            let movie = activation.context.swf.clone();
            let new_do = Video::new(
                activation.context.gc_context,
                movie,
                width,
                height,
                Some(this.into()),
            );

            this.init_display_object(&mut activation.context, new_do.into());
        }
    }

    Ok(Value::Undefined)
}

pub fn attach_net_stream<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(video) = this
        .and_then(|o| o.as_display_object())
        .and_then(|dobj| dobj.as_video())
    {
        let source = args.get(0).cloned().and_then(|v| v.as_object());

        if let Some(stream) = source.and_then(|o| o.as_netstream()) {
            video.attach_netstream(&mut activation.context, stream);
        } else {
            return Err(format!(
                "Cannot use value of type {:?} as video source",
                source
                    .and_then(|o| o.instance_of_class_definition())
                    .map(|c| c.read().name().local_name())
                    .unwrap_or_else(|| "Object".into())
            )
            .into());
        }
    }

    Ok(Value::Undefined)
}
