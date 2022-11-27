//! `flash.display.DisplayObject` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::object::{stage_allocator, Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::Multiname;
use crate::avm2::Namespace;
use crate::avm2::QName;
use crate::avm2::{ArrayObject, ArrayStorage};
use crate::display_object::{DisplayObject, HitTestOptions, TDisplayObject};
use crate::ecma_conversions::round_to_even;
use crate::frame_lifecycle::catchup_display_object_to_frame;
use crate::prelude::*;
use crate::string::AvmString;
use crate::types::{Degrees, Percent};
use crate::vminterface::Instantiator;
use gc_arena::{GcCell, MutationContext};
use std::str::FromStr;
use swf::Twips;
use swf::{BlendMode, Rectangle};

/// Implements `flash.display.DisplayObject`'s instance constructor.
pub fn instance_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Err("You cannot construct DisplayObject directly.".into())
}

/// Implements `flash.display.DisplayObject`'s native instance constructor.
pub fn native_instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        activation.super_init(this, &[])?;

        if this.as_display_object().is_none() {
            let class_object = this
                .instance_of()
                .ok_or("Attempted to construct DisplayObject on a bare object.")?;

            if let Some((movie, symbol)) = activation
                .context
                .library
                .avm2_class_registry()
                .class_symbol(class_object)
            {
                let mut child = activation
                    .context
                    .library
                    .library_for_movie_mut(movie)
                    .instantiate_by_id(symbol, activation.context.gc_context)?;

                this.init_display_object(activation.context.gc_context, child);
                child.set_object2(activation.context.gc_context, this);

                child.post_instantiation(&mut activation.context, None, Instantiator::Avm2, false);
                catchup_display_object_to_frame(&mut activation.context, child);
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `flash.display.DisplayObject`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Value::Undefined)
}

/// Implements `alpha`'s getter.
pub fn alpha<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        return Ok(dobj.alpha().into());
    }

    Ok(Value::Undefined)
}

/// Implements `alpha`'s setter.
pub fn set_alpha<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        let new_alpha = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_number(activation)?;
        dobj.set_alpha(activation.context.gc_context, new_alpha);
    }

    Ok(Value::Undefined)
}

/// Implements `height`'s getter.
pub fn height<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        return Ok(dobj.height().into());
    }

    Ok(Value::Undefined)
}

/// Implements `height`'s setter.
pub fn set_height<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        let new_height = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_number(activation)?;

        if new_height >= 0.0 {
            dobj.set_height(activation.context.gc_context, new_height);
        }
    }

    Ok(Value::Undefined)
}

/// Implements `scaleY`'s getter.
pub fn scale_y<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        return Ok(dobj.scale_y(activation.context.gc_context).unit().into());
    }

    Ok(Value::Undefined)
}

/// Implements `scaleY`'s setter.
pub fn set_scale_y<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        let new_scale = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_number(activation)?;
        dobj.set_scale_y(activation.context.gc_context, Percent::from_unit(new_scale));
    }

    Ok(Value::Undefined)
}

/// Implements `width`'s getter.
pub fn width<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        return Ok(dobj.width().into());
    }

    Ok(Value::Undefined)
}

/// Implements `width`'s setter.
pub fn set_width<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        let new_width = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_number(activation)?;

        if new_width >= 0.0 {
            dobj.set_width(activation.context.gc_context, new_width);
        }
    }

    Ok(Value::Undefined)
}

/// Implements `scaleX`'s getter.
pub fn scale_x<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        return Ok(dobj.scale_x(activation.context.gc_context).unit().into());
    }

    Ok(Value::Undefined)
}

/// Implements `scaleX`'s setter.
pub fn set_scale_x<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        let new_scale = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_number(activation)?;
        dobj.set_scale_x(activation.context.gc_context, Percent::from_unit(new_scale));
    }

    Ok(Value::Undefined)
}

pub fn filters<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        return Ok(ArrayObject::from_storage(activation, dobj.filters())?.into());
    }
    Ok(ArrayObject::empty(activation)?.into())
}

fn build_argument_type_error<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Value<'gc>, Error<'gc>> {
    Err(Error::AvmError(crate::avm2::error::argument_error(
        activation,
        "Error #2005: Parameter 0 is of the incorrect type. Should be type Filter.",
        2005,
    )?))
}

pub fn set_filters<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        let new_filters = args.get(0).cloned().unwrap_or(Value::Undefined);

        if matches!(new_filters, Value::Undefined | Value::Null) {
            let new_storage = ArrayStorage::new(0);
            dobj.set_filters(activation.context.gc_context, new_storage);
        } else {
            let new_filters = new_filters.coerce_to_object(activation)?;

            if let Some(filters_array) = new_filters.as_array_object() {
                if let Some(filters_storage) = filters_array.as_array_storage() {
                    let filter_class =
                        Multiname::new(Namespace::Package("flash.filters".into()), "BitmapFilter");

                    let filter_class_object = activation.resolve_class(&filter_class)?;

                    for filter in filters_storage.iter().flatten() {
                        if matches!(filter, Value::Undefined | Value::Null) {
                            return build_argument_type_error(activation);
                        } else {
                            let filter_object = filter.coerce_to_object(activation)?;

                            if !filter_object.is_of_type(filter_class_object, activation) {
                                return build_argument_type_error(activation);
                            }
                        }
                    }
                    let new_storage = ArrayStorage::from_storage(filters_storage.iter().collect());

                    dobj.set_filters(activation.context.gc_context, new_storage);
                }
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `x`'s getter.
pub fn x<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        return Ok(dobj.x().into());
    }

    Ok(Value::Undefined)
}

/// Implements `x`'s setter.
pub fn set_x<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        let new_x = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_number(activation)?;

        dobj.set_x(activation.context.gc_context, new_x);
    }

    Ok(Value::Undefined)
}

/// Implements `y`'s getter.
pub fn y<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        return Ok(dobj.y().into());
    }

    Ok(Value::Undefined)
}

/// Implements `y`'s setter.
pub fn set_y<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        let new_y = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_number(activation)?;

        dobj.set_y(activation.context.gc_context, new_y);
    }

    Ok(Value::Undefined)
}

/// Stubs `z`'s getter.
pub fn z<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(0.into())
}

/// Stubs `z`'s setter.
pub fn set_z<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Value::Undefined)
}

/// Stubs `rotationX`'s getter.
pub fn rotation_x<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(0.into())
}

/// Stubs `rotationX`'s setter.
pub fn set_rotation_x<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Value::Undefined)
}

/// Stubs `rotationY`'s getter.
pub fn rotation_y<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(0.into())
}

/// Stubs `rotationY`'s setter.
pub fn set_rotation_y<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Value::Undefined)
}

/// Stubs `rotationZ`'s getter.
pub fn rotation_z<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(0.into())
}

/// Stubs `rotationZ`'s setter.
pub fn set_rotation_z<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Value::Undefined)
}

/// Stubs `scaleZ`'s getter.
pub fn scale_z<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(1.into())
}

/// Stubs `scaleZ`'s setter.
pub fn set_scale_z<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Value::Undefined)
}

/// Implements `rotation`'s getter.
pub fn rotation<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        let rot: f64 = dobj.rotation(activation.context.gc_context).into();
        let rem = rot % 360.0;

        if rem <= 180.0 {
            return Ok(rem.into());
        } else {
            return Ok((rem - 360.0).into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `rotation`'s setter.
pub fn set_rotation<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        let new_rotation = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_number(activation)?;

        dobj.set_rotation(activation.context.gc_context, Degrees::from(new_rotation));
    }

    Ok(Value::Undefined)
}

/// Implements `name`'s getter.
pub fn name<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        return Ok(dobj.name().into());
    }

    Ok(Value::Undefined)
}

/// Implements `name`'s setter.
pub fn set_name<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        let new_name = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_string(activation)?;

        if dobj.instantiated_by_timeline() {
            return Err(format!(
                "Display object {} was placed by the timeline and cannot have it's name changed.",
                new_name
            )
            .into());
        }

        dobj.set_name(activation.context.gc_context, new_name);
    }

    Ok(Value::Undefined)
}

/// Implements `parent`.
pub fn parent<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        return Ok(dobj
            .avm2_parent()
            .map(|parent| parent.object2())
            .unwrap_or(Value::Null));
    }

    Ok(Value::Undefined)
}

/// Implements `root`.
pub fn root<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        return Ok(dobj
            .avm2_root(&mut activation.context)
            .map(|root| root.object2())
            .unwrap_or(Value::Null));
    }

    Ok(Value::Undefined)
}

/// Implements `stage`.
pub fn stage<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        return Ok(dobj
            .avm2_stage(&activation.context)
            .map(|stage| stage.object2())
            .unwrap_or(Value::Null));
    }

    Ok(Value::Undefined)
}

/// Implements `visible`'s getter.
pub fn visible<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        return Ok(dobj.visible().into());
    }

    Ok(Value::Undefined)
}

/// Implements `visible`'s setter.
pub fn set_visible<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        let new_visible = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_boolean();

        dobj.set_visible(activation.context.gc_context, new_visible);
    }

    Ok(Value::Undefined)
}

/// Implements `mouseX`.
pub fn mouse_x<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        let local_mouse = dobj.global_to_local(*activation.context.mouse_position);

        return Ok(local_mouse.0.to_pixels().into());
    }

    Ok(Value::Undefined)
}

/// Implements `mouseY`.
pub fn mouse_y<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        let local_mouse = dobj.global_to_local(*activation.context.mouse_position);

        return Ok(local_mouse.1.to_pixels().into());
    }

    Ok(Value::Undefined)
}

/// Implements `hitTestPoint`.
pub fn hit_test_point<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        let x = Twips::from_pixels(
            args.get(0)
                .cloned()
                .unwrap_or(Value::Undefined)
                .coerce_to_number(activation)?,
        );
        let y = Twips::from_pixels(
            args.get(1)
                .cloned()
                .unwrap_or(Value::Undefined)
                .coerce_to_number(activation)?,
        );
        let shape_flag = args
            .get(2)
            .cloned()
            .unwrap_or_else(|| false.into())
            .coerce_to_boolean();

        if shape_flag {
            return Ok(dobj
                .hit_test_shape(
                    &mut activation.context,
                    (x, y),
                    HitTestOptions::AVM_HIT_TEST,
                )
                .into());
        } else {
            return Ok(dobj.hit_test_bounds((x, y)).into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `hitTestObject`.
pub fn hit_test_object<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        if let Some(rhs_dobj) = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .as_object()
            .and_then(|o| o.as_display_object())
        {
            return Ok(dobj.hit_test_object(rhs_dobj).into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `loaderInfo` getter
pub fn loader_info<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        // Contrary to the DisplayObject.loaderInfo documentation,
        // Flash Player defines 'loaderInfo' for non-root DisplayObjects.
        // It always returns the LoaderInfo from the root object.
        if let Some(loader_info) = dobj
            .avm2_root(&mut activation.context)
            .and_then(|root_dobj| root_dobj.loader_info())
        {
            return Ok(loader_info.into());
        }
        return Ok(Value::Null);
    }
    Ok(Value::Undefined)
}

pub fn transform<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        return Ok(activation
            .avm2()
            .classes()
            .transform
            .construct(activation, &[this.into()])?
            .into());
    }
    Ok(Value::Undefined)
}

pub fn set_transform<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        let transform = args[0].coerce_to_object(activation)?;

        // FIXME - consider 3D matrix and pixel bounds
        let matrix = transform
            .get_property(&Multiname::public("matrix"), activation)?
            .coerce_to_object(activation)?;
        let color_transform = transform
            .get_property(&Multiname::public("matrix"), activation)?
            .coerce_to_object(activation)?;

        let matrix =
            crate::avm2::globals::flash::geom::transform::object_to_matrix(matrix, activation)?;
        let color_transform =
            crate::avm2::globals::flash::geom::transform::object_to_color_transform(
                color_transform,
                activation,
            )?;

        let dobj = this.as_display_object().unwrap();
        let mut write = dobj.base_mut(activation.context.gc_context);
        write.set_matrix(matrix);
        write.set_color_transform(color_transform);
    }
    Ok(Value::Undefined)
}

/// Implements `DisplayObject.blendMode`'s getter.
pub fn blend_mode<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        let mode =
            AvmString::new_utf8(activation.context.gc_context, dobj.blend_mode().to_string());
        return Ok(mode.into());
    }
    Ok(Value::Undefined)
}

/// Implements `DisplayObject.blendMode`'s setter.
pub fn set_blend_mode<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        let mode = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_string(activation)?;

        if let Ok(mode) = BlendMode::from_str(&mode.to_string()) {
            dobj.set_blend_mode(activation.context.gc_context, mode);
        } else {
            log::error!("Unknown blend mode {}", mode);
            return Err("ArgumentError: Error #2008: Parameter blendMode must be one of the accepted values.".into());
        }
    }
    Ok(Value::Undefined)
}

fn new_rectangle<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    rectangle: BoundingBox,
) -> Result<Object<'gc>, Error<'gc>> {
    let x = rectangle.x_min.to_pixels();
    let y = rectangle.y_min.to_pixels();
    let width = rectangle.width().to_pixels();
    let height = rectangle.height().to_pixels();
    let args = &[x.into(), y.into(), width.into(), height.into()];
    activation
        .avm2()
        .classes()
        .rectangle
        .construct(activation, args)
}

fn scroll_rect<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        if dobj.has_scroll_rect() {
            return Ok(new_rectangle(activation, dobj.next_scroll_rect().into())?.into());
        } else {
            return Ok(Value::Null);
        }
    }
    Ok(Value::Undefined)
}

fn object_to_rectangle<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    object: Object<'gc>,
) -> Result<Rectangle<Twips>, Error<'gc>> {
    const NAMES: &[&str] = &["x", "y", "width", "height"];
    let mut values = [0.0; 4];
    for (&name, value) in NAMES.iter().zip(&mut values) {
        *value = object
            .get_property(&Multiname::public(name), activation)?
            .coerce_to_number(activation)?;
    }
    let [x, y, width, height] = values;
    Ok(Rectangle {
        x_min: Twips::from_pixels_i32(round_to_even(x)),
        y_min: Twips::from_pixels_i32(round_to_even(y)),
        x_max: Twips::from_pixels_i32(round_to_even(x + width)),
        y_max: Twips::from_pixels_i32(round_to_even(y + height)),
    })
}

fn set_scroll_rect<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        if let Some(rectangle) = args[0].as_object() {
            // Flash only updates the "internal" scrollRect used by `localToLocal` when the next
            // frame is rendered. However, accessing `DisplayObject.scrollRect` from ActionScript
            // will immediately return the updated value.
            //
            // To implement this, our `DisplayObject.scrollRect` ActionScript getter/setter both
            // operate on a `next_scroll_rect` field. Just before we render a DisplayObject, we copy
            // its `next_scroll_rect` to the `scroll_rect` field used for both rendering and
            // `localToGlobal`.
            dobj.set_next_scroll_rect(
                activation.context.gc_context,
                object_to_rectangle(activation, rectangle)?,
            );

            // TODO: Technically we should accept only `flash.geom.Rectangle` objects, in which case
            // `object_to_rectangle` will be infallible. Once this happens, the following line can
            // be moved above the `set_next_scroll_rect` call.
            dobj.set_has_scroll_rect(activation.context.gc_context, true);
        } else {
            dobj.set_has_scroll_rect(activation.context.gc_context, false);
        }
    }
    Ok(Value::Undefined)
}

fn local_to_global<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        let point = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_object(activation)?;
        let x = point
            .get_property(&Multiname::public("x"), activation)?
            .coerce_to_number(activation)?;
        let y = point
            .get_property(&Multiname::public("y"), activation)?
            .coerce_to_number(activation)?;

        let (out_x, out_y) = dobj.local_to_global((Twips::from_pixels(x), Twips::from_pixels(y)));
        return Ok(activation
            .avm2()
            .classes()
            .point
            .construct(
                activation,
                &[out_x.to_pixels().into(), out_y.to_pixels().into()],
            )?
            .into());
    }

    Ok(Value::Undefined)
}

fn global_to_local<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        let point = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_object(activation)?;
        let x = point
            .get_property(&Multiname::public("x"), activation)?
            .coerce_to_number(activation)?;
        let y = point
            .get_property(&Multiname::public("y"), activation)?
            .coerce_to_number(activation)?;

        let (out_x, out_y) = dobj.global_to_local((Twips::from_pixels(x), Twips::from_pixels(y)));
        return Ok(activation
            .avm2()
            .classes()
            .point
            .construct(
                activation,
                &[out_x.to_pixels().into(), out_y.to_pixels().into()],
            )?
            .into());
    }

    Ok(Value::Undefined)
}

fn get_bounds<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        // TODO: add typing `(target: DisplayObject)` for proper type errors
        if let Some(target) = args.get(0).cloned().and_then(|value| match value {
            Value::Undefined | Value::Null => Some(dobj),
            _ => value.as_object().and_then(|o| o.as_display_object()),
        }) {
            let bounds = dobj.bounds();
            let out_bounds = if DisplayObject::ptr_eq(dobj, target) {
                // Getting the clips bounds in its own coordinate space; no AABB transform needed.
                bounds
            } else {
                // Transform AABB to target space.
                // Calculate the matrix to transform into the target coordinate space, and transform the above AABB.
                // Note that this doesn't produce as tight of an AABB as if we had used `bounds_with_transform` with
                // the final matrix, but this matches Flash's behavior.
                let to_global_matrix = dobj.local_to_global_matrix();
                let to_target_matrix = target.global_to_local_matrix();
                let bounds_transform = to_target_matrix * to_global_matrix;
                bounds.transform(&bounds_transform)
            };

            return Ok(new_rectangle(activation, out_bounds)?.into());
        } else {
            return Ok(Value::Undefined);
        }
    }
    Ok(Value::Undefined)
}

fn get_rect<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // TODO: This should get the bounds ignoring strokes. Always equal to or smaller than getBounds.
    // Just defer to getBounds for now. Will have to store edge_bounds vs. shape_bounds in Graphic.
    get_bounds(activation, this, args)
}

fn mask<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.and_then(|this| this.as_display_object()) {
        return Ok(this.masker().map_or(Value::Null, |m| m.object2()));
    }
    Ok(Value::Undefined)
}

fn set_mask<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.and_then(|this| this.as_display_object()) {
        let mask = args.get(0).unwrap_or(&Value::Null);

        if matches!(mask, Value::Null) {
            this.set_masker(activation.context.gc_context, None, true);
        } else {
            let mask = mask
                .coerce_to_object(activation)?
                .as_display_object()
                .ok_or_else(|| -> Error {
                    format!("Mask is not a DisplayObject: {mask:?}").into()
                })?;

            this.set_masker(activation.context.gc_context, Some(mask), true);
            mask.set_maskee(activation.context.gc_context, Some(this), true);
        }
    }
    Ok(Value::Undefined)
}

fn cache_as_bitmap<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.and_then(|this| this.as_display_object()) {
        return Ok(this.is_bitmap_cached().into());
    }
    Ok(Value::Undefined)
}

fn set_cache_as_bitmap<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.and_then(|this| this.as_display_object()) {
        let cache = args.get(0).unwrap_or(&Value::Undefined).coerce_to_boolean();
        this.set_is_bitmap_cached(activation.context.gc_context, cache);
    }
    Ok(Value::Undefined)
}

/// Construct `DisplayObject`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.display"), "DisplayObject"),
        Some(Multiname::new(
            Namespace::package("flash.events"),
            "EventDispatcher",
        )),
        Method::from_builtin(instance_init, "<DisplayObject instance initializer>", mc),
        Method::from_builtin(class_init, "<DisplayObject class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    write.set_instance_allocator(stage_allocator);
    write.set_native_instance_init(Method::from_builtin(
        native_instance_init,
        "<DisplayObject native instance initializer>",
        mc,
    ));

    write.implements(Multiname::new(
        Namespace::package("flash.display"),
        "IBitmapDrawable",
    ));

    const PUBLIC_INSTANCE_PROPERTIES: &[(
        &str,
        Option<NativeMethodImpl>,
        Option<NativeMethodImpl>,
    )] = &[
        ("alpha", Some(alpha), Some(set_alpha)),
        ("blendMode", Some(blend_mode), Some(set_blend_mode)),
        ("height", Some(height), Some(set_height)),
        ("scaleY", Some(scale_y), Some(set_scale_y)),
        ("width", Some(width), Some(set_width)),
        ("scaleX", Some(scale_x), Some(set_scale_x)),
        ("x", Some(x), Some(set_x)),
        ("y", Some(y), Some(set_y)),
        ("z", Some(z), Some(set_z)), //TODO: This property needs to be version-gated to SWF10+
        ("rotation", Some(rotation), Some(set_rotation)),
        ("rotationX", Some(rotation_x), Some(set_rotation_x)),
        ("rotationY", Some(rotation_y), Some(set_rotation_y)),
        ("rotationZ", Some(rotation_z), Some(set_rotation_z)),
        ("scaleZ", Some(scale_z), Some(set_scale_z)),
        ("name", Some(name), Some(set_name)),
        ("parent", Some(parent), None),
        ("root", Some(root), None),
        ("stage", Some(stage), None),
        ("visible", Some(visible), Some(set_visible)),
        ("mouseX", Some(mouse_x), None),
        ("mouseY", Some(mouse_y), None),
        ("loaderInfo", Some(loader_info), None),
        ("filters", Some(filters), Some(set_filters)),
        ("transform", Some(transform), Some(set_transform)),
        ("scrollRect", Some(scroll_rect), Some(set_scroll_rect)),
        ("mask", Some(mask), Some(set_mask)),
        (
            "cacheAsBitmap",
            Some(cache_as_bitmap),
            Some(set_cache_as_bitmap),
        ),
    ];
    write.define_public_builtin_instance_properties(mc, PUBLIC_INSTANCE_PROPERTIES);

    const PUBLIC_INSTANCE_METHODS: &[(&str, NativeMethodImpl)] = &[
        ("hitTestPoint", hit_test_point),
        ("hitTestObject", hit_test_object),
        ("localToGlobal", local_to_global),
        ("globalToLocal", global_to_local),
        ("getBounds", get_bounds),
        ("getRect", get_rect),
    ];
    write.define_public_builtin_instance_methods(mc, PUBLIC_INSTANCE_METHODS);

    class
}
