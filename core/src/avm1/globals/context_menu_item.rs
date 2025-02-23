use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::object::TObject;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::Object;
use crate::avm1::{ScriptObject, Value};
use crate::string::{AvmString, StringContext};
use ruffle_macros::istr;

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "copy" => method(copy; DONT_ENUM | DONT_DELETE);
};

pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let caption = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?;
    let callback = args.get(1).map(|v| v.coerce_to_object(activation));
    let separator_before = args
        .get(2)
        .unwrap_or(&false.into())
        .as_bool(activation.swf_version());
    let enabled = args
        .get(3)
        .unwrap_or(&true.into())
        .as_bool(activation.swf_version());
    let visible = args
        .get(4)
        .unwrap_or(&true.into())
        .as_bool(activation.swf_version());

    this.set(istr!("caption"), caption.into(), activation)?;

    if let Some(callback) = callback {
        this.set(istr!("onSelect"), callback.into(), activation)?;
    }

    this.set(
        istr!("separatorBefore"),
        separator_before.into(),
        activation,
    )?;
    this.set(istr!("enabled"), enabled.into(), activation)?;
    this.set(istr!("visible"), visible.into(), activation)?;

    Ok(this.into())
}

pub fn copy<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let caption = this
        .get(istr!("caption"), activation)?
        .coerce_to_string(activation)?
        .to_string();
    let callback = this
        .get(istr!("onSelect"), activation)?
        .coerce_to_object(activation);

    let enabled = this
        .get(istr!("enabled"), activation)?
        .as_bool(activation.swf_version());
    let separator_before = this
        .get(istr!("separatorBefore"), activation)?
        .as_bool(activation.swf_version());
    let visible = this
        .get(istr!("visible"), activation)?
        .as_bool(activation.swf_version());

    let constructor = activation
        .context
        .avm1
        .prototypes()
        .context_menu_item_constructor;
    let copy = constructor.construct(
        activation,
        &[
            AvmString::new_utf8(activation.gc(), caption).into(),
            callback.into(),
            separator_before.into(),
            enabled.into(),
            visible.into(),
        ],
    )?;

    Ok(copy)
}

pub fn create_proto<'gc>(
    context: &mut StringContext<'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let object = ScriptObject::new(context, Some(proto));
    define_properties_on(PROTO_DECLS, context, object, fn_proto);
    object.into()
}
