//! `flash.display.DisplayObjectContainer` builtin/prototype

use swf::Point;
use swf::Twips;

use crate::avm2::activation::Activation;
use crate::avm2::error::{
    make_error_2006, make_error_2024, make_error_2025, make_error_2150, make_error_2180,
    make_error_3783,
};
use crate::avm2::globals::slots::flash_geom_point as point_slots;
use crate::avm2::object::{Object, TObject as _};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::{ArrayObject, ArrayStorage, Error};
use crate::avm2_stub_method;
use crate::context::UpdateContext;
use crate::display_object::HitTestOptions;
use crate::display_object::{DisplayObject, TDisplayObject, TDisplayObjectContainer};
use std::cmp::min;

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
        .expect("Parent must be a DisplayObjectContainer");

    if let DisplayObject::Stage(_) = proposed_child {
        return Err(make_error_3783(activation));
    }

    if !proposed_child.movie().is_action_script_3() && activation.context.root_swf.version() > 9 {
        return Err(make_error_2180(activation));
    }

    if DisplayObject::ptr_eq(proposed_child, new_parent) {
        return Err(make_error_2024(activation));
    }

    let mut checking_parent = Some(new_parent);

    while let Some(tp) = checking_parent {
        if DisplayObject::ptr_eq(tp, proposed_child) {
            return Err(make_error_2150(activation));
        }

        checking_parent = tp.parent();
    }

    if proposed_index > ctr.num_children() {
        return Err(make_error_2006(activation));
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
        .expect("Parent must be a DisplayObjectContainer");

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
            child.set_placed_by_avm2_script(true);
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
        child.set_placed_by_avm2_script(true);
        ctr.insert_at_index(context, child, index);
    }
}

/// Implements `DisplayObjectContainer.getChildAt`
pub fn get_child_at<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(dobj) = this
        .as_display_object()
        .and_then(|this| this.as_container())
    {
        let index = args.get_i32(0);
        return if let Some(child) = dobj.child_by_index(index as usize) {
            Ok(child.object2_or_null())
        } else {
            Err(make_error_2006(activation))
        };
    }

    Ok(Value::Null)
}

/// Implements `DisplayObjectContainer.getChildByName`
pub fn get_child_by_name<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(dobj) = this
        .as_display_object()
        .and_then(|this| this.as_container())
    {
        let name = args.get_string(activation, 0);
        if let Some(child) = dobj.child_by_name(&name, true) {
            return Ok(child.object2_or_null());
        } else {
            return Ok(Value::Null);
        }
    }

    Ok(Value::Null)
}

/// Implements `DisplayObjectContainer.addChild`
pub fn add_child<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(parent) = this.as_display_object() {
        if let Some(ctr) = parent.as_container() {
            let child = args
                .get_object(activation, 0, "child")?
                .as_display_object()
                .expect("Child must be a display object");

            let target_index = ctr.num_children();

            validate_add_operation(activation, parent, child, target_index)?;
            add_child_to_displaylist(activation.context, parent, child, target_index);

            return Ok(child.object2_or_null());
        }
    }

    Ok(Value::Null)
}

/// Implements `DisplayObjectContainer.addChildAt`
pub fn add_child_at<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(parent) = this.as_display_object() {
        let child = args
            .get_object(activation, 0, "child")?
            .as_display_object()
            .expect("Child must be a display object");
        let target_index = args.get_u32(1) as usize;

        validate_add_operation(activation, parent, child, target_index)?;
        add_child_to_displaylist(activation.context, parent, child, target_index);

        return Ok(child.object2_or_null());
    }

    Ok(Value::Null)
}

/// Implements `DisplayObjectContainer.removeChild`
pub fn remove_child<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(parent) = this.as_display_object() {
        let child = args
            .get_object(activation, 0, "child")?
            .as_display_object()
            .expect("Child must be a display object");

        validate_remove_operation(activation, parent, child)?;
        remove_child_from_displaylist(activation.context, child);

        return Ok(child.object2_or_null());
    }

    Ok(Value::Null)
}

/// Implements `DisplayObjectContainer.numChildren`
pub fn get_num_children<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(parent) = this
        .as_display_object()
        .and_then(|this| this.as_container())
    {
        return Ok(parent.num_children().into());
    }

    Ok(0.into())
}

/// Implements `DisplayObjectContainer.contains`
pub fn contains<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

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
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

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
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(parent) = this.as_display_object() {
        if let Some(mut ctr) = parent.as_container() {
            let target_child = args.get_i32(0);

            if target_child >= ctr.num_children() as i32 || target_child < 0 {
                return Err(make_error_2006(activation));
            }

            let child = ctr.child_by_index(target_child as usize).unwrap();
            child.set_placed_by_avm2_script(true);

            ctr.remove_child(activation.context, child);

            return Ok(child.object2_or_null());
        }
    }

    Ok(Value::Null)
}

/// Implements `DisplayObjectContainer.removeChildren`
pub fn remove_children<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(parent) = this.as_display_object() {
        if let Some(mut ctr) = parent.as_container() {
            let from = args.get_i32(0);
            let to = args.get_i32(1);

            // Flash special-cases `to==i32::MAX` to not throw an error,
            // even if `from` is not in range
            // https://github.com/ruffle-rs/ruffle/issues/11382

            if (from >= ctr.num_children() as i32 || from < 0) && to != i32::MAX {
                return Err(make_error_2006(activation));
            }

            if (to >= ctr.num_children() as i32 || to < 0) && to != i32::MAX {
                return Err(make_error_2006(activation));
            }

            if from > to {
                return Err(make_error_2006(activation));
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
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(parent) = this.as_display_object() {
        let child = args
            .get_object(activation, 0, "child")?
            .as_display_object()
            .expect("Child must be a display object");
        let target_index = args.get_u32(1) as usize;

        let child_parent = child.parent();
        if child_parent.is_none() || !DisplayObject::ptr_eq(child_parent.unwrap(), parent) {
            return Err(make_error_2025(activation));
        }

        validate_add_operation(activation, parent, child, target_index)?;
        add_child_to_displaylist(activation.context, parent, child, target_index);
    }

    Ok(Value::Undefined)
}

/// Implements `DisplayObjectContainer.swapChildrenAt`
pub fn swap_children_at<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(parent) = this.as_display_object() {
        if let Some(mut ctr) = parent.as_container() {
            let index0 = args.get_i32(0);
            let index1 = args.get_i32(1);
            let bounds = ctr.num_children();

            if index0 < 0 || index0 as usize >= bounds {
                return Err(make_error_2006(activation));
            }

            if index1 < 0 || index1 as usize >= bounds {
                return Err(make_error_2006(activation));
            }

            let child0 = ctr.child_by_index(index0 as usize).unwrap();
            let child1 = ctr.child_by_index(index1 as usize).unwrap();

            child0.set_placed_by_avm2_script(true);
            child1.set_placed_by_avm2_script(true);

            ctr.swap_at_index(activation.context, index0 as usize, index1 as usize);
        }
    }

    Ok(Value::Undefined)
}

/// Implements `DisplayObjectContainer.swapChildren`
pub fn swap_children<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(parent) = this.as_display_object() {
        if let Some(mut ctr) = parent.as_container() {
            let child0 = args
                .get_object(activation, 0, "child")?
                .as_display_object()
                .expect("Child must be a display object");

            let index0 = ctr
                .iter_render_list()
                .position(|a| DisplayObject::ptr_eq(a, child0))
                .ok_or(make_error_2025(activation))?;

            let child1 = args
                .get_object(activation, 1, "child")?
                .as_display_object()
                .expect("Child must be a display object");

            let index1 = ctr
                .iter_render_list()
                .position(|a| DisplayObject::ptr_eq(a, child1))
                .ok_or(make_error_2025(activation))?;

            child0.set_placed_by_avm2_script(true);
            child1.set_placed_by_avm2_script(true);

            ctr.swap_at_index(activation.context, index0, index1);
        }
    }

    Ok(Value::Undefined)
}

/// Implements `DisplayObjectContainer.stopAllMovieClips`
pub fn stop_all_movie_clips<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(parent) = this.as_display_object() {
        if let Some(mc) = parent.as_movie_clip() {
            mc.stop(activation.context);
        }

        if let Some(ctr) = parent.as_container() {
            for child in ctr.iter_render_list() {
                if child.as_container().is_some() {
                    if let Some(child_this) = child.object2() {
                        stop_all_movie_clips(activation, child_this.into(), &[])?;
                    }
                }
            }
        }
    }

    Ok(Value::Undefined)
}

pub fn get_objects_under_point<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let thisobj = this.as_object().unwrap();

    avm2_stub_method!(
        activation,
        "flash.display.DisplayObjectContainer",
        "getObjectsUnderPoint",
        "proper hit-test behavior"
    );
    // FIXME: different result at from_shumway/hittesting/hittesting "two-layer button".

    let point = args.get_object(activation, 0, "point")?;
    let x = point
        .get_slot(point_slots::X)
        .coerce_to_number(activation)?;
    let y = point
        .get_slot(point_slots::Y)
        .coerce_to_number(activation)?;

    let point = Point {
        x: Twips::from_pixels(x),
        y: Twips::from_pixels(y),
    };

    let mut under_point = Vec::new();
    let mut children = vec![thisobj.as_display_object().unwrap()];
    let options =
        HitTestOptions::SKIP_MASK | HitTestOptions::SKIP_INVISIBLE | HitTestOptions::SKIP_CHILDREN;

    while let Some(child) = children.pop() {
        let obj = child.object2();
        if let Some(obj) = obj {
            let obj = Object::StageObject(obj);

            if obj != thisobj && child.hit_test_shape(activation.context, point, options) {
                under_point.push(Some(obj.into()));
            }
        }
        if let Some(container) = child.as_container() {
            for child in container.iter_render_list().rev() {
                children.push(child);
            }
        }
    }

    Ok(
        ArrayObject::from_storage(activation.context, ArrayStorage::from_storage(under_point))
            .into(),
    )
}

pub fn are_inaccessible_objects_under_point<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
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
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(dobj) = this
        .as_display_object()
        .and_then(|this| this.as_container())
    {
        return Ok(dobj.raw_container().mouse_children().into());
    }
    Ok(false.into())
}

pub fn set_mouse_children<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(dobj) = this
        .as_display_object()
        .and_then(|this| this.as_container())
    {
        let mouse_children = args.get_bool(0);

        dobj.raw_container_mut(activation.gc())
            .set_mouse_children(mouse_children);
    }
    Ok(Value::Undefined)
}

pub fn get_tab_children<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(obj) = this
        .as_display_object()
        .and_then(|this| this.as_container())
    {
        Ok(Value::Bool(obj.is_tab_children(activation.context)))
    } else {
        Ok(false.into())
    }
}

pub fn set_tab_children<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(obj) = this
        .as_display_object()
        .and_then(|this| this.as_container())
    {
        let value = args.get_bool(0);
        obj.set_tab_children(activation.context, value);
    }

    Ok(Value::Undefined)
}
