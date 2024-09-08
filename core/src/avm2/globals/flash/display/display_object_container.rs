//! `flash.display.DisplayObjectContainer` builtin/prototype

use swf::Point;
use swf::Twips;

use crate::avm2::activation::Activation;
use crate::avm2::error::{argument_error, make_error_2025, range_error};
use crate::avm2::object::{Object, TObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::{ArrayObject, ArrayStorage, Error};
use crate::avm2_stub_method;
use crate::context::UpdateContext;
use crate::display_object::HitTestOptions;
use crate::display_object::{DisplayObject, TDisplayObject, TDisplayObjectContainer};
use std::cmp::min;

/// Implements `flash.display.DisplayObjectContainer`'s native instance constructor.
pub fn super_init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    activation.super_init(this, &[])?;

    Ok(Value::Undefined)
}

/// Validate if we can add a child to a parent at a given index.
///
/// There are several conditions which should cause an add operation to fail:
///
///  * The index is off the end of the child list of the proposed parent.
///  * The child is already a transitive child of the proposed parent.
fn validate_add_operation<'gc>(
    activation: &mut Activation<'_, 'gc>,
    new_parent: DisplayObject<'gc>,
    proposed_child: DisplayObject<'gc>,
    proposed_index: usize,
) -> Result<(), Error<'gc>> {
    let ctr = new_parent
        .as_container()
        .ok_or("ArgumentError: Parent is not a DisplayObjectContainer")?;

    if let DisplayObject::Stage(_) = proposed_child {
        return Err(Error::AvmError(argument_error(
            activation,
            "Error #3783: A Stage object cannot be added as the child of another object.",
            3783,
        )?));
    }

    if !proposed_child.movie().is_action_script_3() {
        return Err(Error::AvmError(argument_error(
            activation,
            "Error #2180: It is illegal to move AVM1 content (AS1 or AS2) to a different part of the displayList when it has been loaded into AVM2 (AS3) content.",
            2180,
        )?));
    }

    if DisplayObject::ptr_eq(proposed_child, new_parent) {
        return Err(Error::AvmError(argument_error(
            activation,
            "Error #2024: An object cannot be added as a child of itself.",
            2024,
        )?));
    }

    let mut checking_parent = Some(new_parent);

    while let Some(tp) = checking_parent {
        if DisplayObject::ptr_eq(tp, proposed_child) {
            return Err(Error::AvmError(argument_error(
                activation,
                "Error #2150: An object cannot be added as a child to one of it's children (or children's children, etc.).",
                2150,
            )?));
        }

        checking_parent = tp.parent();
    }

    if proposed_index > ctr.num_children() {
        // Flash error message: The supplied index is out of bounds.
        return Err(Error::AvmError(range_error(
            activation,
            "Index position does not exist in the child list",
            2006,
        )?));
    }

    Ok(())
}

/// Validate if we can remove a child from a given parent.
///
/// There are several conditions which should cause a remove operation to fail:
///
///  * The child is not a child of the parent
fn validate_remove_operation<'gc>(
    activation: &mut Activation<'_, 'gc>,
    old_parent: DisplayObject<'gc>,
    proposed_child: DisplayObject<'gc>,
) -> Result<(), Error<'gc>> {
    let old_ctr = old_parent
        .as_container()
        .ok_or("ArgumentError: Parent is not a DisplayObjectContainer")?;

    for child in old_ctr.iter_render_list() {
        if DisplayObject::ptr_eq(child, proposed_child) {
            return Ok(());
        }
    }

    Err(make_error_2025(activation))
}

/// Remove an element from it's parent display list.
fn remove_child_from_displaylist<'gc>(context: &mut UpdateContext<'gc>, child: DisplayObject<'gc>) {
    if let Some(parent) = child.parent() {
        if let Some(mut ctr) = parent.as_container() {
            child.set_placed_by_script(context.gc_context, true);
            ctr.remove_child(context, child);
        }
    }
}

/// Add the `child` to `parent`'s display list.
pub(super) fn add_child_to_displaylist<'gc>(
    context: &mut UpdateContext<'gc>,
    parent: DisplayObject<'gc>,
    child: DisplayObject<'gc>,
    index: usize,
) {
    if let Some(mut ctr) = parent.as_container() {
        child.set_placed_by_script(context.gc_context, true);
        ctr.insert_at_index(context, child, index);
    }
}

/// Implements `DisplayObjectContainer.getChildAt`
pub fn get_child_at<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this
        .as_display_object()
        .and_then(|this| this.as_container())
    {
        let index = args.get_i32(activation, 0)?;
        return if let Some(child) = dobj.child_by_index(index as usize) {
            Ok(child.object2())
        } else {
            // Flash error message: The supplied index is out of bounds.
            Err(Error::AvmError(range_error(
                activation,
                &format!("Display object container has no child with id {index}"),
                2006,
            )?))
        };
    }

    Ok(Value::Undefined)
}

/// Implements `DisplayObjectContainer.getChildByName`
pub fn get_child_by_name<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this
        .as_display_object()
        .and_then(|this| this.as_container())
    {
        let name = args.get_string(activation, 0)?;
        if let Some(child) = dobj.child_by_name(&name, false) {
            return Ok(child.object2());
        } else {
            return Ok(Value::Null);
        }
    }

    Ok(Value::Undefined)
}

/// Implements `DisplayObjectContainer.addChild`
pub fn add_child<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(parent) = this.as_display_object() {
        if let Some(ctr) = parent.as_container() {
            let child = args
                .get_object(activation, 0, "child")?
                .as_display_object()
                .ok_or("ArgumentError: Child not a valid display object")?;

            let target_index = ctr.num_children();

            validate_add_operation(activation, parent, child, target_index)?;
            add_child_to_displaylist(activation.context, parent, child, target_index);

            return Ok(child.object2());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `DisplayObjectContainer.addChildAt`
pub fn add_child_at<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(parent) = this.as_display_object() {
        let child = args
            .get_object(activation, 0, "child")?
            .as_display_object()
            .ok_or("ArgumentError: Child not a valid display object")?;
        let target_index = args.get_u32(activation, 1)? as usize;

        validate_add_operation(activation, parent, child, target_index)?;
        add_child_to_displaylist(activation.context, parent, child, target_index);

        return Ok(child.object2());
    }

    Ok(Value::Undefined)
}

/// Implements `DisplayObjectContainer.removeChild`
pub fn remove_child<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(parent) = this.as_display_object() {
        let child = args
            .get_object(activation, 0, "child")?
            .as_display_object()
            .ok_or("ArgumentError: Child not a valid display object")?;

        validate_remove_operation(activation, parent, child)?;
        remove_child_from_displaylist(activation.context, child);

        return Ok(child.object2());
    }

    Ok(Value::Undefined)
}

/// Implements `DisplayObjectContainer.numChildren`
pub fn get_num_children<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(parent) = this
        .as_display_object()
        .and_then(|this| this.as_container())
    {
        return Ok(parent.num_children().into());
    }

    Ok(Value::Undefined)
}

/// Implements `DisplayObjectContainer.contains`
pub fn contains<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(parent) = this.as_display_object() {
        if parent.as_container().is_some() {
            if let Some(child) = args.get_object(activation, 0, "child")?.as_display_object() {
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
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(parent) = this.as_display_object() {
        if let Some(ctr) = parent.as_container() {
            let target_child = args.get_object(activation, 0, "child")?.as_display_object();

            if let Some(target_child) = target_child {
                for (i, child) in ctr.iter_render_list().enumerate() {
                    if DisplayObject::ptr_eq(child, target_child) {
                        return Ok(i.into());
                    }
                }
            }
        }
    }

    Err(make_error_2025(activation))
}

/// Implements `DisplayObjectContainer.removeChildAt`
pub fn remove_child_at<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(parent) = this.as_display_object() {
        if let Some(mut ctr) = parent.as_container() {
            let target_child = args.get_i32(activation, 0)?;

            if target_child >= ctr.num_children() as i32 || target_child < 0 {
                // Flash error message: The supplied index is out of bounds.
                return Err(Error::AvmError(range_error(
                    activation,
                    &format!(
                        "{} does not exist in the child list (valid range is 0 to {})",
                        target_child,
                        ctr.num_children()
                    ),
                    2006,
                )?));
            }

            let child = ctr.child_by_index(target_child as usize).unwrap();
            child.set_placed_by_script(activation.context.gc_context, true);

            ctr.remove_child(activation.context, child);

            return Ok(child.object2());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `DisplayObjectContainer.removeChildren`
pub fn remove_children<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(parent) = this.as_display_object() {
        if let Some(mut ctr) = parent.as_container() {
            let from = args.get_i32(activation, 0)?;
            let to = args.get_i32(activation, 1)?;

            // Flash special-cases `to==i32::MAX` to not throw an error,
            // even if `from` is not in range
            // https://github.com/ruffle-rs/ruffle/issues/11382

            if (from >= ctr.num_children() as i32 || from < 0) && to != i32::MAX {
                // Flash error message: The supplied index is out of bounds.
                return Err(Error::AvmError(range_error(
                    activation,
                    &format!(
                        "Starting position {} does not exist in the child list (valid range is 0 to {})",
                        from,
                        ctr.num_children()
                    ),
                    2006,
                )?));
            }

            if (to >= ctr.num_children() as i32 || to < 0) && to != i32::MAX {
                // Flash error message: The supplied index is out of bounds.
                return Err(Error::AvmError(range_error(
                    activation,
                    &format!(
                        "Ending position {} does not exist in the child list (valid range is 0 to {})",
                        to,
                        ctr.num_children()
                    ),
                    2006,
                )?));
            }

            if from > to {
                // Flash error message: The supplied index is out of bounds.
                return Err(Error::AvmError(range_error(
                    activation,
                    &format!("Range {from} to {to} is invalid"),
                    2006,
                )?));
            }

            ctr.remove_range(
                activation.context,
                from as usize..min(ctr.num_children(), to as usize + 1),
            );
        }
    }

    Ok(Value::Undefined)
}

/// Implements `DisplayObjectContainer.setChildIndex`
pub fn set_child_index<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(parent) = this.as_display_object() {
        let child = args
            .get_object(activation, 0, "child")?
            .as_display_object()
            .ok_or("ArgumentError: Child not a valid display object")?;
        let target_index = args.get_u32(activation, 1)? as usize;

        let child_parent = child.parent();
        if child_parent.is_none() || !DisplayObject::ptr_eq(child_parent.unwrap(), parent) {
            return Err(make_error_2025(activation));
        }

        validate_add_operation(activation, parent, child, target_index)?;
        add_child_to_displaylist(activation.context, parent, child, target_index);

        return Ok(child.object2());
    }

    Ok(Value::Undefined)
}

/// Implements `DisplayObjectContainer.swapChildrenAt`
pub fn swap_children_at<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(parent) = this.as_display_object() {
        if let Some(mut ctr) = parent.as_container() {
            let index0 = args.get_i32(activation, 0)?;
            let index1 = args.get_i32(activation, 1)?;
            let bounds = ctr.num_children();

            if index0 < 0 || index0 as usize >= bounds {
                // Flash error message: The supplied index is out of bounds.
                return Err(Error::AvmError(range_error(
                    activation,
                    &format!("Index {index0} is out of bounds",),
                    2006,
                )?));
            }

            if index1 < 0 || index1 as usize >= bounds {
                // Flash error message: The supplied index is out of bounds.
                return Err(Error::AvmError(range_error(
                    activation,
                    &format!("Index {index1} is out of bounds",),
                    2006,
                )?));
            }

            let child0 = ctr.child_by_index(index0 as usize).unwrap();
            let child1 = ctr.child_by_index(index1 as usize).unwrap();

            child0.set_placed_by_script(activation.context.gc_context, true);
            child1.set_placed_by_script(activation.context.gc_context, true);

            ctr.swap_at_index(activation.context, index0 as usize, index1 as usize);
        }
    }

    Ok(Value::Undefined)
}

/// Implements `DisplayObjectContainer.swapChildren`
pub fn swap_children<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(parent) = this.as_display_object() {
        if let Some(mut ctr) = parent.as_container() {
            let child0 = args
                .get_object(activation, 0, "child")?
                .as_display_object()
                .ok_or("ArgumentError: Child is not a display object")?;
            let child1 = args
                .get_object(activation, 1, "child")?
                .as_display_object()
                .ok_or("ArgumentError: Child is not a display object")?;

            let index0 = ctr
                .iter_render_list()
                .position(|a| DisplayObject::ptr_eq(a, child0))
                .ok_or(make_error_2025(activation))?;
            let index1 = ctr
                .iter_render_list()
                .position(|a| DisplayObject::ptr_eq(a, child1))
                .ok_or(make_error_2025(activation))?;

            child0.set_placed_by_script(activation.context.gc_context, true);
            child1.set_placed_by_script(activation.context.gc_context, true);

            ctr.swap_at_index(activation.context, index0, index1);
        }
    }

    Ok(Value::Undefined)
}

/// Implements `DisplayObjectContainer.stopAllMovieClips`
pub fn stop_all_movie_clips<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(parent) = this.as_display_object() {
        if let Some(mc) = parent.as_movie_clip() {
            mc.stop(activation.context);
        }

        if let Some(ctr) = parent.as_container() {
            for child in ctr.iter_render_list() {
                if child.as_container().is_some() {
                    let child_this = child.object2().as_object();

                    if let Some(child_this) = child_this {
                        stop_all_movie_clips(activation, child_this, &[])?;
                    }
                }
            }
        }
    }

    Ok(Value::Undefined)
}

pub fn get_objects_under_point<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(
        activation,
        "flash.display.DisplayObjectContainer",
        "getObjectsUnderPoint",
        "proper hit-test behavior"
    );

    let point = args.get_object(activation, 0, "point")?;
    let x = point
        .get_public_property("x", activation)?
        .coerce_to_number(activation)?;
    let y = point
        .get_public_property("y", activation)?
        .coerce_to_number(activation)?;

    let point = Point {
        x: Twips::from_pixels(x),
        y: Twips::from_pixels(y),
    };

    let mut under_point = Vec::new();
    let mut children = vec![this.as_display_object().unwrap()];
    // FIXME - what are the actual options?
    let options = HitTestOptions::SKIP_MASK;
    while let Some(child) = children.pop() {
        if child.hit_test_shape(activation.context, point, options) {
            under_point.push(Some(child.object2()));
        }
        if let Some(container) = child.as_container() {
            for child in container.iter_render_list() {
                children.push(child);
            }
        }
    }

    Ok(ArrayObject::from_storage(activation, ArrayStorage::from_storage(under_point))?.into())
}

pub fn are_inaccessible_objects_under_point<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(
        activation,
        "flash.display.DisplayObjectContainer",
        "areInaccessibleObjectsUnderPoint"
    );
    Ok(false.into())
}

pub fn get_mouse_children<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this
        .as_display_object()
        .and_then(|this| this.as_container())
    {
        return Ok(dobj.raw_container().mouse_children().into());
    }
    Ok(Value::Undefined)
}

pub fn set_mouse_children<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dobj) = this
        .as_display_object()
        .and_then(|this| this.as_container())
    {
        let mouse_children = args.get_bool(0);

        dobj.raw_container_mut(activation.context.gc_context)
            .set_mouse_children(mouse_children);
    }
    Ok(Value::Undefined)
}

pub fn get_tab_children<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(obj) = this
        .as_display_object()
        .and_then(|this| this.as_container())
    {
        Ok(Value::Bool(obj.is_tab_children(activation.context)))
    } else {
        Ok(Value::Undefined)
    }
}

pub fn set_tab_children<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(obj) = this
        .as_display_object()
        .and_then(|this| this.as_container())
    {
        let value = args.get_bool(0);
        obj.set_tab_children(activation.context, value);
    }

    Ok(Value::Undefined)
}
