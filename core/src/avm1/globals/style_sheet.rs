use crate::avm1::object::{Object, TObject};
use crate::avm1::property_decl::define_properties_on;
use crate::avm1::{
    property_decl::Declaration, ArrayObject, ExecutionReason, NativeObject, ScriptObject,
};
use crate::avm1::{Activation, Error, Value};
use crate::avm1_stub;
use crate::backend::navigator::Request;
use crate::html::{transform_dashes_to_camel_case, CssStream, TextFormat};
use crate::string::{AvmString, StringContext};
use gc_arena::Gc;

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "setStyle" => method(set_style; DONT_ENUM | DONT_DELETE | READ_ONLY | VERSION_7);
    "clear" => method(clear; DONT_ENUM | DONT_DELETE | READ_ONLY | VERSION_7);
    "getStyleNames" => method(get_style_names; DONT_ENUM | DONT_DELETE | READ_ONLY | VERSION_7);
    "load" => method(load; DONT_ENUM | DONT_DELETE | READ_ONLY | VERSION_7);
    "getStyle" => method(get_style; DONT_ENUM | DONT_DELETE | READ_ONLY | VERSION_7);
    "transform" => method(transform; DONT_ENUM | DONT_DELETE | READ_ONLY | VERSION_7);
    "parseCSS" => method(parse_css; DONT_ENUM | DONT_DELETE | READ_ONLY | VERSION_7);
    "parse" => method(parse_css; DONT_ENUM | DONT_DELETE | READ_ONLY | VERSION_7);
};

fn shallow_copy<'gc>(
    activation: &mut Activation<'_, 'gc>,
    value: Value<'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    if let Value::Object(object) = value {
        let object_proto = activation.context.avm1.prototypes().object;
        let result = ScriptObject::new(activation.context.gc_context, Some(object_proto));

        for key in object.get_keys(activation, false) {
            result.set(key, object.get_stored(key, activation)?, activation)?;
        }

        Ok(result.into())
    } else {
        Ok(Value::Null)
    }
}

fn set_style<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if !this.has_property(activation, "_styles".into()) {
        this.set("_styles", ArrayObject::empty(activation).into(), activation)?;
    }
    if !this.has_property(activation, "_css".into()) {
        this.set("_css", ArrayObject::empty(activation).into(), activation)?;
    }
    let css = this
        .get_stored("_css".into(), activation)?
        .coerce_to_object(activation);
    let styles = this
        .get_stored("_styles".into(), activation)?
        .coerce_to_object(activation);
    let name = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?;
    let object = args.get(1).unwrap_or(&Value::Undefined);

    css.set(name, shallow_copy(activation, *object)?, activation)?;
    styles.set(
        name,
        this.call_method(
            "transform".into(),
            &[shallow_copy(activation, *object)?],
            activation,
            ExecutionReason::Special,
        )?,
        activation,
    )?;

    Ok(Value::Undefined)
}

fn get_style<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let css = this
        .get_stored("_css".into(), activation)?
        .coerce_to_object(activation);
    let name = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?;
    let style = css.get(name, activation)?;
    shallow_copy(activation, style)
}

fn get_style_names<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let css = this
        .get_stored("_css".into(), activation)?
        .coerce_to_object(activation);
    Ok(ArrayObject::new(
        activation.context.gc_context,
        activation.context.avm1.prototypes().array,
        css.get_keys(activation, false)
            .into_iter()
            .map(Value::String),
    )
    .into())
}

fn load<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let url = match args.get(0) {
        Some(val) => val.coerce_to_string(activation)?,
        None => return Ok(false.into()),
    };

    let request = Request::get(url.to_utf8_lossy().into_owned());

    let future = activation.context.load_manager.load_stylesheet(
        activation.context.player.clone(),
        this,
        request,
    );
    activation.context.navigator.spawn_future(future);

    Ok(true.into())
}

fn transform<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "TextField.StyleSheet", "transform");

    let text_format = TextFormat::default();

    let proto = activation.context.avm1.prototypes().text_format;
    let object = ScriptObject::new(activation.context.gc_context, Some(proto));
    object.set_native(
        activation.context.gc_context,
        NativeObject::TextFormat(Gc::new(activation.context.gc_context, text_format.into())),
    );
    Ok(object.into())
}

fn parse_css<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let source = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?;

    if let Ok(css) = CssStream::new(&source).parse() {
        for (selector, properties) in css.into_iter() {
            if !selector.is_empty() {
                let proto = activation.context.avm1.prototypes().object;
                let object = ScriptObject::new(activation.context.gc_context, Some(proto));

                for (key, value) in properties.into_iter() {
                    object.set(
                        AvmString::new(activation.gc(), transform_dashes_to_camel_case(key)),
                        AvmString::new(activation.gc(), value).into(),
                        activation,
                    )?;
                }

                set_style(
                    activation,
                    this,
                    &[
                        AvmString::new(activation.gc(), selector).into(),
                        object.into(),
                    ],
                )?;
            }
        }
        Ok(Value::Bool(true))
    } else {
        Ok(Value::Bool(false))
    }
}

fn clear<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    this.set("_styles", ArrayObject::empty(activation).into(), activation)?;
    this.set("_css", ArrayObject::empty(activation).into(), activation)?;
    Ok(Value::Undefined)
}

pub fn constructor<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.into())
}

pub fn create_proto<'gc>(
    context: &mut StringContext<'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let style_sheet_proto = ScriptObject::new(context.gc_context, Some(proto));
    define_properties_on(PROTO_DECLS, context, style_sheet_proto, fn_proto);
    style_sheet_proto.into()
}
