use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::events::EventData;
use crate::avm2::method::Method;
use crate::avm2::method::NativeMethodImpl;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::Object;
use crate::avm2::object::TObject;
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{GcCell, MutationContext};

/// Implements `flash.events.IOErrorEvent`'s instance constructor.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        activation.super_init(this, args)?; // ErrorEvent, Event use these
    }
    Ok(Value::Undefined)
}

/// Implements `flash.events.IOErrorEvent`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements `text`'s getter.
// FIXME - we should define the ancestor class `TextEvent`
// and declare this getter there
pub fn text<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(evt) = this.as_event() {
            if let EventData::IOError { text } = evt.event_data() {
                return Ok(Value::String(*text));
            }
        }
    }
    Ok(Value::Undefined)
}

/// Construct `IOErrorEvent`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.events"), "IOErrorEvent"),
        // TODO: this should derive IOErrorEvent -> ErrorEvent -> TextEvent -> Event
        Some(QName::new(Namespace::package("flash.events"), "Event").into()),
        Method::from_builtin(instance_init, "<IOErrorEvent instance initializer>", mc),
        Method::from_builtin(class_init, "<IOErrorEvent class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    write.set_attributes(ClassAttributes::SEALED);

    const CONSTANTS: &[(&str, &str)] = &[("IO_ERROR", "ioError")];

    write.define_public_constant_string_class_traits(CONSTANTS);

    const PUBLIC_INSTANCE_PROPERTIES: &[(
        &str,
        Option<NativeMethodImpl>,
        Option<NativeMethodImpl>,
    )] = &[("text", Some(text), None)];
    write.define_public_builtin_instance_properties(mc, PUBLIC_INSTANCE_PROPERTIES);

    class
}
