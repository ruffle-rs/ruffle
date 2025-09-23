use crate::avm2::error::{make_error_2136, Error};
use crate::avm2::globals::flash::display::display_object::initialize_for_allocator;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::{Activation, ClassObject, Object, Value};
use crate::avm2_stub_method;
use crate::display_object::Video;

pub fn video_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let video_class = activation.avm2().classes().video.inner_class_definition();

    let mut target_class = Some(class.inner_class_definition());
    while let Some(target) = target_class {
        if target == video_class {
            let movie = activation.caller_movie_or_root();
            let new_do = Video::new(activation.gc(), movie, 0, 0, None);
            return Ok(initialize_for_allocator(
                activation.context,
                new_do.into(),
                class,
            ));
        }

        if let Some((movie, symbol)) = activation
            .context
            .library
            .avm2_class_registry()
            .class_symbol(target)
        {
            let child = activation
                .context
                .library
                .library_for_movie_mut(movie)
                .instantiate_by_id(symbol, activation.context.gc_context);

            if let Some(child) = child {
                return Ok(initialize_for_allocator(activation.context, child, class));
            } else {
                return Err(make_error_2136(activation));
            }
        }

        target_class = target.super_class();
    }

    unreachable!("A Video subclass should have Video in superclass chain");
}

/// Implements `flash.media.Video`'s `init` method, which is called from the constructor
pub fn init<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(video) = this.as_display_object().and_then(|dobj| dobj.as_video()) {
        let width = args.get_i32(0);
        let height = args.get_i32(1);

        video.set_size(width, height);
    }

    Ok(Value::Undefined)
}

pub fn attach_net_stream<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(video) = this.as_display_object().and_then(|dobj| dobj.as_video()) {
        let stream = args.try_get_object(0).map(|o| o.as_netstream().unwrap());

        if let Some(stream) = stream {
            video.attach_netstream(activation.context, stream);
        } else {
            // TODO attachNetStream(null) should clear the current stream
            avm2_stub_method!(
                activation,
                "flash.media.Video",
                "attachNetStream",
                "with null argument"
            );
        }
    }

    Ok(Value::Undefined)
}
