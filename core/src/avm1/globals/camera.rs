//! Camera object

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::property_decl::{DeclContext, StaticDeclarations, SystemClass};
use crate::avm1::{Object, Value};
use crate::avm1_stub;

const PROTO_DECLS: StaticDeclarations = declare_static_properties! {
    "setMode" => method(set_mode; DONT_ENUM | DONT_DELETE);
    "setQuality" => method(set_quality; DONT_ENUM | DONT_DELETE);
    "setKeyFrameInterval" => method(set_key_frame_interval; DONT_ENUM | DONT_DELETE);
    "setMotionLevel" => method(set_motion_level; DONT_ENUM | DONT_DELETE);
    "setLoopback" => method(set_loopback; DONT_ENUM | DONT_DELETE);
    "setCursor" => method(set_cursor; DONT_ENUM | DONT_DELETE);
};

const OBJECT_DECLS: StaticDeclarations = declare_static_properties! {
    "get" => method(get);
    "names" => property(get_names);
};

pub fn create_class<'gc>(
    context: &mut DeclContext<'_, 'gc>,
    super_proto: Object<'gc>,
) -> SystemClass<'gc> {
    let class = context.empty_class(super_proto);
    context.define_properties_on(class.proto, PROTO_DECLS(context));
    context.define_properties_on(class.constr, OBJECT_DECLS(context));
    class
}

fn get<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "Camera", "get");
    // Camera.get() returns null when there's no camera.
    Ok(Value::Null)
}

fn get_names<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "Camera", "names");
    activation
        .prototypes()
        .array_constructor
        .construct(activation, &[])
}

fn set_mode<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "Camera", "setMode");
    Ok(Value::Undefined)
}

fn set_quality<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "Camera", "setQuality");
    Ok(Value::Undefined)
}

fn set_key_frame_interval<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "Camera", "setKeyFrameInterval");
    Ok(Value::Undefined)
}

fn set_motion_level<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "Camera", "setMotionLevel");
    Ok(Value::Undefined)
}

fn set_loopback<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "Camera", "setLoopback");
    Ok(Value::Undefined)
}

fn set_cursor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "Camera", "setCursor");
    Ok(Value::Undefined)
}
