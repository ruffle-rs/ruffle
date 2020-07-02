//! DisplayObject common methods

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::Executable;
use crate::avm1::property::Attribute::*;
use crate::avm1::{Object, ScriptObject, TObject, UpdateContext, Value};
use crate::display_object::{DisplayObject, TDisplayObject};
use enumset::EnumSet;
use gc_arena::MutationContext;

/// Depths used/returned by ActionScript are offset by this amount from depths used inside the SWF/by the VM.
/// The depth of objects placed on the timeline in the Flash IDE start from 0 in the SWF,
/// but are negative when queried from MovieClip.getDepth().
/// Add this to convert from AS -> SWF depth.
pub const AVM_DEPTH_BIAS: i32 = 16384;

/// The maximum depth that the AVM will allow you to swap or attach clips to.
/// What is the derivation of this number...?
pub const AVM_MAX_DEPTH: i32 = 2_130_706_428;

macro_rules! with_display_object {
    ( $gc_context: ident, $object:ident, $fn_proto: expr, $($name:expr => $fn:expr),* ) => {{
        $(
            $object.force_set_function(
                $name,
                |activation, context: &mut UpdateContext<'_, 'gc, '_>, this, args| -> Result<Value<'gc>, Error<'gc>> {
                    if let Some(display_object) = this.as_display_object() {
                        return $fn(display_object, activation, context, args);
                    }
                    Ok(Value::Undefined)
                } as crate::avm1::function::NativeFunction<'gc>,
                $gc_context,
                DontDelete | ReadOnly | DontEnum,
                $fn_proto
            );
        )*
    }};
}

/// Add common display object prototype methods to the given prototype.
pub fn define_display_object_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    mut object: ScriptObject<'gc>,
    fn_proto: Object<'gc>,
) {
    with_display_object!(
        gc_context,
        object,
        Some(fn_proto),
        "getDepth" => get_depth
    );

    object.add_property(
        gc_context,
        "_global",
        Executable::Native(|activation, context, _this, _args| {
            Ok(activation.avm.global_object(context))
        }),
        Some(Executable::Native(overwrite_global)),
        DontDelete | ReadOnly | DontEnum,
    );

    object.add_property(
        gc_context,
        "_root",
        Executable::Native(|activation, context, _this, _args| Ok(activation.root_object(context))),
        Some(Executable::Native(overwrite_root)),
        DontDelete | ReadOnly | DontEnum,
    );

    object.add_property(
        gc_context,
        "_parent",
        Executable::Native(get_parent),
        None,
        DontDelete | ReadOnly | DontEnum,
    );
}

pub fn get_parent<'gc>(
    activation: &mut Activation<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this
        .as_display_object()
        .and_then(|mc| mc.parent())
        .map(|dn| dn.object().coerce_to_object(activation, context))
        .map(Value::Object)
        .unwrap_or(Value::Undefined))
}

pub fn get_depth<'gc>(
    display_object: DisplayObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if activation.current_swf_version() >= 6 {
        let depth = display_object.depth().wrapping_sub(AVM_DEPTH_BIAS);
        Ok(depth.into())
    } else {
        Ok(Value::Undefined)
    }
}

pub fn overwrite_root<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let new_val = args
        .get(0)
        .map(|v| v.to_owned())
        .unwrap_or(Value::Undefined);
    this.define_value(ac.gc_context, "_root", new_val, EnumSet::new());

    Ok(Value::Undefined)
}

pub fn overwrite_global<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let new_val = args
        .get(0)
        .map(|v| v.to_owned())
        .unwrap_or(Value::Undefined);
    this.define_value(ac.gc_context, "_global", new_val, EnumSet::new());

    Ok(Value::Undefined)
}
