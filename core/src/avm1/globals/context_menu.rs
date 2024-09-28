use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::object::TObject;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::Object;
use crate::avm1::{ScriptObject, Value};
use crate::context_menu;
use crate::display_object::DisplayObject;
use crate::string::StringContext;

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "copy" => method(copy; DONT_ENUM | DONT_DELETE);
    "hideBuiltInItems" => method(hide_builtin_items; DONT_ENUM | DONT_DELETE);
};

pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let callback = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_object(activation);

    this.set("onSelect", callback.into(), activation)?;

    let built_in_items = ScriptObject::new(
        activation.context.gc_context,
        Some(activation.context.avm1.prototypes().object),
    );

    built_in_items.set("print", true.into(), activation)?;
    built_in_items.set("forward_back", true.into(), activation)?;
    built_in_items.set("rewind", true.into(), activation)?;
    built_in_items.set("loop", true.into(), activation)?;
    built_in_items.set("play", true.into(), activation)?;
    built_in_items.set("quality", true.into(), activation)?;
    built_in_items.set("zoom", true.into(), activation)?;
    built_in_items.set("save", true.into(), activation)?;

    this.set("builtInItems", built_in_items.into(), activation)?;

    let constructor = activation.context.avm1.prototypes().array_constructor;
    let custom_items = constructor.construct(activation, &[])?;

    this.set("customItems", custom_items, activation)?;

    Ok(this.into())
}

pub fn copy<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let callback = this
        .get("onSelect", activation)?
        .coerce_to_object(activation);

    let constructor = activation
        .context
        .avm1
        .prototypes()
        .context_menu_constructor;
    let copy = constructor
        .construct(activation, &[callback.into()])?
        .coerce_to_object(activation);

    let built_in = this
        .get("builtInItems", activation)?
        .coerce_to_object(activation);
    let copy_built_in = copy
        .get("builtInItems", activation)?
        .coerce_to_object(activation);

    let save = built_in
        .get("save", activation)?
        .as_bool(activation.swf_version());
    let zoom = built_in
        .get("zoom", activation)?
        .as_bool(activation.swf_version());
    let quality = built_in
        .get("quality", activation)?
        .as_bool(activation.swf_version());
    let play = built_in
        .get("play", activation)?
        .as_bool(activation.swf_version());
    let loop_ = built_in
        .get("loop", activation)?
        .as_bool(activation.swf_version());
    let rewind = built_in
        .get("rewind", activation)?
        .as_bool(activation.swf_version());
    let forward_back = built_in
        .get("forward_back", activation)?
        .as_bool(activation.swf_version());
    let print = built_in
        .get("print", activation)?
        .as_bool(activation.swf_version());

    copy_built_in.set("save", save.into(), activation)?;
    copy_built_in.set("zoom", zoom.into(), activation)?;
    copy_built_in.set("quality", quality.into(), activation)?;
    copy_built_in.set("play", play.into(), activation)?;
    copy_built_in.set("loop", loop_.into(), activation)?;
    copy_built_in.set("rewind", rewind.into(), activation)?;
    copy_built_in.set("forward_back", forward_back.into(), activation)?;
    copy_built_in.set("print", print.into(), activation)?;

    let custom_items = this
        .get("customItems", activation)?
        .coerce_to_object(activation);
    let custom_items_copy = copy
        .get("customItems", activation)?
        .coerce_to_object(activation);

    for i in 0..custom_items.length(activation)? {
        let element = custom_items.get_element(activation, i);
        custom_items_copy.set_element(activation, i, element)?;
    }

    Ok(copy.into())
}

pub fn hide_builtin_items<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let built_in_items = this
        .get("builtInItems", activation)?
        .coerce_to_object(activation);
    built_in_items.set("zoom", false.into(), activation)?;
    built_in_items.set("quality", false.into(), activation)?;
    built_in_items.set("play", false.into(), activation)?;
    built_in_items.set("loop", false.into(), activation)?;
    built_in_items.set("rewind", false.into(), activation)?;
    built_in_items.set("forward_back", false.into(), activation)?;
    built_in_items.set("print", false.into(), activation)?;
    Ok(Value::Undefined)
}

pub fn create_proto<'gc>(
    context: &mut StringContext<'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let object = ScriptObject::new(context.gc_context, Some(proto));
    define_properties_on(PROTO_DECLS, context, object, fn_proto);
    object.into()
}

pub fn make_context_menu_state<'gc>(
    menu: Option<Object<'gc>>,
    object: Option<DisplayObject<'gc>>,
    activation: &mut Activation<'_, 'gc>,
) -> context_menu::ContextMenuState<'gc> {
    let mut result = context_menu::ContextMenuState::new();

    result.set_display_object(object);

    let mut builtin_items = context_menu::BuiltInItemFlags::for_stage(activation.context.stage);
    if let Some(menu) = menu {
        if let Ok(Value::Object(builtins)) = menu.get("builtInItems", activation) {
            if matches!(builtins.get("zoom", activation), Ok(Value::Bool(false))) {
                builtin_items.zoom = false;
            }
            if matches!(builtins.get("quality", activation), Ok(Value::Bool(false))) {
                builtin_items.quality = false;
            }
            if matches!(builtins.get("play", activation), Ok(Value::Bool(false))) {
                builtin_items.play = false;
            }
            if matches!(builtins.get("loop", activation), Ok(Value::Bool(false))) {
                builtin_items.loop_ = false;
            }
            if matches!(builtins.get("rewind", activation), Ok(Value::Bool(false))) {
                builtin_items.rewind = false;
            }
            if matches!(
                builtins.get("forward_back", activation),
                Ok(Value::Bool(false))
            ) {
                builtin_items.forward_and_back = false;
            }
            if matches!(builtins.get("print", activation), Ok(Value::Bool(false))) {
                builtin_items.print = false;
            }
        }
    }

    result.build_builtin_items(builtin_items, activation.context);

    if let Some(menu) = menu {
        if let Ok(Value::Object(custom_items)) = menu.get("customItems", activation) {
            if let Ok(length) = custom_items.length(activation) {
                for i in 0..length {
                    let item = custom_items.get_element(activation, i);
                    if let Value::Object(item) = item {
                        let caption =
                            if let Ok(Value::String(caption)) = item.get("caption", activation) {
                                caption
                            } else {
                                continue;
                            };
                        let on_select = if let Ok(Value::Object(on_select)) =
                            item.get("onSelect", activation)
                        {
                            on_select
                        } else {
                            continue;
                        };
                        // false if `false`, everything else is true
                        let visible =
                            !matches!(item.get("visible", activation), Ok(Value::Bool(false)));
                        // true if `true`, everything else is false
                        let enabled =
                            matches!(item.get("enabled", activation), Ok(Value::Bool(true)));
                        let separator_before = matches!(
                            item.get("separatorBefore", activation),
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
        }
    }

    result
}
