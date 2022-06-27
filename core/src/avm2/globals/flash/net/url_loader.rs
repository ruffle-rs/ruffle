//! `flash.net.URLLoader` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::TObject;
use crate::avm2::value::Value;
use crate::avm2::{Error, Object};
use crate::backend::navigator::RequestOptions;
use crate::loader::DataFormat;
use gc_arena::{GcCell, MutationContext};

/// Implements `flash.net.URLLoader`'s class constructor.
fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements `flash.net.URLLoader`'s instance constructor.
fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut this) = this {
        activation.super_init(this, &[])?;
        this.set_property(
            &QName::dynamic_name("dataFormat").into(),
            "text".into(),
            activation,
        )?;
        this.set_property(
            &QName::dynamic_name("data").into(),
            Value::Undefined,
            activation,
        )?;

        if let Some(request) = args.get(0) {
            if request != &Value::Null {
                load(activation, Some(this), args)?;
            }
        }
    }
    Ok(Value::Undefined)
}

fn bytes_loaded<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    // For now, just use `bytes_total`. The `bytesLoaded` value
    // should really update as the download progresses, instead
    // of jumping at completion from 0 to the total length
    log::warn!("URLLoader.bytesLoaded - not yet implemented");
    bytes_total(activation, this, args)
}

fn bytes_total<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        let data = this.get_property(&QName::dynamic_name("data").into(), activation)?;

        if let Value::Object(data) = data {
            // `bytesTotal` should be 0 while the download is in progress
            // (the `data` property is only set after the download is completed)
            if let Some(array) = data.as_bytearray() {
                return Ok(array.len().into());
            } else {
                return Err(format!("Unexpected value for `data` property: {:?}", data).into());
            }
        } else if let Value::String(data) = data {
            return Ok(data.len().into());
        }
        return Ok(0.into());
    }
    Ok(Value::Undefined)
}

fn load<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        let request = match args.get(0) {
            Some(Value::Object(request)) => request,
            // This should never actually happen
            _ => return Ok(Value::Undefined),
        };

        let data_format = this
            .get_property(&QName::dynamic_name("dataFormat").into(), activation)?
            .coerce_to_string(activation)?;

        let data_format = if &data_format == b"binary" {
            DataFormat::Binary
        } else if &data_format == b"text" {
            DataFormat::Text
        } else if &data_format == b"variables" {
            DataFormat::Variables
        } else {
            return Err(format!("Unknown data format: {}", data_format).into());
        };

        return spawn_fetch(activation, this, request, data_format);
    }
    Ok(Value::Undefined)
}

fn spawn_fetch<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    loader_object: Object<'gc>,
    url_request: &Object<'gc>,
    data_format: DataFormat,
) -> Result<Value<'gc>, Error> {
    let url = url_request
        .get_property(&QName::dynamic_name("url").into(), activation)?
        .coerce_to_string(activation)?;

    let url = url.to_utf8_lossy();

    let future = activation.context.load_manager.load_data_into_url_loader(
        activation.context.player.clone(),
        loader_object,
        &url,
        // FIXME - get these from the `URLRequest`
        RequestOptions::get(),
        data_format,
    );
    activation.context.navigator.spawn_future(future);
    Ok(Value::Undefined)
}

pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.net"), "URLLoader"),
        Some(QName::new(Namespace::package("flash.events"), "EventDispatcher").into()),
        Method::from_builtin(instance_init, "<URLLoader instance initializer>", mc),
        Method::from_builtin(class_init, "<URLLoader class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    const PUBLIC_INSTANCE_PROPERTIES: &[(
        &str,
        Option<NativeMethodImpl>,
        Option<NativeMethodImpl>,
    )] = &[
        ("bytesLoaded", Some(bytes_loaded), None),
        ("bytesTotal", Some(bytes_total), None),
    ];
    write.define_public_builtin_instance_properties(mc, PUBLIC_INSTANCE_PROPERTIES);

    const PUBLIC_INSTANCE_SLOTS: &[(&str, &str, &str)] =
        &[("data", "", "Object"), ("dataFormat", "", "String")];
    write.define_public_slot_instance_traits(PUBLIC_INSTANCE_SLOTS);

    const PUBLIC_INSTANCE_METHODS: &[(&str, NativeMethodImpl)] = &[("load", load)];
    write.define_public_builtin_instance_methods(mc, PUBLIC_INSTANCE_METHODS);

    class
}
