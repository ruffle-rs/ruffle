//! `flash.display.Stage` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::Method;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{Object, TObject};
use crate::avm2::traits::Trait;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::display_object::TDisplayObject;
use gc_arena::{GcCell, MutationContext};
use swf::Color;

/// Implements `flash.display.Stage`'s instance constructor.
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

/// Implements `flash.display.Stage`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Overrides `accessibilityProperties`'s setter.
pub fn set_accessibility_properties<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Error: You cannot set accessibility properties on the stage.".into())
}

/// Overrides `alpha`'s setter.
pub fn set_alpha<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Error: You cannot set the stage's opacity.".into())
}

/// Overrides `blendMode`'s setter.
pub fn set_blend_mode<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Error: You cannot set the blend mode of the stage.".into())
}

/// Overrides `cacheAsBitmap`'s setter.
pub fn set_cache_as_bitmap<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Error: You cannot set the stage to be cached as a bitmap.".into())
}

/// Overrides `contextMenu`'s setter.
pub fn set_context_menu<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Error: You cannot set the stage's context menu.".into())
}

/// Overrides `filters`'s setter.
pub fn set_filters<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Error: You cannot apply filters to the stage.".into())
}

/// Overrides `focusRect`'s setter.
pub fn set_focus_rect<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Error: You cannot set the stage's focus rect.".into())
}

/// Overrides `loaderInfo`'s setter.
pub fn set_loader_info<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Error: You cannot set the blend mode of the stage.".into())
}

/// Overrides `mask`'s setter.
pub fn set_mask<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Error: You cannot mask the stage.".into())
}

/// Overrides `mouseEnabled`'s setter.
pub fn set_mouse_enabled<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Error: You cannot enable or disable the mouse on the stage.".into())
}

/// Overrides `name`'s getter.
pub fn name<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Null)
}

/// Overrides `name`'s setter.
pub fn set_name<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Error: You cannot set the name of the stage.".into())
}

/// Overrides `opaqueBackground`'s setter.
pub fn set_opaque_background<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Error: You cannot give or take away the stage's opaque background.".into())
}

/// Overrides `rotation`'s setter.
pub fn set_rotation<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Error: You cannot rotate the stage.".into())
}

/// Overrides `scale9Grid`'s setter.
pub fn set_scale_nine_grid<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Error: You cannot set the stage's 9-slice grid.".into())
}

/// Overrides `scaleX`'s setter.
pub fn set_scale_x<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Error: You cannot set the stage's horizontal scale.".into())
}

/// Overrides `scaleY`'s setter.
pub fn set_scale_y<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Error: You cannot set the stage's vertical scale.".into())
}

/// Overrides `scrollRect`'s setter.
pub fn set_scroll_rect<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Error: You cannot set the stage's scroll rectangle.".into())
}

/// Overrides `tabEnabled`'s setter.
pub fn set_tab_enabled<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Error: You cannot enable or disable tabbing the stage.".into())
}

/// Overrides `tabIndex`'s setter.
pub fn set_tab_index<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Error: You cannot set the stage's tab index.".into())
}

/// Overrides `transform`'s setter.
pub fn set_transform<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Error: You cannot transform the stage.".into())
}

/// Overrides `visible`'s setter.
pub fn set_visible<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Error: You cannot hide or unhide the stage.".into())
}

/// Overrides `x`'s setter.
pub fn set_x<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Error: You cannot move the stage horizontally.".into())
}

/// Overrides `y`'s setter.
pub fn set_y<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Error: You cannot move the stage vertically.".into())
}

/// Implement `browserZoomFactor`'s getter
pub fn browser_zoom_factor<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(dobj) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_stage())
    {
        return Ok(dobj.viewport_scale_factor().into());
    }

    Ok(Value::Undefined)
}

/// Implement `color`'s getter
pub fn color<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(dobj) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_stage())
    {
        return Ok(dobj
            .background_color()
            .unwrap_or_else(|| Color::from_rgb(0xffffff, 255))
            .to_rgb()
            .into());
    }

    Ok(Value::Undefined)
}

/// Construct `Stage`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.display"), "Stage"),
        Some(
            QName::new(
                Namespace::package("flash.display"),
                "DisplayObjectContainer",
            )
            .into(),
        ),
        Method::from_builtin(instance_init),
        Method::from_builtin(class_init),
        mc,
    );

    let mut write = class.write(mc);

    write.set_attributes(ClassAttributes::SEALED);

    write.define_instance_trait(
        Trait::from_setter(
            QName::new(Namespace::public(), "accessibilityProperties"),
            Method::from_builtin(set_accessibility_properties),
        )
        .with_override(),
    );
    write.define_instance_trait(
        Trait::from_setter(
            QName::new(Namespace::public(), "alpha"),
            Method::from_builtin(set_alpha),
        )
        .with_override(),
    );
    write.define_instance_trait(
        Trait::from_setter(
            QName::new(Namespace::public(), "blendMode"),
            Method::from_builtin(set_blend_mode),
        )
        .with_override(),
    );
    write.define_instance_trait(
        Trait::from_setter(
            QName::new(Namespace::public(), "cacheAsBitmap"),
            Method::from_builtin(set_cache_as_bitmap),
        )
        .with_override(),
    );
    write.define_instance_trait(
        Trait::from_setter(
            QName::new(Namespace::public(), "contextMenu"),
            Method::from_builtin(set_context_menu),
        )
        .with_override(),
    );
    write.define_instance_trait(
        Trait::from_setter(
            QName::new(Namespace::public(), "filters"),
            Method::from_builtin(set_filters),
        )
        .with_override(),
    );
    write.define_instance_trait(
        Trait::from_setter(
            QName::new(Namespace::public(), "focusRect"),
            Method::from_builtin(set_focus_rect),
        )
        .with_override(),
    );
    write.define_instance_trait(
        Trait::from_setter(
            QName::new(Namespace::public(), "loaderInfo"),
            Method::from_builtin(set_loader_info),
        )
        .with_override(),
    );
    write.define_instance_trait(
        Trait::from_setter(
            QName::new(Namespace::public(), "mask"),
            Method::from_builtin(set_mask),
        )
        .with_override(),
    );
    write.define_instance_trait(
        Trait::from_setter(
            QName::new(Namespace::public(), "mouseEnabled"),
            Method::from_builtin(set_mouse_enabled),
        )
        .with_override(),
    );

    write.define_instance_trait(
        Trait::from_getter(
            QName::new(Namespace::public(), "name"),
            Method::from_builtin(name),
        )
        .with_override(),
    );
    write.define_instance_trait(
        Trait::from_setter(
            QName::new(Namespace::public(), "name"),
            Method::from_builtin(set_name),
        )
        .with_override(),
    );

    write.define_instance_trait(
        Trait::from_setter(
            QName::new(Namespace::public(), "opaqueBackground"),
            Method::from_builtin(set_opaque_background),
        )
        .with_override(),
    );
    write.define_instance_trait(
        Trait::from_setter(
            QName::new(Namespace::public(), "rotation"),
            Method::from_builtin(set_rotation),
        )
        .with_override(),
    );
    write.define_instance_trait(
        Trait::from_setter(
            QName::new(Namespace::public(), "scale9Grid"),
            Method::from_builtin(set_scale_nine_grid),
        )
        .with_override(),
    );
    write.define_instance_trait(
        Trait::from_setter(
            QName::new(Namespace::public(), "scaleX"),
            Method::from_builtin(set_scale_x),
        )
        .with_override(),
    );
    write.define_instance_trait(
        Trait::from_setter(
            QName::new(Namespace::public(), "scaleY"),
            Method::from_builtin(set_scale_y),
        )
        .with_override(),
    );
    write.define_instance_trait(
        Trait::from_setter(
            QName::new(Namespace::public(), "scrollRect"),
            Method::from_builtin(set_scroll_rect),
        )
        .with_override(),
    );
    write.define_instance_trait(
        Trait::from_setter(
            QName::new(Namespace::public(), "tabEnabled"),
            Method::from_builtin(set_tab_enabled),
        )
        .with_override(),
    );
    write.define_instance_trait(
        Trait::from_setter(
            QName::new(Namespace::public(), "tabIndex"),
            Method::from_builtin(set_tab_index),
        )
        .with_override(),
    );
    write.define_instance_trait(
        Trait::from_setter(
            QName::new(Namespace::public(), "transform"),
            Method::from_builtin(set_transform),
        )
        .with_override(),
    );
    write.define_instance_trait(
        Trait::from_setter(
            QName::new(Namespace::public(), "visible"),
            Method::from_builtin(set_visible),
        )
        .with_override(),
    );
    write.define_instance_trait(
        Trait::from_setter(
            QName::new(Namespace::public(), "x"),
            Method::from_builtin(set_x),
        )
        .with_override(),
    );
    write.define_instance_trait(
        Trait::from_setter(
            QName::new(Namespace::public(), "y"),
            Method::from_builtin(set_y),
        )
        .with_override(),
    );
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public(), "browserZoomFactor"),
        Method::from_builtin(browser_zoom_factor),
    ));
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public(), "color"),
        Method::from_builtin(color),
    ));

    class
}
