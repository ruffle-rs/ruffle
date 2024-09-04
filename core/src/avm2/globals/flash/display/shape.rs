//! `flash.display.Shape` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::globals::flash::display::display_object::initialize_for_allocator;
use crate::avm2::object::{ClassObject, Object, StageObject, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::Multiname;
use crate::display_object::Graphic;

pub fn shape_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let shape_cls = activation.avm2().classes().shape.inner_class_definition();

    let mut class_def = Some(class.inner_class_definition());
    let orig_class = class;
    while let Some(class) = class_def {
        if class == shape_cls {
            let display_object = Graphic::empty(activation.context).into();
            return initialize_for_allocator(activation, display_object, orig_class);
        }

        if let Some((movie, symbol)) = activation
            .context
            .library
            .avm2_class_registry()
            .class_symbol(class)
        {
            let child = activation
                .context
                .library
                .library_for_movie_mut(movie)
                .instantiate_by_id(symbol, activation.context.gc_context)?;

            return initialize_for_allocator(activation, child, orig_class);
        }
        class_def = class.super_class();
    }
    unreachable!("A Shape subclass should have Shape in superclass chain");
}

/// Implements `graphics`.
pub fn get_graphics<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let namespaces = activation.avm2().namespaces;

    if let Some(dobj) = this.as_display_object() {
        // Lazily initialize the `Graphics` object in a hidden property.
        let graphics = match this.get_property(
            &Multiname::new(namespaces.flash_display_internal, "_graphics"),
            activation,
        )? {
            Value::Undefined | Value::Null => {
                let graphics = Value::from(StageObject::graphics(activation, dobj)?);
                this.set_property(
                    &Multiname::new(namespaces.flash_display_internal, "_graphics"),
                    graphics,
                    activation,
                )?;
                graphics
            }
            graphics => graphics,
        };
        return Ok(graphics);
    }

    Ok(Value::Undefined)
}
