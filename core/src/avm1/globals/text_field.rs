use crate::avm1::function::Executable;
use crate::avm1::property::Attribute::*;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Error, Object, ScriptObject, TObject, UpdateContext, Value};
use crate::display_object::{EditText, TDisplayObject};
use gc_arena::MutationContext;

/// Implements `TextField`
pub fn constructor<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    Ok(Value::Undefined.into())
}

pub fn get_text<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if let Some(display_object) = this.as_display_object() {
        if let Some(text_field) = display_object.as_edit_text() {
            return Ok(text_field.text().into());
        }
    }
    Ok(Value::Undefined.into())
}

pub fn set_text<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if let Some(display_object) = this.as_display_object() {
        if let Some(text_field) = display_object.as_edit_text() {
            if let Some(value) = args.get(0) {
                text_field.set_text(
                    value
                        .to_owned()
                        .coerce_to_string(avm, context)
                        .unwrap_or_else(|_| "undefined".to_string()),
                    context.gc_context,
                )
            }
        }
    }
    Ok(Value::Undefined.into())
}

macro_rules! with_text_field {
    ( $gc_context: ident, $object:ident, $fn_proto: expr, $($name:expr => $fn:expr),* ) => {{
        $(
            $object.force_set_function(
                $name,
                |avm, context: &mut UpdateContext<'_, 'gc, '_>, this, args| -> Result<ReturnValue<'gc>, Error> {
                    if let Some(display_object) = this.as_display_object() {
                        if let Some(text_field) = display_object.as_edit_text() {
                            return $fn(text_field, avm, context, args);
                        }
                    }
                    Ok(Value::Undefined.into())
                } as crate::avm1::function::NativeFunction<'gc>,
                $gc_context,
                DontDelete | ReadOnly | DontEnum,
                $fn_proto
            );
        )*
    }};
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let mut object = ScriptObject::object(gc_context, Some(proto));

    with_text_field!(
        gc_context,
        object,
        Some(fn_proto),
        "toString" => |text_field: EditText<'gc>, _avm: &mut Avm1<'gc>, _context: &mut UpdateContext<'_, 'gc, '_>, _args| {
            Ok(text_field.path().into())
        }
    );

    object.into()
}

pub fn attach_virtual_properties<'gc>(gc_context: MutationContext<'gc, '_>, object: Object<'gc>) {
    object.add_property(
        gc_context,
        "text",
        Executable::Native(get_text),
        Some(Executable::Native(set_text)),
        DontDelete | ReadOnly | DontEnum,
    );
}
