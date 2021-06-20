//! `flash.events.Event` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{event_allocator, EventObject, Object, TObject};
use crate::avm2::string::AvmString;
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{GcCell, MutationContext};

/// Implements `flash.events.Event`'s instance constructor.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        activation.super_init(this, &[])?;

        if let Some(mut evt) = this.as_event_mut(activation.context.gc_context) {
            evt.set_event_type(
                args.get(0)
                    .cloned()
                    .unwrap_or(Value::Undefined)
                    .coerce_to_string(activation)?,
            );
            evt.set_bubbles(
                args.get(1)
                    .cloned()
                    .unwrap_or(Value::Bool(false))
                    .coerce_to_boolean(),
            );
            evt.set_cancelable(
                args.get(2)
                    .cloned()
                    .unwrap_or(Value::Bool(false))
                    .coerce_to_boolean(),
            );
        }
    }

    Ok(Value::Undefined)
}

/// Implements `flash.events.Event`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements `bubbles` property's getter
pub fn bubbles<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(evt) = this.unwrap().as_event() {
        return Ok(evt.is_bubbling().into());
    }

    Ok(Value::Undefined)
}

/// Implements `cancelable` property's getter
pub fn cancelable<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(evt) = this.unwrap().as_event() {
        return Ok(evt.is_cancelable().into());
    }

    Ok(Value::Undefined)
}

/// Implements `type` property's getter
pub fn get_type<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(evt) = this.unwrap().as_event() {
        return Ok(evt.event_type().into());
    }

    Ok(Value::Undefined)
}

/// Implements `target` property's getter
pub fn target<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(evt) = this.unwrap().as_event() {
        return Ok(evt.target().map(|o| o.into()).unwrap_or(Value::Null));
    }

    Ok(Value::Undefined)
}

/// Implements `currentTarget` property's getter
pub fn current_target<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(evt) = this.unwrap().as_event() {
        return Ok(evt
            .current_target()
            .map(|o| o.into())
            .unwrap_or(Value::Null));
    }

    Ok(Value::Undefined)
}

/// Implements `eventPhase` property's getter
pub fn event_phase<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(evt) = this.unwrap().as_event() {
        let event_phase = evt.phase() as u32;
        return Ok(event_phase.into());
    }

    Ok(Value::Undefined)
}

/// Implements `clone`
pub fn clone<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(evt) = this.unwrap().as_event() {
        let evt_class = activation.avm2().classes().event;

        return Ok(EventObject::from_event(activation, evt_class, evt.clone())?.into());
    }

    Ok(Value::Undefined)
}

/// Implements `formatToString`
pub fn format_to_string<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    use std::fmt::Write;

    if let Some(mut this) = this {
        let class_name = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_string(activation)?;

        let mut stringified_params = String::new();
        if let Some(params) = args.get(1..) {
            for param_name in params {
                let param_name = QName::dynamic_name(match param_name {
                    Value::Undefined | Value::Null => "null".into(),
                    _ => param_name.coerce_to_string(activation)?,
                });

                let param_value = this
                    .get_property(this, &param_name, activation)?
                    .coerce_to_debug_string(activation)?;
                write!(
                    stringified_params,
                    " {}={}",
                    param_name.local_name(),
                    param_value
                )
                .unwrap();
            }
        }

        return Ok(AvmString::new(
            activation.context.gc_context,
            format!("[{}{}]", class_name, stringified_params),
        )
        .into());
    }

    Ok(Value::Undefined)
}

/// Implements `isDefaultPrevented`
pub fn is_default_prevented<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(evt) = this.unwrap().as_event() {
        return Ok(evt.is_cancelled().into());
    }

    Ok(Value::Undefined)
}

/// Implements `preventDefault`
pub fn prevent_default<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut evt) = this.unwrap().as_event_mut(activation.context.gc_context) {
        evt.cancel();
    }

    Ok(Value::Undefined)
}

/// Implements `stopPropagation`
pub fn stop_propagation<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut evt) = this.unwrap().as_event_mut(activation.context.gc_context) {
        evt.stop_propagation();
    }

    Ok(Value::Undefined)
}

/// Implements `stopImmediatePropagation`
pub fn stop_immediate_propagation<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut evt) = this.unwrap().as_event_mut(activation.context.gc_context) {
        evt.stop_immediate_propagation();
    }

    Ok(Value::Undefined)
}

/// Implements `toString`
pub fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        return this.value_of(activation.context.gc_context);
    }

    Ok(Value::Undefined)
}

/// Construct `Event`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.events"), "Event"),
        Some(QName::new(Namespace::public(), "Object").into()),
        Method::from_builtin(instance_init, "<Event instance initializer>", mc),
        Method::from_builtin(class_init, "<Event class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    write.set_attributes(ClassAttributes::SEALED);
    write.set_instance_allocator(event_allocator);

    const PUBLIC_INSTANCE_PROPERTIES: &[(
        &str,
        Option<NativeMethodImpl>,
        Option<NativeMethodImpl>,
    )] = &[
        ("bubbles", Some(bubbles), None),
        ("cancelable", Some(cancelable), None),
        ("type", Some(get_type), None),
        ("target", Some(target), None),
        ("currentTarget", Some(current_target), None),
        ("eventPhase", Some(event_phase), None),
    ];
    write.define_public_builtin_instance_properties(mc, PUBLIC_INSTANCE_PROPERTIES);

    const PUBLIC_INSTANCE_METHODS: &[(&str, NativeMethodImpl)] = &[
        ("clone", clone),
        ("formatToString", format_to_string),
        ("isDefaultPrevented", is_default_prevented),
        ("preventDefault", prevent_default),
        ("stopPropagation", stop_propagation),
        ("stopImmediatePropagation", stop_immediate_propagation),
        ("toString", to_string),
    ];
    write.define_public_builtin_instance_methods(mc, PUBLIC_INSTANCE_METHODS);

    const CONSTANTS: &[(&str, &str)] = &[
        ("ACTIVATE", "activate"),
        ("ADDED", "added"),
        ("ADDED_TO_STAGE", "addedToStage"),
        ("BROWSER_ZOOM_CHANGE", "browserZoomChange"),
        ("CANCEL", "cancel"),
        ("CHANGE", "change"),
        ("CHANNEL_MESSAGE", "channelMessage"),
        ("CHANNEL_STATE", "channelState"),
        ("CLEAR", "clear"),
        ("CLOSE", "close"),
        ("CLOSING", "closing"),
        ("COMPLETE", "complete"),
        ("CONNECT", "connect"),
        ("CONTEXT3D_CREATE", "context3DCreate"),
        ("COPY", "copy"),
        ("CUT", "cut"),
        ("DEACTIVATE", "deactivate"),
        ("DISPLAYING", "displaying"),
        ("ENTER_FRAME", "enterFrame"),
        ("EXIT_FRAME", "exitFrame"),
        ("EXITING", "exiting"),
        ("FRAME_CONSTRUCTED", "frameConstructed"),
        ("FRAME_LABEL", "frameLabel"),
        ("FULLSCREEN", "fullScreen"),
        ("HTML_BOUNDS_CHANGE", "htmlBoundsChange"),
        ("HTML_DOM_INITIALIZE", "htmlDOMInitialize"),
        ("HTML_RENDER", "htmlRender"),
        ("ID3", "id3"),
        ("INIT", "init"),
        ("LOCATION_CHANGE", "locationChange"),
        ("MOUSE_LEAVE", "mouseLeave"),
        ("NETWORK_CHANGE", "networkChange"),
        ("OPEN", "open"),
        ("PASTE", "paste"),
        ("PREPARING", "preparing"),
        ("REMOVED", "removed"),
        ("REMOVED_FROM_STAGE", "removedFromStage"),
        ("RENDER", "render"),
        ("RESIZE", "resize"),
        ("SCROLL", "scroll"),
        ("SELECT", "select"),
        ("SELECT_ALL", "selectAll"),
        ("SOUND_COMPLETE", "soundComplete"),
        ("STANDARD_ERROR_CLOSE", "standardErrorClose"),
        ("STANDARD_INPUT_CLOSE", "standardInputClose"),
        ("STANDARD_OUTPUT_CLOSE", "standardOutputClose"),
        ("SUSPEND", "suspend"),
        ("TAB_CHILDREN_CHANGE", "tabChildrenChange"),
        ("TAB_ENABLED_CHANGE", "tabEnabledChange"),
        ("TAB_INDEX_CHANGE", "tabIndexChange"),
        ("TEXT_INTERACTION_MODE_CHANGE", "textInteractionModeChange"),
        ("TEXTURE_READY", "textureReady"),
        ("UNLOAD", "unload"),
        ("USER_IDLE", "userIdle"),
        ("USER_PRESENT", "userPresent"),
        ("VIDEO_FRAME", "videoFrame"),
        ("WORKER_STATE", "workerState"),
    ];
    write.define_public_constant_string_class_traits(CONSTANTS);

    class
}
