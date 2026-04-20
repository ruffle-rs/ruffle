//! TextSnapshot object

use gc_arena::Collect;
use ruffle_common::avm_string::AvmString;

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::parameters::ParametersExt;
use crate::avm1::property_decl::{DeclContext, StaticDeclarations, SystemClass};
use crate::avm1::{NativeObject, Object, Value};
use crate::avm1_stub;
use crate::context::UpdateContext;
use crate::display_object::{MovieClip, TextSnapshot};

#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct TextSnapshotObject<'gc>(TextSnapshot<'gc>);

impl std::fmt::Debug for TextSnapshotObject<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("TextSnapshotObject")
            .field("text_snapshot", &self.0)
            .finish()
    }
}

impl<'gc> TextSnapshotObject<'gc> {
    pub fn new(context: &mut UpdateContext<'gc>, target: MovieClip<'gc>) -> Self {
        Self(TextSnapshot::new(context, target))
    }

    pub fn text_snapshot(self) -> TextSnapshot<'gc> {
        self.0
    }
}

const PROTO_DECLS: StaticDeclarations = declare_static_properties! {
    "getCount" => method(get_count; VERSION_6);
    "setSelected" => method(set_selected; VERSION_6);
    "getSelected" => method(get_selected; VERSION_6);
    "getText" => method(get_text; VERSION_6);
    "getSelectedText" => method(get_selected_text; VERSION_6);
    "hitTestTextNearPos" => method(hit_test_text_near_pos; VERSION_6);
    "findText" => method(find_text; VERSION_6);
    "setSelectColor" => method(set_select_color; VERSION_6);
    "getTextRunInfo" => method(get_text_run_info; VERSION_6);
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
    if args.len() == 1
        && let Some(target) = args.try_get_object(activation, 0)?
        && let Some(target) = target.as_display_object()
        && let Some(target) = target.as_movie_clip()
    {
        let object = TextSnapshotObject::new(activation.context, target);
        this.set_native(activation.gc(), NativeObject::TextSnapshot(object));
    }
    Ok(Value::Undefined)
}

fn get_count<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let NativeObject::TextSnapshot(object) = this.native() else {
        return Ok(Value::Undefined);
    };

    if !args.is_empty() {
        return Ok(Value::Undefined);
    }

    Ok(object.text_snapshot().count().into())
}

fn set_selected<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "TextSnapshot", "setSelected");
    Ok(Value::Undefined)
}

fn get_selected<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "TextSnapshot", "getSelected");
    Ok(Value::Undefined)
}

fn get_text<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let NativeObject::TextSnapshot(object) = this.native() else {
        return Ok(Value::Undefined);
    };

    let [from, to, ..] = args else {
        return Ok(Value::Undefined);
    };

    if args.len() > 3 {
        return Ok(Value::Undefined);
    }

    let from = from.coerce_to_i32(activation)?;
    let to = to.coerce_to_i32(activation)?;
    let include_newlines = args.get_bool(activation, 2);

    let text = object.text_snapshot().get_text(from, to, include_newlines);
    Ok(AvmString::new(activation.gc(), text).into())
}

fn get_selected_text<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "TextSnapshot", "getSelectedText");
    Ok(Value::Undefined)
}

fn hit_test_text_near_pos<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "TextSnapshot", "hitTestTextNearPos");
    Ok(Value::Undefined)
}

fn set_select_color<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "TextSnapshot", "setSelectColor");
    Ok(Value::Undefined)
}

fn find_text<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let NativeObject::TextSnapshot(object) = this.native() else {
        return Ok(Value::Undefined);
    };

    let [start, text, case_sensitive] = args else {
        return Ok(Value::Undefined);
    };

    let start = start.coerce_to_i32(activation)?;
    let text = text.coerce_to_string(activation)?;
    let case_sensitive = case_sensitive.as_bool(activation.swf_version());

    let index = object
        .text_snapshot()
        .find_text(start, text.as_wstr(), case_sensitive);
    Ok(index.into())
}

fn get_text_run_info<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "TextSnapshot", "getTextRunInfo");
    Ok(Value::Undefined)
}
