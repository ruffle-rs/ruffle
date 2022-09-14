use crate::avm2::activation::Activation;
use crate::avm2::multiname::Multiname;
use crate::avm2::object::{Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::context_menu;

pub fn hide_built_in_items<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        if let Value::Object(mut items) =
            this.get_property(&Multiname::public("builtInItems"), activation)?
        {
            // items is a ContextMenuBuiltInItems
            items.set_property(
                &Multiname::public("forwardAndBack"),
                Value::Bool(false),
                activation,
            )?;
            items.set_property(&Multiname::public("loop"), Value::Bool(false), activation)?;
            items.set_property(&Multiname::public("play"), Value::Bool(false), activation)?;
            items.set_property(&Multiname::public("print"), Value::Bool(false), activation)?;
            items.set_property(
                &Multiname::public("quality"),
                Value::Bool(false),
                activation,
            )?;
            items.set_property(&Multiname::public("rewind"), Value::Bool(false), activation)?;
            items.set_property(&Multiname::public("save"), Value::Bool(false), activation)?;
            items.set_property(&Multiname::public("zoom"), Value::Bool(false), activation)?;
        }
    }

    Ok(Value::Undefined)
}

pub fn make_context_menu_state<'gc>(
    menu: Option<Object<'gc>>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> context_menu::ContextMenuState<'gc> {
    let mut result = context_menu::ContextMenuState::new();

    let mut builtin_items = context_menu::BuiltInItemFlags::for_stage(activation.context.stage);
    if let Some(menu) = menu {
        if let Ok(Value::Object(builtins)) =
            menu.get_property(&Multiname::public("builtInItems"), activation)
        {
            if matches!(
                builtins.get_property(&Multiname::public("zoom"), activation),
                Ok(Value::Bool(false))
            ) {
                builtin_items.zoom = false;
            }
            if matches!(
                builtins.get_property(&Multiname::public("quality"), activation),
                Ok(Value::Bool(false))
            ) {
                builtin_items.quality = false;
            }
            if matches!(
                builtins.get_property(&Multiname::public("play"), activation),
                Ok(Value::Bool(false))
            ) {
                builtin_items.play = false;
            }
            if matches!(
                builtins.get_property(&Multiname::public("loop"), activation),
                Ok(Value::Bool(false))
            ) {
                builtin_items.loop_ = false;
            }
            if matches!(
                builtins.get_property(&Multiname::public("rewind"), activation),
                Ok(Value::Bool(false))
            ) {
                builtin_items.rewind = false;
            }
            if matches!(
                builtins.get_property(&Multiname::public("forwardAndBack"), activation),
                Ok(Value::Bool(false))
            ) {
                builtin_items.forward_and_back = false;
            }
            if matches!(
                builtins.get_property(&Multiname::public("print"), activation),
                Ok(Value::Bool(false))
            ) {
                builtin_items.print = false;
            }
        }
    }

    result.build_builtin_items(builtin_items, activation.context.stage);

    if let Some(menu) = menu {
        if let Ok(Value::Object(custom_items)) =
            menu.get_property(&Multiname::public("customItems"), activation)
        {
            // note: this borrows the array, but it shouldn't be possible for
            // AS to get invoked here and cause BorrowMutError
            if let Some(array) = custom_items.as_array_storage() {
                for (i, item) in array.iter().enumerate() {
                    // this is a CustomMenuItem
                    if let Some(Value::Object(item)) = item {
                        let caption = if let Ok(Value::String(s)) =
                            item.get_property(&Multiname::public("caption"), activation)
                        {
                            s
                        } else {
                            // It's a CustomMenuItem, so this shouldn't happen
                            continue;
                        };
                        let enabled = matches!(
                            item.get_property(&Multiname::public("enabled"), activation),
                            Ok(Value::Bool(true))
                        );
                        let visible = matches!(
                            item.get_property(&Multiname::public("visible"), activation),
                            Ok(Value::Bool(true))
                        );
                        let separator_before = matches!(
                            item.get_property(&Multiname::public("separatorBefore"), activation),
                            Ok(Value::Bool(true))
                        );

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
