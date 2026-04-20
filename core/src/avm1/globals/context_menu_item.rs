use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::property_decl::{DeclContext, StaticDeclarations, SystemClass};
use crate::avm1::{Object, Value};
use crate::string::AvmString;
use ruffle_macros::istr;

const PROTO_DECLS: StaticDeclarations = declare_static_properties! {
    "copy" => method(copy; DONT_ENUM | DONT_DELETE);
};

pub fn create_class<'gc>(
    context: &mut DeclContext<'_, 'gc>,
    super_proto: Object<'gc>,
) -> SystemClass<'gc> {
    let class = context.class(constructor, super_proto);
    context.define_properties_on(class.proto, PROTO_DECLS(context));
    class
}

fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let caption = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?;
    let callback = args.get(1).copied();
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
        this.set(istr!("onSelect"), callback, activation)?;
    }

    this.set(
        istr!("separatorBefore"),
        separator_before.into(),
        activation,
    )?;
    this.set(istr!("enabled"), enabled.into(), activation)?;
    this.set(istr!("visible"), visible.into(), activation)?;

    Ok(Value::Undefined)
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
        .coerce_to_object_or_bare(activation)?;

    let enabled = this
        .get(istr!("enabled"), activation)?
        .as_bool(activation.swf_version());
    let separator_before = this
        .get(istr!("separatorBefore"), activation)?
        .as_bool(activation.swf_version());
    let visible = this
        .get(istr!("visible"), activation)?
        .as_bool(activation.swf_version());

    let constructor = activation.prototypes().context_menu_item_constructor;
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
