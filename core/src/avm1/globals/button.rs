//! Button prototype

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::globals::bitmap_filter;
use crate::avm1::globals::movie_clip::{new_rectangle, object_to_rectangle};
use crate::avm1::property_decl::{DeclContext, Declaration, SystemClass};
use crate::avm1::ArrayBuilder;
use crate::avm1::{globals, Object, Value};
use crate::avm1_stub;
use crate::display_object::{Avm1Button, TDisplayObject, TInteractiveObject};
use crate::string::AvmString;

macro_rules! button_getter {
    ($name:ident) => {
        |activation, this, _args| {
            if let Some(display_object) = this.as_display_object() {
                if let Some(button) = display_object.as_avm1_button() {
                    return $name(button, activation);
                }
            }
            Ok(Value::Undefined)
        }
    };
}

macro_rules! button_setter {
    ($name:ident) => {
        |activation, this, args| {
            if let Some(display_object) = this.as_display_object() {
                if let Some(button) = display_object.as_avm1_button() {
                    let value = args.get(0).unwrap_or(&Value::Undefined).clone();
                    $name(button, activation, value)?;
                }
            }
            Ok(Value::Undefined)
        }
    };
}

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "enabled" => bool(true);
    "useHandCursor" => bool(true);
    "getDepth" => method(globals::get_depth; DONT_DELETE | READ_ONLY | VERSION_6);
    "blendMode" => property(button_getter!(blend_mode), button_setter!(set_blend_mode); DONT_DELETE | VERSION_8);
    "scale9Grid" => property(button_getter!(scale_9_grid), button_setter!(set_scale_9_grid); DONT_DELETE | DONT_ENUM | VERSION_8);
    "filters" => property(button_getter!(filters), button_setter!(set_filters); DONT_DELETE | DONT_ENUM | VERSION_8);
    "cacheAsBitmap" => property(button_getter!(cache_as_bitmap), button_setter!(set_cache_as_bitmap); DONT_DELETE | DONT_ENUM | VERSION_8);
    // NOTE: `tabEnabled` is not a built-in property of Button.
    "tabIndex" => property(button_getter!(tab_index), button_setter!(set_tab_index); VERSION_6);
};

pub fn create_class<'gc>(
    context: &mut DeclContext<'_, 'gc>,
    super_proto: Object<'gc>,
) -> SystemClass<'gc> {
    let class = context.empty_class(super_proto);
    context.define_properties_on(class.proto, PROTO_DECLS);
    class
}

fn blend_mode<'gc>(
    this: Avm1Button<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let mode = AvmString::new_utf8(activation.gc(), this.blend_mode().to_string());
    Ok(mode.into())
}

fn set_blend_mode<'gc>(
    this: Avm1Button<'gc>,
    _activation: &mut Activation<'_, 'gc>,
    value: Value<'gc>,
) -> Result<(), Error<'gc>> {
    // No-op if value is not a valid blend mode.
    if let Some(mode) = value.as_blend_mode() {
        this.set_blend_mode(mode.into());
    } else {
        tracing::error!("Unknown blend mode {value:?}");
    }
    Ok(())
}

fn filters<'gc>(
    this: Avm1Button<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(ArrayBuilder::new(activation)
        .with(
            this.filters()
                .iter()
                .map(|filter| bitmap_filter::filter_to_avm1(activation, filter.clone())),
        )
        .into())
}

fn set_filters<'gc>(
    this: Avm1Button<'gc>,
    activation: &mut Activation<'_, 'gc>,
    value: Value<'gc>,
) -> Result<(), Error<'gc>> {
    let mut filters = vec![];
    if let Value::Object(value) = value {
        for index in value.get_keys(activation, false).into_iter().rev() {
            let filter_object = value.get(index, activation)?.coerce_to_object(activation);
            if let Some(filter) = bitmap_filter::avm1_to_filter(filter_object, activation.context) {
                filters.push(filter);
            }
        }
    }
    this.set_filters(filters.into_boxed_slice());
    Ok(())
}

fn cache_as_bitmap<'gc>(
    this: Avm1Button<'gc>,
    _activation: &mut Activation<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    // Note that the *getter* returns actual, and *setter* is preference
    Ok(this.is_bitmap_cached().into())
}

fn set_cache_as_bitmap<'gc>(
    this: Avm1Button<'gc>,
    activation: &mut Activation<'_, 'gc>,
    value: Value<'gc>,
) -> Result<(), Error<'gc>> {
    // Note that the *getter* returns actual, and *setter* is preference
    this.set_bitmap_cached_preference(value.as_bool(activation.swf_version()));
    Ok(())
}

fn scale_9_grid<'gc>(
    this: Avm1Button<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "Button", "scale9Grid");
    let rect = this.scaling_grid();
    if rect.is_valid() {
        new_rectangle(activation, rect)
    } else {
        Ok(Value::Undefined)
    }
}

fn set_scale_9_grid<'gc>(
    this: Avm1Button<'gc>,
    activation: &mut Activation<'_, 'gc>,
    value: Value<'gc>,
) -> Result<(), Error<'gc>> {
    avm1_stub!(activation, "Button", "scale9Grid");
    if let Value::Object(object) = value {
        if let Some(rectangle) = object_to_rectangle(activation, object)? {
            this.set_scaling_grid(rectangle);
        }
    } else {
        this.set_scaling_grid(Default::default());
    };
    Ok(())
}

fn tab_index<'gc>(
    this: Avm1Button<'gc>,
    _activation: &mut Activation<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(index) = this.tab_index() {
        Ok(Value::Number(index as f64))
    } else {
        Ok(Value::Undefined)
    }
}

fn set_tab_index<'gc>(
    this: Avm1Button<'gc>,
    activation: &mut Activation<'_, 'gc>,
    value: Value<'gc>,
) -> Result<(), Error<'gc>> {
    let value = match value {
        Value::Undefined | Value::Null => None,
        Value::Bool(_) | Value::Number(_) => {
            // FIXME This coercion is not perfect, as it wraps
            //       instead of falling back to MIN, as FP does
            let i32_value = value.coerce_to_i32(activation)?;
            Some(i32_value)
        }
        _ => Some(i32::MIN),
    };
    this.set_tab_index(value);
    Ok(())
}
