//! `flash.media.Sound` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::Method;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{sound_allocator, Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::character::Character;
use gc_arena::{GcCell, MutationContext};

/// Implements `flash.media.Sound`'s instance constructor.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        activation.super_init(this, &[])?;

        if this.as_sound().is_none() {
            let class_object = this
                .as_class_object()
                .ok_or("Attempted to construct non-instance Sound.")?;

            if let Some((movie, symbol)) = activation
                .context
                .library
                .avm2_class_registry()
                .class_symbol(class_object)
            {
                if let Some(Character::Sound(sound)) = activation
                    .context
                    .library
                    .library_for_movie_mut(movie)
                    .character_by_id(symbol)
                {
                    this.set_sound(activation.context.gc_context, *sound);
                } else {
                    log::warn!("Attempted to construct subclass of Sound, {}, which is associated with non-Sound character {}", class_object.as_class().expect("Class object is also a class").read().name().local_name(), symbol);
                }
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `flash.media.Sound`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Construct `Sound`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.media"), "Sound"),
        Some(QName::new(Namespace::package("flash.events"), "EventDispatcher").into()),
        Method::from_builtin(instance_init, "<Sound instance initializer>", mc),
        Method::from_builtin(class_init, "<Sound class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    write.set_attributes(ClassAttributes::SEALED);
    write.set_instance_allocator(sound_allocator);

    class
}
