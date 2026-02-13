//! Camera object

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::property_decl::{DeclContext, StaticDeclarations, SystemClass};
use crate::avm1::{Object, Value};
use crate::avm1_stub;

const PROTO_DECLS: StaticDeclarations = declare_static_properties! {
    use fn method;
    "setMode" => method(SET_MODE; DONT_ENUM | DONT_DELETE);
    "setQuality" => method(SET_QUALITY; DONT_ENUM | DONT_DELETE);
    "setKeyFrameInterval" => method(SET_KEY_FRAME_INTERVAL; DONT_ENUM | DONT_DELETE);
    "setMotionLevel" => method(SET_MOTION_LEVEL; DONT_ENUM | DONT_DELETE);
    "setLoopback" => method(SET_LOOPBACK; DONT_ENUM | DONT_DELETE);
    "setCursor" => method(SET_CURSOR; DONT_ENUM | DONT_DELETE);
};

const OBJECT_DECLS: StaticDeclarations = declare_static_properties! {
    // In playerglobals.swf, this is a bytecode function delegating to the internal native impl.
    "get" => function(|a, this, args| method(a, this, args, method::INTERNAL_GET));
    use fn method;
    "names" => property(GET_NAMES);
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

pub mod method {
    pub const SET_MODE: u16 = 0;
    pub const SET_QUALITY: u16 = 1;
    pub const SET_KEY_FRAME_INTERVAL: u16 = 2;
    pub const SET_MOTION_LEVEL: u16 = 3;
    pub const SET_LOOPBACK: u16 = 4;
    pub const SET_CURSOR: u16 = 5;

    pub const INTERNAL_GET: u16 = 200;
    pub const GET_NAMES: u16 = 201;
}

pub fn method<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
    index: u16,
) -> Result<Value<'gc>, Error<'gc>> {
    use method::*;
    const CNAME: &str = "Camera";

    match index {
        INTERNAL_GET => {
            avm1_stub!(activation, CNAME, "get");
            // Camera.get() returns null when there's no camera.
            return Ok(Value::Null);
        }
        GET_NAMES => {
            avm1_stub!(activation, CNAME, "names");
            return activation
                .prototypes()
                .array_constructor
                .construct(activation, &[]);
        }
        SET_MODE => avm1_stub!(activation, CNAME, "setMode"),
        SET_QUALITY => avm1_stub!(activation, CNAME, "setQuality"),
        SET_KEY_FRAME_INTERVAL => avm1_stub!(activation, CNAME, "setKeyFrameInterval"),
        SET_MOTION_LEVEL => avm1_stub!(activation, CNAME, "setMotionLevel"),
        SET_LOOPBACK => avm1_stub!(activation, CNAME, "setLoopback"),
        SET_CURSOR => avm1_stub!(activation, CNAME, "setCursor"),
        _ => (),
    }

    Ok(Value::Undefined)
}
