//! Structure holding the temporary state of an open context menu.
//!
//! The context menu items and callbacks set to `object.menu`
//! are stored aside when the menu is open. This way the context menu
//! items work even if the movie changed `object.menu` in the meantime.

use crate::avm1::{AvmString, Object};
use gc_arena::Collect;
use serde::Serialize;

#[derive(Collect)]
#[collect(no_drop)]
pub struct ContextMenuState<'gc> {
    pub is_playing_root_movie: bool,
    pub builtin_items: Vec<&'static str>,
    pub custom_items: Vec<ContextMenuItemState<'gc>>,
}

#[derive(Collect)]
#[collect(no_drop)]
pub struct ContextMenuItemState<'gc> {
    pub enabled: bool,
    pub separator_before: bool,
    pub caption: AvmString<'gc>,
    pub item: Object<'gc>,
    pub callback: Object<'gc>,
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
    pub enabled: bool,
    #[serde(rename = "separatorBefore")]
    pub separator_before: bool,
    pub caption: String,
}

impl<'gc> ContextMenuState<'gc> {
    pub fn get_info(&self) -> ContextMenuInfo {
        let custom_items = self
            .custom_items
            .iter()
            .map(|item| ContextMenuItemInfo {
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
}
