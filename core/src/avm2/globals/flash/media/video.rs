use crate::avm2::globals::flash::display::display_object::initialize_for_allocator;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::{Activation, ClassObject, Error, Object, TObject, Value};
use crate::display_object::{TDisplayObject, Video};

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
            return initialize_for_allocator(activation, new_do.into(), class);
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
                .instantiate_by_id(symbol, activation.context.gc_context)?;

            return initialize_for_allocator(activation, child, class);
        }

        target_class = target.super_class();
    }

    unreachable!("A Video subclass should have Video in superclass chain");
}

/// Implements `flash.media.Video`'s `init` method, which is called from the constructor
pub fn init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(video) = this.as_display_object().and_then(|dobj| dobj.as_video()) {
        let width = args.get_i32(activation, 0)?;
        let height = args.get_i32(activation, 1)?;

        video.set_size(activation.gc(), width, height);
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
        let source = args.get_value(0).as_object();

        if let Some(stream) = source.and_then(|o| o.as_netstream()) {
            video.attach_netstream(activation.context, stream);
        } else {
            return Err(format!(
                "Cannot use value of type {:?} as video source",
                source.map(|o| o.instance_class().name().local_name())
            )
            .into());
        }
    }

    Ok(Value::Undefined)
}
