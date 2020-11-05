//! `flash.display.DisplayObjectContainer` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::method::Method;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{Object, TObject};
use crate::avm2::traits::Trait;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::context::UpdateContext;
use crate::display_object::{DisplayObject, TDisplayObject};
use gc_arena::{GcCell, MutationContext};

/// Implements `flash.display.DisplayObjectContainer`'s instance constructor.
pub fn instance_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
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
    let mc = new_parent
        .as_movie_clip()
        .ok_or("ArgumentError: Parent is not a movieclip")?;

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

    if proposed_index > mc.num_children() {
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
    let _ = old_parent
        .as_movie_clip()
        .ok_or("ArgumentError: Parent is not a movieclip")?;

    for child in old_parent.children() {
        if DisplayObject::ptr_eq(child, proposed_child) {
            return Ok(());
        }
    }

    Err("ArgumentError: Cannot remove object from display list it is not a child of.".into())
}

/// Remove an element from it's parent display list.
///
/// ActionScript 3 works in child indexes, even though the underlying display
/// list tracks children by depth. We attempt to maintain the same depth
/// numbers used previously, shifting each object down so that, for example,
/// object B has the depth that object A used to have, object C has the depth
/// object B used to have, etc.
fn remove_child_from_displaylist<'gc>(
    context: &mut UpdateContext<'_, 'gc, '_>,
    child: DisplayObject<'gc>,
) {
    if let Some(parent) = child.parent() {
        if let Some(mut mc) = parent.as_movie_clip() {
            mc.remove_child_from_avm(context, child);
        }
    }
}

/// Add the `child` to `parent`'s display list.
///
/// ActionScript 3 works in child indexes, even though the underlying display
/// list tracks children by depth. We attempt to maintain the same depth
/// numbers used previously, shifting each object down so that, for example,
/// object A has the depth that object B used to have, object B has the depth
/// that object C used to have, etc. If we run out of depths to reuse then we
/// start incrementing by one.
fn add_child_to_displaylist<'gc>(
    context: &mut UpdateContext<'_, 'gc, '_>,
    parent: DisplayObject<'gc>,
    child: DisplayObject<'gc>,
    index: usize,
) {
    //TODO: Non-MC objects can be containers in AS3!
    if let Some(mut mc) = parent.as_movie_clip() {
        mc.add_child_from_avm_by_id(context, child, index);
        child.set_placed_by_script(context.gc_context, true);
    }
}

/// Implements `DisplayObjectContainer.getChildAt`
pub fn get_child_at<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        let id = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_u32(activation)?;
        let child = dobj.get_child_by_id(id as usize).ok_or_else(|| {
            format!(
                "RangeError: Display object container has no child with id {}",
                id
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
    if let Some(dobj) = this.and_then(|this| this.as_display_object()) {
        let name = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_string(activation)?;
        let child = dobj.get_child_by_name(&name, false).ok_or_else(|| {
            format!(
                "RangeError: Display object container has no child with name {}",
                name
            )
        })?;

        return Ok(child.object2());
    }

    Ok(Value::Undefined)
}

/// Implements `DisplayObjectContainer.addChild`
pub fn add_child<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(parent) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_movie_clip())
    {
        let child = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_object(activation)?
            .as_display_object()
            .ok_or("ArgumentError: Child not a valid display object")?;
        let target_index = parent.num_children();

        validate_add_operation(parent.into(), child, target_index)?;
        add_child_to_displaylist(&mut activation.context, parent.into(), child, target_index);

        return Ok(child.object2());
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
            .coerce_to_object(activation)?
            .as_display_object()
            .ok_or("ArgumentError: Child not a valid display object")?;
        let target_index = args
            .get(1)
            .cloned()
            .ok_or("ArgumentError: Index to add child at not specified")?
            .coerce_to_u32(activation)? as usize;

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
            .coerce_to_object(activation)?
            .as_display_object()
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
        .and_then(|this| this.as_movie_clip())
    {
        return Ok(parent.num_children().into());
    }

    Ok(Value::Undefined)
}

/// Implements `DisplayObjectContainer.contains`
pub fn contains<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(parent) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_movie_clip())
    {
        if let Some(child) = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_object(activation)?
            .as_display_object()
        {
            let mut maybe_child_parent = Some(child);
            while let Some(child_parent) = maybe_child_parent {
                if DisplayObject::ptr_eq(child_parent, parent.into()) {
                    return Ok(true.into());
                }

                maybe_child_parent = child_parent.parent();
            }
        }
    }

    Ok(false.into())
}

/// Construct `DisplayObjectContainer`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(
            Namespace::package("flash.display"),
            "DisplayObjectContainer",
        ),
        Some(QName::new(Namespace::package("flash.display"), "InteractiveObject").into()),
        Method::from_builtin(instance_init),
        Method::from_builtin(class_init),
        mc,
    );

    let mut write = class.write(mc);

    write.define_instance_trait(Trait::from_method(
        QName::new(Namespace::public_namespace(), "getChildAt"),
        Method::from_builtin(get_child_at),
    ));
    write.define_instance_trait(Trait::from_method(
        QName::new(Namespace::public_namespace(), "getChildByName"),
        Method::from_builtin(get_child_by_name),
    ));
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public_namespace(), "numChildren"),
        Method::from_builtin(num_children),
    ));
    write.define_instance_trait(Trait::from_method(
        QName::new(Namespace::public_namespace(), "addChild"),
        Method::from_builtin(add_child),
    ));
    write.define_instance_trait(Trait::from_method(
        QName::new(Namespace::public_namespace(), "addChildAt"),
        Method::from_builtin(add_child_at),
    ));
    write.define_instance_trait(Trait::from_method(
        QName::new(Namespace::public_namespace(), "removeChild"),
        Method::from_builtin(remove_child),
    ));
    write.define_instance_trait(Trait::from_method(
        QName::new(Namespace::public_namespace(), "contains"),
        Method::from_builtin(contains),
    ));

    class
}
