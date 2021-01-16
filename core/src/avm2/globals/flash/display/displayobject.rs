//! `flash.display.DisplayObject` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::method::Method;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{Object, TObject};
use crate::avm2::string::AvmString;
use crate::avm2::traits::Trait;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::display_object::TDisplayObject;
use crate::types::{Degrees, Percent};
use gc_arena::{GcCell, MutationContext};
use swf::Twips;

/// Implements `flash.display.DisplayObject`'s instance constructor.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        activation.super_init(this, &[])?;
    }

    Ok(Value::Undefined)
}

/// Implements `flash.display.DisplayObject`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements `alpha`'s getter.
pub fn alpha<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
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
) -> Result<Value<'gc>, Error> {
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
) -> Result<Value<'gc>, Error> {
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
) -> Result<Value<'gc>, Error> {
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
) -> Result<Value<'gc>, Error> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        return Ok(Value::Number(
            dobj.scale_y(activation.context.gc_context).into_unit(),
        ));
    }

    Ok(Value::Undefined)
}

/// Implements `scaleY`'s setter.
pub fn set_scale_y<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
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
) -> Result<Value<'gc>, Error> {
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
) -> Result<Value<'gc>, Error> {
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
) -> Result<Value<'gc>, Error> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        return Ok(Value::Number(
            dobj.scale_x(activation.context.gc_context).into_unit(),
        ));
    }

    Ok(Value::Undefined)
}

/// Implements `scaleX`'s setter.
pub fn set_scale_x<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
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

/// Implements `x`'s getter.
pub fn x<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
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
) -> Result<Value<'gc>, Error> {
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
) -> Result<Value<'gc>, Error> {
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
) -> Result<Value<'gc>, Error> {
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

/// Implements `rotation`'s getter.
pub fn rotation<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        let rot: f64 = dobj.rotation(activation.context.gc_context).into();
        let rem = rot % 360.0;

        if rem <= 180.0 {
            return Ok(Value::Number(rem));
        } else {
            return Ok(Value::Number(rem - 360.0));
        }
    }

    Ok(Value::Undefined)
}

/// Implements `rotation`'s setter.
pub fn set_rotation<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
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
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        return Ok(AvmString::new(activation.context.gc_context, dobj.name().to_string()).into());
    }

    Ok(Value::Undefined)
}

/// Implements `name`'s setter.
pub fn set_name<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
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

        dobj.set_name(activation.context.gc_context, &new_name);
    }

    Ok(Value::Undefined)
}

/// Implements `parent`.
pub fn parent<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        return Ok(dobj
            .parent()
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
) -> Result<Value<'gc>, Error> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        return Ok(dobj
            .root(&activation.context)
            .map(|root| root.object2())
            .unwrap_or(Value::Null));
    }

    Ok(Value::Undefined)
}

/// Implements `visible`'s getter.
pub fn visible<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
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
) -> Result<Value<'gc>, Error> {
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
) -> Result<Value<'gc>, Error> {
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
) -> Result<Value<'gc>, Error> {
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
) -> Result<Value<'gc>, Error> {
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
            return Ok(dobj.hit_test_shape(&mut activation.context, (x, y)).into());
        } else {
            return Ok(dobj.hit_test_bounds((x, y)).into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `hitTestObject`.
pub fn hit_test_object<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        if let Some(rhs_dobj) = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_object(activation)?
            .as_display_object()
        {
            return Ok(dobj.hit_test_object(rhs_dobj).into());
        }
    }

    Ok(Value::Undefined)
}

/// Construct `DisplayObject`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.display"), "DisplayObject"),
        Some(QName::new(Namespace::package("flash.events"), "EventDispatcher").into()),
        Method::from_builtin(instance_init),
        Method::from_builtin(class_init),
        mc,
    );

    let mut write = class.write(mc);

    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public(), "alpha"),
        Method::from_builtin(alpha),
    ));
    write.define_instance_trait(Trait::from_setter(
        QName::new(Namespace::public(), "alpha"),
        Method::from_builtin(set_alpha),
    ));
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public(), "height"),
        Method::from_builtin(height),
    ));
    write.define_instance_trait(Trait::from_setter(
        QName::new(Namespace::public(), "height"),
        Method::from_builtin(set_height),
    ));
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public(), "scaleY"),
        Method::from_builtin(scale_y),
    ));
    write.define_instance_trait(Trait::from_setter(
        QName::new(Namespace::public(), "scaleY"),
        Method::from_builtin(set_scale_y),
    ));
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public(), "width"),
        Method::from_builtin(width),
    ));
    write.define_instance_trait(Trait::from_setter(
        QName::new(Namespace::public(), "width"),
        Method::from_builtin(set_width),
    ));
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public(), "scaleX"),
        Method::from_builtin(scale_x),
    ));
    write.define_instance_trait(Trait::from_setter(
        QName::new(Namespace::public(), "scaleX"),
        Method::from_builtin(set_scale_x),
    ));
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public(), "x"),
        Method::from_builtin(x),
    ));
    write.define_instance_trait(Trait::from_setter(
        QName::new(Namespace::public(), "x"),
        Method::from_builtin(set_x),
    ));
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public(), "y"),
        Method::from_builtin(y),
    ));
    write.define_instance_trait(Trait::from_setter(
        QName::new(Namespace::public(), "y"),
        Method::from_builtin(set_y),
    ));
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public(), "rotation"),
        Method::from_builtin(rotation),
    ));
    write.define_instance_trait(Trait::from_setter(
        QName::new(Namespace::public(), "rotation"),
        Method::from_builtin(set_rotation),
    ));
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public(), "name"),
        Method::from_builtin(name),
    ));
    write.define_instance_trait(Trait::from_setter(
        QName::new(Namespace::public(), "name"),
        Method::from_builtin(set_name),
    ));
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public(), "parent"),
        Method::from_builtin(parent),
    ));
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public(), "root"),
        Method::from_builtin(root),
    ));
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public(), "visible"),
        Method::from_builtin(visible),
    ));
    write.define_instance_trait(Trait::from_setter(
        QName::new(Namespace::public(), "visible"),
        Method::from_builtin(set_visible),
    ));
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public(), "mouseX"),
        Method::from_builtin(mouse_x),
    ));
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public(), "mouseY"),
        Method::from_builtin(mouse_y),
    ));
    write.define_instance_trait(Trait::from_method(
        QName::new(Namespace::public(), "hitTestPoint"),
        Method::from_builtin(hit_test_point),
    ));
    write.define_instance_trait(Trait::from_method(
        QName::new(Namespace::public(), "hitTestObject"),
        Method::from_builtin(hit_test_object),
    ));

    class
}
