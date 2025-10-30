//! PrintJob object

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::property_decl::{DeclContext, Declaration, SystemClass};
use crate::avm1::{Object, Value};
use crate::avm1_stub;

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "start" => method(start; DONT_ENUM | DONT_DELETE | VERSION_7);
    "addPage" => method(add_page; DONT_ENUM | DONT_DELETE | VERSION_7);
    "send" => method(send; DONT_ENUM | DONT_DELETE | VERSION_7);
    "paperHeight" => property(paper_height; DONT_DELETE | READ_ONLY);
    "paperWidth" => property(paper_width; DONT_DELETE | READ_ONLY);
    "pageHeight" => property(page_height; DONT_DELETE | READ_ONLY);
    "pageWidth" => property(page_width; DONT_DELETE | READ_ONLY);
    "orientation" => property(orientation; DONT_DELETE | READ_ONLY);
};

pub fn create_class<'gc>(
    context: &mut DeclContext<'_, 'gc>,
    super_proto: Object<'gc>,
) -> SystemClass<'gc> {
    let class = context.class(constructor, super_proto);
    context.define_properties_on(class.proto, PROTO_DECLS);
    class
}

fn constructor<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Value::Undefined)
}

fn start<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "PrintJob", "start");
    Ok(Value::Undefined)
}

fn add_page<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "PrintJob", "addPage");
    Ok(Value::Undefined)
}

fn send<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "PrintJob", "send");
    Ok(Value::Undefined)
}

fn paper_height<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "PrintJob", "paperHeight");
    Ok(0.into())
}

fn paper_width<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "PrintJob", "paperWidth");
    Ok(0.into())
}

fn page_height<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "PrintJob", "pageHeight");
    Ok(0.into())
}

fn page_width<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "PrintJob", "pageWidth");
    Ok(0.into())
}

fn orientation<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "PrintJob", "orientation");
    Ok(0.into())
}
