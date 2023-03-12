//! Structure holding the temporary state of an open context menu.
//!
//! The context menu items and callbacks set to `object.menu`
//! are stored aside when the menu is open. This way the context menu
//! items work even if the movie changed `object.menu` in the meantime.

use crate::avm1;
use crate::avm2;
use crate::display_object::Stage;
use crate::display_object::TDisplayObject;
use gc_arena::Collect;
use ruffle_render::quality::StageQuality;
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
    pub fn build_builtin_items(&mut self, item_flags: BuiltInItemFlags, stage: Stage<'gc>) {
        let root_mc = if let Some(root_clip) = stage.root_clip().and_then(|c| c.as_movie_clip()) {
            root_clip
        } else {
            return;
        };
        if item_flags.play {
            let is_playing_root_movie = root_mc.playing();
            self.push(
                ContextMenuItem {
                    enabled: true,
                    separator_before: true,
                    caption: "Play".to_string(),
                    checked: is_playing_root_movie,
                },
                ContextMenuCallback::Play,
            );
        }
        if item_flags.rewind {
            let is_first_frame = root_mc.current_frame() <= 1;
            self.push(
                ContextMenuItem {
                    enabled: !is_first_frame,
                    separator_before: true,
                    caption: "Rewind".to_string(),
                    checked: false,
                },
                ContextMenuCallback::Rewind,
            );
        }
        if item_flags.forward_and_back {
            let is_first_frame = root_mc.current_frame() <= 1;
            self.push(
                ContextMenuItem {
                    enabled: true,
                    separator_before: false,
                    caption: "Forward".to_string(),
                    checked: false,
                },
                ContextMenuCallback::Forward,
            );
            self.push(
                ContextMenuItem {
                    enabled: !is_first_frame,
                    separator_before: false,
                    caption: "Back".to_string(),
                    checked: false,
                },
                ContextMenuCallback::Back,
            );
        }
        if item_flags.quality {
            // TODO: This should be a submenu, but at time of writing those aren't supported
            self.push(
                ContextMenuItem {
                    enabled: stage.quality() != StageQuality::Low,
                    separator_before: true,
                    checked: stage.quality() == StageQuality::Low,
                    caption: "Quality: Low".to_string(),
                },
                ContextMenuCallback::QualityLow,
            );
            self.push(
                ContextMenuItem {
                    enabled: stage.quality() != StageQuality::Medium,
                    separator_before: false,
                    checked: stage.quality() == StageQuality::Medium,
                    caption: "Quality: Medium".to_string(),
                },
                ContextMenuCallback::QualityMedium,
            );
            self.push(
                ContextMenuItem {
                    enabled: stage.quality() != StageQuality::High,
                    separator_before: false,
                    checked: stage.quality() == StageQuality::High,
                    caption: "Quality: High".to_string(),
                },
                ContextMenuCallback::QualityHigh,
            );
        }
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
    QualityLow,
    QualityMedium,
    QualityHigh,
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
    Avm2 {
        item: avm2::Object<'gc>,
    },
}

pub struct BuiltInItemFlags {
    pub forward_and_back: bool,
    pub loop_: bool,
    pub play: bool,
    pub print: bool,
    pub quality: bool,
    pub rewind: bool,
    pub save: bool,
    pub zoom: bool,
}

impl BuiltInItemFlags {
    pub fn for_stage(stage: Stage<'_>) -> Self {
        let root_mc = stage.root_clip().and_then(|c| c.as_movie_clip());
        let is_multiframe_movie = root_mc.map(|mc| mc.total_frames() > 1).unwrap_or(false);
        if is_multiframe_movie {
            Self {
                forward_and_back: true,
                loop_: true,
                play: true,
                print: true,
                quality: true,
                rewind: true,
                zoom: true,

                save: false,
            }
        } else {
            Self {
                print: true,
                quality: true,
                zoom: true,

                forward_and_back: false,
                rewind: false,
                loop_: false,
                play: false,
                save: false,
            }
        }
    }
}
