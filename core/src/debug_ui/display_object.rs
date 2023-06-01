use crate::context::UpdateContext;
use crate::debug_ui::handle::DisplayObjectHandle;
use crate::debug_ui::Message;
use crate::display_object::{DisplayObject, TDisplayObject, TDisplayObjectContainer};
use egui::{Checkbox, CollapsingHeader, ComboBox, Grid, Id, Ui, Widget, Window};
use std::borrow::Cow;
use swf::{BlendMode, ColorTransform, Fixed8};

const DEFAULT_DEBUG_COLORS: [[f32; 3]; 10] = [
    [0.00, 0.39, 0.00], // "darkgreen" / #006400
    [0.00, 0.00, 0.55], // "darkblue" / #00008b
    [0.69, 0.19, 0.38], // "maroon3" / #b03060
    [1.00, 0.27, 0.00], // "orangered" / #ff4500
    [1.00, 1.00, 0.00], // "yellow" / #ffff00
    [0.00, 1.00, 0.00], // "lime" / #00ff00
    [0.00, 1.00, 1.00], // "aqua" / #00ffff
    [1.00, 0.00, 1.00], // "fuchsia" / #ff00ff
    [0.39, 0.58, 0.93], // "cornflower" / #6495ed
    [1.00, 0.87, 0.68], // "navajowhite" / #ffdead
];

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

#[derive(Debug)]
pub struct DisplayObjectWindow {
    open_panel: Panel,
    debug_rect_color: [f32; 3],
    debug_rect_visible: bool,
}

impl Default for DisplayObjectWindow {
    fn default() -> Self {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNT: AtomicUsize = AtomicUsize::new(0);
        let index = COUNT.fetch_add(1, Ordering::Relaxed);
        let debug_rect_color = DEFAULT_DEBUG_COLORS[index % DEFAULT_DEBUG_COLORS.len()];

        Self {
            open_panel: Default::default(),
            debug_rect_color,
            debug_rect_visible: false,
        }
    }
}

impl DisplayObjectWindow {
    pub fn debug_rect_color(&self) -> Option<swf::Color> {
        if self.debug_rect_visible {
            Some(swf::Color {
                r: (self.debug_rect_color[0] * 255.0) as u8,
                g: (self.debug_rect_color[1] * 255.0) as u8,
                b: (self.debug_rect_color[2] * 255.0) as u8,
                a: 255,
            })
        } else {
            None
        }
    }

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
                if let Some(other) = object.parent() {
                    ui.label("Parent");
                    display_object_button(ui, context, messages, other);
                    ui.end_row();
                }

                if let Some(other) = object.masker() {
                    ui.label("Masker");
                    display_object_button(ui, context, messages, other);
                    ui.end_row();
                }

                if let Some(other) = object.maskee() {
                    ui.label("Maskee");
                    display_object_button(ui, context, messages, other);
                    ui.end_row();
                }

                ui.label("Debug Rect");
                ui.horizontal(|ui| {
                    Checkbox::without_text(&mut self.debug_rect_visible).ui(ui);
                    ui.color_edit_button_rgb(&mut self.debug_rect_color);
                });
                ui.end_row();

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

                let color_transform = *object.base().color_transform();
                ui.label("Color Transform");
                ui.label(summary_color_transform(color_transform));
                ui.end_row();
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

                ui.label("AVM1 Path");
                ui.text_edit_singleline(&mut object.path().to_string());
                ui.end_row();

                ui.label("Depth");
                ui.label(object.depth().to_string());
                ui.end_row();

                ui.label("Clip Depth");
                ui.label(object.clip_depth().to_string());
                ui.end_row();

                ui.label("World Bounds");
                ui.label(object.world_bounds().to_string());
                ui.end_row();

                ui.label("Local Bounds");
                ui.label(object.local_bounds().to_string());
                ui.end_row();

                let matrix = *object.base().matrix();
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

fn summary_color_transform(ct: ColorTransform) -> Cow<'static, str> {
    let mut lines = vec![];

    if ct.r_multiply == ct.g_multiply
        && ct.g_multiply == ct.b_multiply
        && ct.r_add == ct.g_add
        && ct.g_add == ct.b_add
    {
        // All color values are the same, no need to list them 3 times
        if let Some(entry) = summary_color_transform_entry("C", ct.r_multiply, ct.r_add) {
            lines.push(entry);
        }
        if let Some(entry) = summary_color_transform_entry("A", ct.a_multiply, ct.a_add) {
            lines.push(entry);
        }
    } else {
        if let Some(entry) = summary_color_transform_entry("R", ct.r_multiply, ct.r_add) {
            lines.push(entry);
        }
        if let Some(entry) = summary_color_transform_entry("G", ct.g_multiply, ct.g_add) {
            lines.push(entry);
        }
        if let Some(entry) = summary_color_transform_entry("B", ct.b_multiply, ct.b_add) {
            lines.push(entry);
        }
        if let Some(entry) = summary_color_transform_entry("A", ct.a_multiply, ct.a_add) {
            lines.push(entry);
        }
    }

    if lines.is_empty() {
        Cow::Borrowed("Default")
    } else {
        Cow::Owned(lines.join("\n"))
    }
}

fn summary_color_transform_entry(name: &str, mult: Fixed8, add: i16) -> Option<String> {
    match (mult, add) {
        (Fixed8::ONE, 0) => None,
        (Fixed8::ONE, _) => Some(format!("{name} = {name} + {add}")),
        (Fixed8::ZERO, _) => Some(format!("{name} = {add}")),
        (_, 0) => Some(format!("{name} = {name} * {mult}")),
        _ => Some(format!("{name} = {name} * {mult} + {add}")),
    }
}

fn summary_name(object: DisplayObject) -> Cow<'static, str> {
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

fn display_object_button<'gc>(
    ui: &mut Ui,
    context: &mut UpdateContext<'_, 'gc>,
    messages: &mut Vec<Message>,
    object: DisplayObject<'gc>,
) {
    if ui.button(summary_name(object)).clicked() {
        messages.push(Message::TrackDisplayObject(DisplayObjectHandle::new(
            context, object,
        )));
    }
}
