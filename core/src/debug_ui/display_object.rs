use crate::context::UpdateContext;
use crate::debug_ui::handle::DisplayObjectHandle;
use crate::debug_ui::Message;
use crate::display_object::{DisplayObject, TDisplayObject, TDisplayObjectContainer};
use egui::{CollapsingHeader, ComboBox, Grid, Id, Ui, Window};
use std::borrow::Cow;
use swf::BlendMode;

const ALL_BLEND_MODES: [BlendMode; 14] = [
    BlendMode::Normal,
    BlendMode::Layer,
    BlendMode::Multiply,
    BlendMode::Screen,
    BlendMode::Lighten,
    BlendMode::Darken,
    BlendMode::Difference,
    BlendMode::Add,
    BlendMode::Subtract,
    BlendMode::Invert,
    BlendMode::Alpha,
    BlendMode::Erase,
    BlendMode::Overlay,
    BlendMode::HardLight,
];

#[derive(Debug, Eq, PartialEq, Hash, Default, Copy, Clone)]
enum Panel {
    #[default]
    Position,
    Display,
    Children,
}

#[derive(Debug, Default)]
pub struct DisplayObjectWindow {
    open_panel: Panel,
}

impl DisplayObjectWindow {
    pub fn show<'gc>(
        &mut self,
        egui_ctx: &egui::Context,
        context: &mut UpdateContext<'_, 'gc>,
        object: DisplayObject<'gc>,
        messages: &mut Vec<Message>,
    ) -> bool {
        let mut keep_open = true;
        Window::new(summary_name(object))
            .id(Id::new(object.as_ptr()))
            .open(&mut keep_open)
            .show(egui_ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.open_panel, Panel::Position, "Position");
                    ui.selectable_value(&mut self.open_panel, Panel::Display, "Display");
                    if let Some(ctr) = object.as_container() {
                        if !ctr.is_empty() {
                            ui.selectable_value(
                                &mut self.open_panel,
                                Panel::Children,
                                format!("Children ({})", ctr.num_children()),
                            );
                        }
                    }
                });
                ui.separator();

                match self.open_panel {
                    Panel::Position => self.show_position(ui, object),
                    Panel::Display => self.show_display(ui, context, object, messages),
                    Panel::Children => self.show_children(ui, context, object, messages),
                }
            });
        keep_open
    }

    pub fn show_display<'gc>(
        &mut self,
        ui: &mut Ui,
        context: &mut UpdateContext<'_, 'gc>,
        object: DisplayObject<'gc>,
        messages: &mut Vec<Message>,
    ) {
        Grid::new(ui.id().with("display"))
            .num_columns(2)
            .show(ui, |ui| {
                if let Some(parent) = object.parent() {
                    ui.label("Parent");
                    if ui.button(summary_name(parent)).clicked() {
                        messages.push(Message::TrackDisplayObject(DisplayObjectHandle::new(
                            context, parent,
                        )));
                    }
                    ui.end_row();
                }

                let was_visible = object.visible();
                let mut is_visible = was_visible;
                ui.label("Visibility");
                ui.checkbox(&mut is_visible, "Visible");
                ui.end_row();
                if is_visible != was_visible {
                    object.set_visible(context.gc_context, is_visible);
                }

                ui.label("Blend mode");
                let old_blend = object.blend_mode();
                let mut new_blend = old_blend;
                ComboBox::from_id_source(ui.id().with("blendmode"))
                    .selected_text(blend_mode_name(old_blend))
                    .show_ui(ui, |ui| {
                        for mode in ALL_BLEND_MODES {
                            ui.selectable_value(&mut new_blend, mode, blend_mode_name(mode));
                        }
                    });
                ui.end_row();
                if new_blend != old_blend {
                    object.set_blend_mode(context.gc_context, new_blend);
                }
            });
    }

    pub fn show_position(&mut self, ui: &mut Ui, object: DisplayObject<'_>) {
        Grid::new(ui.id().with("position"))
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("Name");
                // &mut of a temporary thing because we don't want to actually be able to change this
                // If we disable it, the user can't highlight or interact with it, so this makes it readonly but enabled
                ui.text_edit_singleline(&mut object.name().to_string());
                ui.end_row();

                ui.label("World Bounds");
                ui.label(object.world_bounds().to_string());
                ui.end_row();

                ui.label("Local Bounds");
                ui.label(object.local_bounds().to_string());
                ui.end_row();

                let base = object.base();
                let matrix = base.matrix();
                ui.label("Local Position");
                ui.label(format!("{}, {}", matrix.tx, matrix.ty));
                ui.end_row();

                ui.label("Local Rotation");
                ui.label(format!("{}, {}", matrix.b, matrix.c));
                ui.end_row();

                ui.label("Local Scale");
                ui.label(format!("{}, {}", matrix.a, matrix.d));
                ui.end_row();
            });
    }

    pub fn show_children<'gc>(
        &mut self,
        ui: &mut Ui,
        context: &mut UpdateContext<'_, 'gc>,
        object: DisplayObject<'gc>,
        messages: &mut Vec<Message>,
    ) {
        if let Some(ctr) = object.as_container() {
            for child in ctr.iter_render_list() {
                show_display_tree(ui, context, child, messages);
            }
        }
    }
}

pub fn show_display_tree<'gc>(
    ui: &mut Ui,
    context: &mut UpdateContext<'_, 'gc>,
    object: DisplayObject<'gc>,
    messages: &mut Vec<Message>,
) {
    CollapsingHeader::new(summary_name(object))
        .id_source(ui.id().with(object.as_ptr()))
        .show(ui, |ui| {
            if ui.button("Track").clicked() {
                messages.push(Message::TrackDisplayObject(DisplayObjectHandle::new(
                    context, object,
                )));
            }
            if let Some(ctr) = object.as_container() {
                for child in ctr.iter_render_list() {
                    show_display_tree(ui, context, child, messages);
                }
            }
        });
}

fn summary_name(object: DisplayObject) -> Cow<str> {
    let do_type = match object {
        DisplayObject::Stage(_) => "Stage",
        DisplayObject::Bitmap(_) => "Bitmap",
        DisplayObject::Avm1Button(_) => "Avm1Button",
        DisplayObject::Avm2Button(_) => "Avm2Button",
        DisplayObject::EditText(_) => "EditText",
        DisplayObject::Graphic(_) => "Graphic",
        DisplayObject::MorphShape(_) => "MorphShape",
        DisplayObject::MovieClip(_) => "MovieClip",
        DisplayObject::Text(_) => "Text",
        DisplayObject::Video(_) => "Video",
        DisplayObject::LoaderDisplay(_) => "LoaderDisplay",
    };

    let name = object.name();
    if name.is_empty() {
        Cow::Borrowed(do_type)
    } else {
        Cow::Owned(format!("{do_type} \"{name}\""))
    }
}

fn blend_mode_name(mode: BlendMode) -> &'static str {
    match mode {
        BlendMode::Normal => "Normal",
        BlendMode::Layer => "Layer",
        BlendMode::Multiply => "Multiply",
        BlendMode::Screen => "Screen",
        BlendMode::Lighten => "Lighten",
        BlendMode::Darken => "Darken",
        BlendMode::Difference => "Difference",
        BlendMode::Add => "Add",
        BlendMode::Subtract => "Subtract",
        BlendMode::Invert => "Invert",
        BlendMode::Alpha => "Alpha",
        BlendMode::Erase => "Erase",
        BlendMode::Overlay => "Overlay",
        BlendMode::HardLight => "HardLight",
    }
}
