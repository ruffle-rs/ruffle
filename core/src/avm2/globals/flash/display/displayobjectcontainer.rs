//! `flash.display.DisplayObjectContainer` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::object::{Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::Namespace;
use crate::avm2::QName;
use crate::context::UpdateContext;
use crate::display_object::{DisplayObject, Lists, TDisplayObject, TDisplayObjectContainer};
use gc_arena::{GcCell, MutationContext};
use std::cmp::min;

/// Implements `flash.display.DisplayObjectContainer`'s instance constructor.
pub fn instance_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("You cannot construct DisplayObjectContainer directly.".into())
}

/// Implements `flash.display.DisplayObjectContainer`'s native instance constructor.
pub fn native_instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        activation.super_init(this, &[])?;
    }

    Ok(Value::Undefined)
}

/// Implements `flash.display.DisplayObjectContainer`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Validate if we can add a child to a parent at a given index.
///
/// There are several conditions which should cause an add operation to fail:
///
///  * The index is off the end of the child list of the proposed parent.
///  * The child is already a transitive child of the proposed parent.
fn validate_add_operation<'gc>(
    new_parent: DisplayObject<'gc>,
    proposed_child: DisplayObject<'gc>,
    proposed_index: usize,
) -> Result<(), Error> {
    let ctr = new_parent
        .as_container()
        .ok_or("ArgumentError: Parent is not a DisplayObjectContainer")?;

    let mut checking_parent = Some(new_parent);

    while let Some(tp) = checking_parent {
        if DisplayObject::ptr_eq(tp, proposed_child) {
            return Err(
                "ArgumentError: Proposed child is an ancestor of the proposed parent, you cannot add the child to the parent"
                    .into(),
            );
        }

        checking_parent = tp.parent();
    }

    if proposed_index > ctr.num_children() {
        return Err("RangeError: Index position does not exist in the child list".into());
    }

    Ok(())
}

/// Validate if we can remove a child from a given parent.
///
/// There are several conditions which should cause a remove operation to fail:
///
///  * The child is not a child of the parent
fn validate_remove_operation<'gc>(
    old_parent: DisplayObject<'gc>,
    proposed_child: DisplayObject<'gc>,
) -> Result<(), Error> {
    let old_ctr = old_parent
        .as_container()
        .ok_or("ArgumentError: Parent is not a DisplayObjectContainer")?;

    for child in old_ctr.iter_render_list() {
        if DisplayObject::ptr_eq(child, proposed_child) {
            return Ok(());
        }
    }

    Err("ArgumentError: Cannot remove object from display list it is not a child of.".into())
}

/// Remove an element from it's parent display list.
fn remove_child_from_displaylist<'gc>(
    context: &mut UpdateContext<'_, 'gc, '_>,
    child: DisplayObject<'gc>,
) {
    if let Some(parent) = child.parent() {
        if let Some(mut ctr) = parent.as_container() {
            ctr.remove_child(context, child, Lists::all());
        }
    }
}

/// Add the `child` to `parent`'s display list.
pub(super) fn add_child_to_displaylist<'gc>(
    context: &mut UpdateContext<'_, 'gc, '_>,
    parent: DisplayObject<'gc>,
    child: DisplayObject<'gc>,
    index: usize,
) {
    if let Some(mut ctr) = parent.as_container() {
        ctr.insert_at_index(context, child, index);
        child.set_placed_by_script(context.gc_context, true);
    }
}

/// Implements `DisplayObjectContainer.getChildAt`
pub fn get_child_at<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(dobj) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_container())
    {
        let index = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_i32(activation)?;
        let child = dobj.child_by_index(index as usize).ok_or_else(|| {
            format!(
                "RangeError: Display object container has no child with id {}",
                index
            )
        })?;

        return Ok(child.object2());
    }

    Ok(Value::Undefined)
}

/// Implements `DisplayObjectContainer.getChildByName`
pub fn get_child_by_name<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(dobj) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_container())
    {
        let name = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_string(activation)?;
        if let Some(child) = dobj.child_by_name(&name, false) {
            return Ok(child.object2());
        } else {
            log::warn!("Display object container has no child with name {}", name);
            return Ok(Value::Null);
        }
    }

    Ok(Value::Undefined)
}

/// Implements `DisplayObjectContainer.addChild`
pub fn add_child<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(parent) = this.and_then(|this| this.as_display_object()) {
        if let Some(ctr) = parent.as_container() {
            let child = args
                .get(0)
                .cloned()
                .unwrap_or(Value::Undefined)
                .as_object()
                .and_then(|o| o.as_display_object())
                .ok_or("ArgumentError: Child not a valid display object")?;
            let target_index = ctr.num_children();

            validate_add_operation(parent, child, target_index)?;
            add_child_to_displaylist(&mut activation.context, parent, child, target_index);

            return Ok(child.object2());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `DisplayObjectContainer.addChildAt`
pub fn add_child_at<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(parent) = this.and_then(|this| this.as_display_object()) {
        let child = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .as_object()
            .and_then(|o| o.as_display_object())
            .ok_or("ArgumentError: Child not a valid display object")?;
        let target_index = args
            .get(1)
            .cloned()
            .ok_or("ArgumentError: Index to add child at not specified")?
            .coerce_to_i32(activation)? as usize;

        validate_add_operation(parent, child, target_index)?;
        add_child_to_displaylist(&mut activation.context, parent, child, target_index);

        return Ok(child.object2());
    }

    Ok(Value::Undefined)
}

/// Implements `DisplayObjectContainer.removeChild`
pub fn remove_child<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(parent) = this.and_then(|this| this.as_display_object()) {
        let child = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .as_object()
            .and_then(|o| o.as_display_object())
            .ok_or("ArgumentError: Child not a valid display object")?;

        validate_remove_operation(parent, child)?;
        remove_child_from_displaylist(&mut activation.context, child);

        return Ok(child.object2());
    }

    Ok(Value::Undefined)
}

/// Implements `DisplayObjectContainer.numChildren`
pub fn num_children<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(parent) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_container())
    {
        return Ok(parent.num_children().into());
    }

    Ok(Value::Undefined)
}

/// Implements `DisplayObjectContainer.contains`
pub fn contains<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(parent) = this.and_then(|this| this.as_display_object()) {
        if parent.as_container().is_some() {
            if let Some(child) = args
                .get(0)
                .cloned()
                .unwrap_or(Value::Undefined)
                .as_object()
                .and_then(|o| o.as_display_object())
            {
                let mut maybe_child_parent = Some(child);
                while let Some(child_parent) = maybe_child_parent {
                    if DisplayObject::ptr_eq(child_parent, parent) {
                        return Ok(true.into());
                    }

                    maybe_child_parent = child_parent.parent();
                }
            }
        }
    }

    Ok(false.into())
}

/// Implements `DisplayObjectContainer.getChildIndex`
pub fn get_child_index<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(parent) = this.and_then(|this| this.as_display_object()) {
        if let Some(ctr) = parent.as_container() {
            let target_child = args
                .get(0)
                .cloned()
                .unwrap_or(Value::Undefined)
                .as_object()
                .and_then(|o| o.as_display_object());

            if let Some(target_child) = target_child {
                for (i, child) in ctr.iter_render_list().enumerate() {
                    if DisplayObject::ptr_eq(child, target_child) {
                        return Ok(i.into());
                    }
                }
            }
        }
    }

    Err("ArgumentError: Child is not a child of this object".into())
}

/// Implements `DisplayObjectContainer.removeChildAt`
pub fn remove_child_at<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(parent) = this.and_then(|this| this.as_display_object()) {
        if let Some(mut ctr) = parent.as_container() {
            let target_child = args
                .get(0)
                .cloned()
                .unwrap_or(Value::Undefined)
                .coerce_to_i32(activation)?;

            if target_child >= ctr.num_children() as i32 || target_child < 0 {
                return Err(format!(
                    "RangeError: {} does not exist in the child list (valid range is 0 to {})",
                    target_child,
                    ctr.num_children()
                )
                .into());
            }

            let child = ctr.child_by_index(target_child as usize).unwrap();

            ctr.remove_child(&mut activation.context, child, Lists::all());

            return Ok(child.object2());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `DisplayObjectContainer.removeChildren`
pub fn remove_children<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(parent) = this.and_then(|this| this.as_display_object()) {
        if let Some(mut ctr) = parent.as_container() {
            let from = args
                .get(0)
                .cloned()
                .unwrap_or_else(|| 0.into())
                .coerce_to_i32(activation)?;
            let to = args
                .get(1)
                .cloned()
                .unwrap_or_else(|| i32::MAX.into())
                .coerce_to_i32(activation)?;

            if from >= ctr.num_children() as i32 || from < 0 {
                return Err(format!(
                    "RangeError: Starting position {} does not exist in the child list (valid range is 0 to {})",
                    from,
                    ctr.num_children()
                )
                .into());
            }

            if (to >= ctr.num_children() as i32 || to < 0) && to != i32::MAX {
                return Err(format!(
                    "RangeError: Ending position {} does not exist in the child list (valid range is 0 to {})",
                    to,
                    ctr.num_children()
                )
                .into());
            }

            if from > to {
                return Err(format!("RangeError: Range {} to {} is invalid", from, to).into());
            }

            ctr.remove_range(
                &mut activation.context,
                from as usize..min(ctr.num_children(), to as usize + 1),
            );
        }
    }

    Ok(Value::Undefined)
}

/// Implements `DisplayObjectContainer.setChildIndex`
pub fn set_child_index<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(parent) = this.and_then(|this| this.as_display_object()) {
        let child = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .as_object()
            .and_then(|o| o.as_display_object())
            .ok_or("ArgumentError: Child not a valid display object")?;
        let target_index = args
            .get(1)
            .cloned()
            .ok_or("ArgumentError: Index to add child at not specified")?
            .coerce_to_i32(activation)? as usize;

        let child_parent = child.parent();
        if child_parent.is_none() || !DisplayObject::ptr_eq(child_parent.unwrap(), parent) {
            return Err("ArgumentError: Given child is not a child of this display object".into());
        }

        validate_add_operation(parent, child, target_index)?;
        add_child_to_displaylist(&mut activation.context, parent, child, target_index);

        return Ok(child.object2());
    }

    Ok(Value::Undefined)
}

/// Implements `DisplayObjectContainer.swapChildrenAt`
pub fn swap_children_at<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(parent) = this.and_then(|this| this.as_display_object()) {
        if let Some(mut ctr) = parent.as_container() {
            let index0 = args
                .get(0)
                .cloned()
                .unwrap_or(Value::Undefined)
                .coerce_to_i32(activation)?;
            let index1 = args
                .get(1)
                .cloned()
                .unwrap_or(Value::Undefined)
                .coerce_to_i32(activation)?;
            let bounds = ctr.num_children();

            if index0 < 0 || index0 as usize >= bounds {
                return Err(format!("RangeError: Index {} is out of bounds", index0).into());
            }

            if index1 < 0 || index1 as usize >= bounds {
                return Err(format!("RangeError: Index {} is out of bounds", index1).into());
            }

            let child0 = ctr.child_by_index(index0 as usize).unwrap();
            let child1 = ctr.child_by_index(index1 as usize).unwrap();

            child0.set_placed_by_script(activation.context.gc_context, true);
            child1.set_placed_by_script(activation.context.gc_context, true);

            ctr.swap_at_index(&mut activation.context, index0 as usize, index1 as usize);
        }
    }

    Ok(Value::Undefined)
}

/// Implements `DisplayObjectContainer.swapChildren`
pub fn swap_children<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(parent) = this.and_then(|this| this.as_display_object()) {
        if let Some(mut ctr) = parent.as_container() {
            let child0 = args
                .get(0)
                .cloned()
                .unwrap_or(Value::Undefined)
                .as_object()
                .and_then(|o| o.as_display_object())
                .ok_or("ArgumentError: Child is not a display object")?;
            let child1 = args
                .get(1)
                .cloned()
                .unwrap_or(Value::Undefined)
                .as_object()
                .and_then(|o| o.as_display_object())
                .ok_or("ArgumentError: Child is not a display object")?;

            let index0 = ctr
                .iter_render_list()
                .position(|a| DisplayObject::ptr_eq(a, child0))
                .ok_or("ArgumentError: Child is not a child of this display object")?;
            let index1 = ctr
                .iter_render_list()
                .position(|a| DisplayObject::ptr_eq(a, child1))
                .ok_or("ArgumentError: Child is not a child of this display object")?;

            child0.set_placed_by_script(activation.context.gc_context, true);
            child1.set_placed_by_script(activation.context.gc_context, true);

            ctr.swap_at_index(&mut activation.context, index0, index1);
        }
    }

    Ok(Value::Undefined)
}

/// Implements `DisplayObjectContainer.stopAllMovieClips`
pub fn stop_all_movie_clips<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(parent) = this.and_then(|this| this.as_display_object()) {
        if let Some(mc) = parent.as_movie_clip() {
            mc.stop(&mut activation.context);
        }

        if let Some(ctr) = parent.as_container() {
            for child in ctr.iter_render_list() {
                if child.as_container().is_some() {
                    let child_this = child.object2().as_object();
                    stop_all_movie_clips(activation, child_this, &[])?;
                }
            }
        }
    }

    Ok(Value::Undefined)
}

/// Stubs `DisplayObjectContainer.getObjectsUnderPoint`
pub fn get_objects_under_point<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("DisplayObjectContainer.getObjectsUnderPoint not yet implemented".into())
}

/// Stubs `DisplayObjectContainer.areInaccessibleObjectsUnderPoint`
pub fn are_inaccessible_objects_under_point<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("DisplayObjectContainer.areInaccessibleObjectsUnderPoint not yet implemented".into())
}

pub fn mouse_children<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    log::warn!("DisplayObjectContainer.mouseChildren getter: not yet implemented");
    Ok(Value::Undefined)
}

pub fn set_mouse_children<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    log::warn!("DisplayObjectContainer.mouseChildren setter: not yet implemented");
    Ok(Value::Undefined)
}

/// Construct `DisplayObjectContainer`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(
            Namespace::package("flash.display"),
            "DisplayObjectContainer",
        ),
        Some(QName::new(Namespace::package("flash.display"), "InteractiveObject").into()),
        Method::from_builtin(
            instance_init,
            "<DisplayObjectContainer instance initializer>",
            mc,
        ),
        Method::from_builtin(class_init, "<DisplayObjectContainer class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    write.set_native_instance_init(Method::from_builtin(
        native_instance_init,
        "<DisplayObjectContainer native instance initializer>",
        mc,
    ));

    const PUBLIC_INSTANCE_PROPERTIES: &[(
        &str,
        Option<NativeMethodImpl>,
        Option<NativeMethodImpl>,
    )] = &[
        ("numChildren", Some(num_children), None),
        (
            "mouseChildren",
            Some(mouse_children),
            Some(set_mouse_children),
        ),
    ];
    write.define_public_builtin_instance_properties(mc, PUBLIC_INSTANCE_PROPERTIES);

    const PUBLIC_INSTANCE_METHODS: &[(&str, NativeMethodImpl)] = &[
        ("getChildAt", get_child_at),
        ("getChildByName", get_child_by_name),
        ("addChild", add_child),
        ("addChildAt", add_child_at),
        ("removeChild", remove_child),
        ("contains", contains),
        ("getChildIndex", get_child_index),
        ("removeChildAt", remove_child_at),
        ("removeChildren", remove_children),
        ("setChildIndex", set_child_index),
        ("swapChildrenAt", swap_children_at),
        ("swapChildren", swap_children),
        ("stopAllMovieClips", stop_all_movie_clips),
        ("getObjectsUnderPoint", get_objects_under_point),
        (
            "areInaccessibleObjectsUnderPoint",
            are_inaccessible_objects_under_point,
        ),
    ];
    write.define_public_builtin_instance_methods(mc, PUBLIC_INSTANCE_METHODS);

    class
}
