//! Structure holding the temporary state of an open context menu.
//!
//! The context menu items and callbacks set to `object.menu`
//! are stored aside when the menu is open. This way the context menu
//! items work even if the movie changed `object.menu` in the meantime.

use crate::avm1;
use crate::avm2;
use crate::context::UpdateContext;
use crate::display_object::{EditText, Stage};
use crate::display_object::{InteractiveObject, TDisplayObject};
use crate::events::TextControlCode;
use crate::i18n::core_text;
use gc_arena::Collect;
use ruffle_render::quality::StageQuality;

#[derive(Collect, Default)]
#[collect(no_drop)]
pub struct ContextMenuState<'gc> {
    #[collect(require_static)]
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

    pub fn build_builtin_items(
        &mut self,
        item_flags: BuiltInItemFlags,
        context: &mut UpdateContext<'_, 'gc>,
    ) {
        let stage = context.stage;
        let language = &context.ui.language();

        // When a text field is focused and the mouse is hovering it,
        // show the copy/paste menu.
        if let Some(text) = context.focus_tracker.get_as_edit_text() {
            if InteractiveObject::option_ptr_eq(context.mouse_data.hovered, text.as_interactive()) {
                self.build_text_items(text, context);
                return;
            }
        }

        let Some(root_mc) = stage.root_clip().and_then(|c| c.as_movie_clip()) else {
            return;
        };
        if item_flags.play {
            let is_playing_root_movie = root_mc.playing();
            self.push(
                ContextMenuItem {
                    enabled: true,
                    separator_before: true,
                    caption: core_text(language, "context-menu-play"),
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
                    caption: core_text(language, "context-menu-rewind"),
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
                    caption: core_text(language, "context-menu-forward"),
                    checked: false,
                },
                ContextMenuCallback::Forward,
            );
            self.push(
                ContextMenuItem {
                    enabled: !is_first_frame,
                    separator_before: false,
                    caption: core_text(language, "context-menu-back"),
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
                    caption: core_text(language, "context-menu-quality-low"),
                },
                ContextMenuCallback::QualityLow,
            );
            self.push(
                ContextMenuItem {
                    enabled: stage.quality() != StageQuality::Medium,
                    separator_before: false,
                    checked: stage.quality() == StageQuality::Medium,
                    caption: core_text(language, "context-menu-quality-medium"),
                },
                ContextMenuCallback::QualityMedium,
            );
            self.push(
                ContextMenuItem {
                    enabled: stage.quality() != StageQuality::High,
                    separator_before: false,
                    checked: stage.quality() == StageQuality::High,
                    caption: core_text(language, "context-menu-quality-high"),
                },
                ContextMenuCallback::QualityHigh,
            );
        }
    }

    fn build_text_items(&mut self, text: EditText<'gc>, context: &mut UpdateContext<'_, 'gc>) {
        let language = &context.ui.language();
        self.push(
            ContextMenuItem {
                enabled: text.is_text_control_applicable(TextControlCode::Cut, context),
                separator_before: true,
                caption: core_text(language, "context-menu-cut"),
                checked: false,
            },
            ContextMenuCallback::TextControl {
                code: TextControlCode::Cut,
                text,
            },
        );
        self.push(
            ContextMenuItem {
                enabled: text.is_text_control_applicable(TextControlCode::Copy, context),
                separator_before: false,
                caption: core_text(language, "context-menu-copy"),
                checked: false,
            },
            ContextMenuCallback::TextControl {
                code: TextControlCode::Copy,
                text,
            },
        );
        self.push(
            ContextMenuItem {
                enabled: text.is_text_control_applicable(TextControlCode::Paste, context),
                separator_before: false,
                caption: core_text(language, "context-menu-paste"),
                checked: false,
            },
            ContextMenuCallback::TextControl {
                code: TextControlCode::Paste,
                text,
            },
        );
        self.push(
            ContextMenuItem {
                enabled: text.is_text_control_applicable(TextControlCode::Delete, context),
                separator_before: false,
                caption: core_text(language, "context-menu-delete"),
                checked: false,
            },
            ContextMenuCallback::TextControl {
                code: TextControlCode::Delete,
                text,
            },
        );
        self.push(
            ContextMenuItem {
                enabled: text.is_text_control_applicable(TextControlCode::SelectAll, context),
                separator_before: true,
                caption: core_text(language, "context-menu-select-all"),
                checked: false,
            },
            ContextMenuCallback::TextControl {
                code: TextControlCode::SelectAll,
                text,
            },
        );
    }
}

#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ContextMenuItem {
    pub enabled: bool,
    #[cfg_attr(feature = "serde", serde(rename = "separatorBefore"))]
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
    TextControl {
        #[collect(require_static)]
        code: TextControlCode,
        text: EditText<'gc>,
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
