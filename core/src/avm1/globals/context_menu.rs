use crate::avm1::activation::Activation;
use crate::avm1::{Object, Value};
use crate::context_menu;
use crate::display_object::DisplayObject;
use ruffle_macros::istr;

pub fn make_context_menu_state<'gc>(
    menu: Option<Object<'gc>>,
    object: Option<DisplayObject<'gc>>,
    activation: &mut Activation<'_, 'gc>,
) -> context_menu::ContextMenuState<'gc> {
    let mut result = context_menu::ContextMenuState::new();

    result.set_display_object(object);

    let mut builtin_items = context_menu::BuiltInItemFlags::for_stage(activation.context.stage);
    if let Some(menu) = menu
        && let Ok(Value::Object(builtins)) = menu.get(istr!("builtInItems"), activation)
    {
        if matches!(
            builtins.get(istr!("zoom"), activation),
            Ok(Value::Bool(false))
        ) {
            builtin_items.zoom = false;
        }
        if matches!(
            builtins.get(istr!("quality"), activation),
            Ok(Value::Bool(false))
        ) {
            builtin_items.quality = false;
        }
        if matches!(
            builtins.get(istr!("play"), activation),
            Ok(Value::Bool(false))
        ) {
            builtin_items.play = false;
        }
        if matches!(
            builtins.get(istr!("loop"), activation),
            Ok(Value::Bool(false))
        ) {
            builtin_items.loop_ = false;
        }
        if matches!(
            builtins.get(istr!("rewind"), activation),
            Ok(Value::Bool(false))
        ) {
            builtin_items.rewind = false;
        }
        if matches!(
            builtins.get(istr!("forward_back"), activation),
            Ok(Value::Bool(false))
        ) {
            builtin_items.forward_and_back = false;
        }
        if matches!(
            builtins.get(istr!("print"), activation),
            Ok(Value::Bool(false))
        ) {
            builtin_items.print = false;
        }
    }

    result.build_builtin_items(builtin_items, activation.context);

    if let Some(menu) = menu
        && let Ok(Value::Object(custom_items)) = menu.get(istr!("customItems"), activation)
        && let Ok(length) = custom_items.length(activation)
    {
        for i in 0..length {
            let item = custom_items.get_element(activation, i);
            if let Value::Object(item) = item {
                let caption =
                    if let Ok(Value::String(caption)) = item.get(istr!("caption"), activation) {
                        caption
                    } else {
                        continue;
                    };
                let on_select =
                    if let Ok(Value::Object(on_select)) = item.get(istr!("onSelect"), activation) {
                        on_select
                    } else {
                        continue;
                    };
                // false if `false`, everything else is true
                let visible = !matches!(
                    item.get(istr!("visible"), activation),
                    Ok(Value::Bool(false))
                );
                // true if `true`, everything else is false
                let enabled = matches!(
                    item.get(istr!("enabled"), activation),
                    Ok(Value::Bool(true))
                );
                let separator_before = matches!(
                    item.get(istr!("separatorBefore"), activation),
                    Ok(Value::Bool(true))
                );

                if !visible {
                    continue;
                }

                if result
                    .info()
                    .iter()
                    .any(|menu_item| menu_item.caption == caption.to_string())
                {
                    continue;
                }

                result.push(
                    context_menu::ContextMenuItem {
                        enabled,
                        separator_before: separator_before || i == 0,
                        caption: caption.to_string(),
                        checked: false,
                    },
                    context_menu::ContextMenuCallback::Avm1 {
                        item,
                        callback: on_select,
                    },
                );
            }
        }
    }

    result
}
