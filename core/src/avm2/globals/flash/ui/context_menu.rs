use crate::avm2::activation::Activation;
use crate::avm2::object::{Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::context_menu;
use crate::display_object::DisplayObject;

pub fn hide_built_in_items<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Value::Object(items) = this.get_public_property("builtInItems", activation)? {
        // items is a ContextMenuBuiltInItems
        items.set_public_property("forwardAndBack", Value::Bool(false), activation)?;
        items.set_public_property("loop", Value::Bool(false), activation)?;
        items.set_public_property("play", Value::Bool(false), activation)?;
        items.set_public_property("print", Value::Bool(false), activation)?;
        items.set_public_property("quality", Value::Bool(false), activation)?;
        items.set_public_property("rewind", Value::Bool(false), activation)?;
        items.set_public_property("save", Value::Bool(false), activation)?;
        items.set_public_property("zoom", Value::Bool(false), activation)?;
    }

    Ok(Value::Undefined)
}

pub fn make_context_menu_state<'gc>(
    menu: Option<Object<'gc>>,
    object: Option<DisplayObject<'gc>>,
    activation: &mut Activation<'_, 'gc>,
) -> context_menu::ContextMenuState<'gc> {
    let mut result = context_menu::ContextMenuState::new();

    result.set_display_object(object);

    macro_rules! check_bool {
        ( $obj:expr, $name:expr, $value:expr ) => {
            matches!(
                $obj.get_public_property($name, activation),
                Ok(Value::Bool($value))
            )
        };
    }

    let mut builtin_items = context_menu::BuiltInItemFlags::for_stage(activation.context.stage);
    if let Some(menu) = menu {
        if let Ok(Value::Object(builtins)) = menu.get_public_property("builtInItems", activation) {
            if check_bool!(builtins, "zoom", false) {
                builtin_items.zoom = false;
            }
            if check_bool!(builtins, "quality", false) {
                builtin_items.quality = false;
            }
            if check_bool!(builtins, "play", false) {
                builtin_items.play = false;
            }
            if check_bool!(builtins, "loop", false) {
                builtin_items.loop_ = false;
            }
            if check_bool!(builtins, "rewind", false) {
                builtin_items.rewind = false;
            }
            if check_bool!(builtins, "forwardAndBack", false) {
                builtin_items.forward_and_back = false;
            }
            if check_bool!(builtins, "print", false) {
                builtin_items.print = false;
            }
        }
    }

    result.build_builtin_items(builtin_items, activation.context);

    if let Some(menu) = menu {
        if let Ok(Value::Object(custom_items)) = menu.get_public_property("customItems", activation)
        {
            // note: this borrows the array, but it shouldn't be possible for
            // AS to get invoked here and cause BorrowMutError
            if let Some(array) = custom_items.as_array_storage() {
                for (i, item) in array.iter().enumerate() {
                    // TODO: Non-CustomMenuItem Object-s shouldn't count

                    if let Some(Value::Object(item)) = item {
                        let caption = if let Ok(Value::String(s)) =
                            item.get_public_property("caption", activation)
                        {
                            s
                        } else {
                            continue;
                        };
                        let enabled = check_bool!(item, "enabled", true);
                        let visible = check_bool!(item, "visible", true);
                        let separator_before = check_bool!(item, "separatorBefore", true);

                        if !visible {
                            continue;
                        }

                        result.push(
                            context_menu::ContextMenuItem {
                                enabled,
                                separator_before: separator_before || i == 0,
                                caption: caption.to_string(),
                                checked: false,
                            },
                            context_menu::ContextMenuCallback::Avm2 { item },
                        );
                    }
                }
            }
        }
    }
    result
}
