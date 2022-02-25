//! Structure holding the temporary state of an open context menu.
//!
//! The context menu items and callbacks set to `object.menu`
//! are stored aside when the menu is open. This way the context menu
//! items work even if the movie changed `object.menu` in the meantime.

use crate::avm1;
use gc_arena::Collect;
use serde::Serialize;

#[derive(Collect, Default)]
#[collect(no_drop)]
pub struct ContextMenuState<'gc> {
    info: Vec<ContextMenuItem>,
    callbacks: Vec<ContextMenuCallback<'gc>>,
}

impl<'gc> ContextMenuState<'gc> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn push(&mut self, item: ContextMenuItem, callback: ContextMenuCallback<'gc>) {
        self.info.push(item);
        self.callbacks.push(callback);
    }
    pub fn info(&self) -> &Vec<ContextMenuItem> {
        &self.info
    }
    pub fn callback(&self, index: usize) -> &ContextMenuCallback<'gc> {
        &self.callbacks[index]
    }
}

#[derive(Collect, Clone, Serialize)]
#[collect(require_static)]
pub struct ContextMenuItem {
    pub enabled: bool,
    #[serde(rename = "separatorBefore")]
    pub separator_before: bool,
    pub checked: bool,
    pub caption: String,
}

#[derive(Collect)]
#[collect(no_drop)]
pub enum ContextMenuCallback<'gc> {
    Zoom,
    Quality,
    Play,
    Loop,
    Rewind,
    Forward,
    Back,
    Print,
    Avm1 {
        item: avm1::Object<'gc>,
        callback: avm1::Object<'gc>,
    },
}
