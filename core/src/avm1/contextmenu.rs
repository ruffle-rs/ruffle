//! Structure holding the temporary state of an open context menu.
//!
//! The context menu items and callbacks set to `object.menu`
//! are stored aside when the menu is open. This way the context menu
//! items work even if the movie changed `object.menu` in the meantime.
//!
//! TODO: Refactor for AVM2?

use crate::avm1::{Activation, ActivationIdentifier, AvmString, Object, TObject, Value};
use crate::context::UpdateContext;
use crate::display_object::TDisplayObject;
use gc_arena::Collect;
use serde::Serialize;

#[derive(Collect)]
#[collect(no_drop)]
pub struct ContextMenuState<'gc> {
    is_playing_root_movie: bool,
    builtin_items: Vec<&'static str>,
    custom_items: Vec<ContextMenuItemState<'gc>>,
}

#[derive(Collect)]
#[collect(no_drop)]
pub struct ContextMenuItemState<'gc> {
    visible: bool,
    enabled: bool,
    separator_before: bool,
    caption: AvmString<'gc>,
    item: Object<'gc>,
    callback: Object<'gc>,
}

#[derive(Serialize)]
pub struct ContextMenuInfo {
    #[serde(rename = "playing")]
    pub is_playing_root_movie: bool,
    #[serde(rename = "builtinItems")]
    pub builtin_items: Vec<&'static str>,
    #[serde(rename = "customItems")]
    pub custom_items: Vec<ContextMenuItemInfo>,
}

impl ContextMenuInfo {
    pub fn empty() -> ContextMenuInfo {
        ContextMenuInfo {
            is_playing_root_movie: false, // doesn't matter
            builtin_items: vec![],
            custom_items: vec![],
        }
    }
}

#[derive(Serialize)]
pub struct ContextMenuItemInfo {
    pub visible: bool,
    pub enabled: bool,
    pub separator_before: bool,
    pub caption: String,
}

impl<'gc> ContextMenuState<'gc> {
    pub fn new(menu: Option<Object<'gc>>, activation: &mut Activation<'_, 'gc, '_>) -> Self {
        let is_playing_root_movie =
            if let Some(mc) = activation.context.stage.root_clip().as_movie_clip() {
                mc.playing()
            } else {
                false
            };

        let builtin_items = {
            let is_multiframe_movie = activation.context.swf.header().num_frames > 1;
            let mut names = if is_multiframe_movie {
                vec![
                    "zoom",
                    "quality",
                    "play",
                    "loop",
                    "rewind",
                    "forward_back",
                    "print",
                ]
            } else {
                vec!["zoom", "quality", "print"]
            };
            if let Some(menu) = menu {
                if let Ok(Value::Object(builtins)) = menu.get("builtInItems", activation) {
                    names.retain(|name| {
                        !matches!(builtins.get(name, activation), Ok(Value::Bool(false)))
                    });
                }
            }
            names
        };

        let custom_items = {
            let mut items = vec![];
            if let Some(menu) = menu {
                if let Ok(Value::Object(custom_items)) = menu.get("customItems", activation) {
                    for item in custom_items.array() {
                        if let Value::Object(item) = item {
                            let caption = if let Ok(Value::String(caption)) =
                                item.get("caption", activation)
                            {
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

                            if !visible {
                                continue;
                            }

                            items.push(ContextMenuItemState {
                                visible,
                                enabled,
                                separator_before: false,
                                caption: caption,
                                item: item,
                                callback: on_select,
                            });
                        }
                    }
                }
            }
            items
        };

        Self {
            is_playing_root_movie,
            builtin_items,
            custom_items,
        }
    }

    pub fn get_info(&self) -> ContextMenuInfo {
        let custom_items = self
            .custom_items
            .iter()
            .map(|item| ContextMenuItemInfo {
                visible: item.visible,
                enabled: item.enabled,
                separator_before: item.separator_before,
                caption: item.caption.to_string(),
            })
            .collect();

        ContextMenuInfo {
            builtin_items: self.builtin_items.clone(),
            custom_items,
            is_playing_root_movie: self.is_playing_root_movie,
        }
    }

    pub fn run_callback(&self, index: usize, context: &mut UpdateContext<'_, 'gc, '_>) {
        let version = context.swf.header().version;
        let globals = context.avm1.global_object_cell();
        let root_clip = context.stage.root_clip();

        let mut activation = Activation::from_nothing(
            context.reborrow(),
            ActivationIdentifier::root("[Context Menu Callback]"),
            version,
            globals,
            root_clip,
        );

        let item = &self.custom_items[index];

        // TODO: `this` is undefined, but our VM
        // currently doesn't allow `this` to be a Value (#843).
        let undefined = Value::Undefined.coerce_to_object(&mut activation);

        // TODO: remember to also change the first arg
        // we support contextmenu on non-root-movie
        let params = vec![root_clip.object(), Value::Object(item.item)];

        let _ = item.callback.call(
            "[Context Menu Callback]",
            &mut activation,
            undefined,
            None,
            &params,
        );

        crate::player::Player::run_actions(&mut activation.context);
    }
}
