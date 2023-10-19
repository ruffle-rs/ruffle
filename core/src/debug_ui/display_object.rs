mod search;

use ruffle_render::blend::ExtendedBlendMode;
pub use search::DisplayObjectSearchWindow;

use crate::avm1::TObject as _;
use crate::avm2::object::TObject as _;
use crate::context::UpdateContext;
use crate::debug_ui::handle::{AVM1ObjectHandle, AVM2ObjectHandle, DisplayObjectHandle};
use crate::debug_ui::movie::open_movie_button;
use crate::debug_ui::Message;
use crate::display_object::{
    DisplayObject, EditText, MovieClip, TDisplayObject, TDisplayObjectContainer,
};
use egui::collapsing_header::CollapsingState;
use egui::{Button, Checkbox, CollapsingHeader, ComboBox, Grid, Id, TextEdit, Ui, Widget, Window};
use ruffle_wstr::{WStr, WString};
use std::borrow::Cow;
use swf::{ColorTransform, Fixed8};

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

const ALL_BLEND_MODES: [ExtendedBlendMode; 15] = [
    ExtendedBlendMode::Normal,
    ExtendedBlendMode::Layer,
    ExtendedBlendMode::Multiply,
    ExtendedBlendMode::Screen,
    ExtendedBlendMode::Lighten,
    ExtendedBlendMode::Darken,
    ExtendedBlendMode::Difference,
    ExtendedBlendMode::Add,
    ExtendedBlendMode::Subtract,
    ExtendedBlendMode::Invert,
    ExtendedBlendMode::Alpha,
    ExtendedBlendMode::Erase,
    ExtendedBlendMode::Overlay,
    ExtendedBlendMode::HardLight,
    ExtendedBlendMode::Shader,
];

#[derive(Debug, Eq, PartialEq, Hash, Default, Copy, Clone)]
enum Panel {
    #[default]
    Position,
    Display,
    Children,
    TypeSpecific,
}

#[derive(Debug)]
pub struct DisplayObjectWindow {
    open_panel: Panel,
    debug_rect_color: [f32; 3],
    debug_rect_visible: bool,
    hovered_debug_rect: Option<DisplayObjectHandle>,
    search: String,
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
            hovered_debug_rect: None,
            search: Default::default(),
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

    pub fn hovered_debug_rect(&self) -> Option<DisplayObjectHandle> {
        self.hovered_debug_rect.clone()
    }

    pub fn show<'gc>(
        &mut self,
        egui_ctx: &egui::Context,
        context: &mut UpdateContext<'_, 'gc>,
        object: DisplayObject<'gc>,
        messages: &mut Vec<Message>,
    ) -> bool {
        let mut keep_open = true;
        self.hovered_debug_rect = None;

        Window::new(summary_name(object))
            .id(Id::new(object.as_ptr()))
            .open(&mut keep_open)
            .scroll2([true, true])
            .show(egui_ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.open_panel, Panel::Position, "Position");
                    ui.selectable_value(&mut self.open_panel, Panel::Display, "Display");
                    if has_type_specific_tab(object) {
                        ui.selectable_value(
                            &mut self.open_panel,
                            Panel::TypeSpecific,
                            display_object_type(object),
                        );
                    }
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
                    Panel::Position => self.show_position(ui, context, object, messages),
                    Panel::Display => self.show_display(ui, context, object, messages),
                    Panel::Children => self.show_children(ui, context, object, messages),
                    Panel::TypeSpecific => {
                        if let DisplayObject::MovieClip(object) = object {
                            self.show_movieclip(ui, context, object)
                        } else if let DisplayObject::EditText(object) = object {
                            self.show_edit_text(ui, object)
                        }
                    }
                }
            });
        keep_open
    }

    pub fn show_edit_text(&mut self, ui: &mut Ui, object: EditText) {
        Grid::new(ui.id().with("edittext"))
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("Selection");
                if let Some(selection) = object.selection() {
                    if selection.is_caret() {
                        ui.label(selection.start().to_string());
                    } else {
                        ui.label(format!("{} - {}", selection.start(), selection.end()));
                    }
                } else {
                    ui.weak("None");
                }
                ui.end_row();
            });

        CollapsingHeader::new("Span List")
            .id_source(ui.id().with("spans"))
            .show(ui, |ui| {
                Grid::new(ui.id().with("spans"))
                    .num_columns(7)
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Start");
                        ui.label("End");
                        ui.label("Length");
                        ui.label("URL");
                        ui.label("Font");
                        ui.label("Style");
                        ui.label("Text");
                        ui.end_row();

                        for (start, end, text, format) in object.spans().iter_spans() {
                            ui.label(start.to_string());
                            ui.label(end.to_string());

                            ui.label(format.span_length.to_string());
                            ui.label(format.url.to_string());
                            ui.label(format.font.to_string());

                            if format.bold && format.italic {
                                ui.label("Bold Italic");
                            } else if format.bold {
                                ui.label("Bold");
                            } else if format.italic {
                                ui.label("Italic");
                            } else {
                                ui.label("Regular");
                            }

                            ui.label(text.to_string());
                            ui.end_row();
                        }
                    });
            });
    }

    pub fn show_movieclip<'gc>(
        &mut self,
        ui: &mut Ui,
        context: &mut UpdateContext<'_, 'gc>,
        object: MovieClip<'gc>,
    ) {
        Grid::new(ui.id().with("movieclip"))
            .num_columns(2)
            .show(ui, |ui| {
                let label = object.current_label();
                ui.label("Current Frame");
                if let Some((label, label_frame)) = label {
                    ui.label(format!(
                        "{} ({} from frame {})",
                        object.current_frame(),
                        label,
                        label_frame,
                    ));
                } else {
                    ui.label(object.current_frame().to_string());
                }
                ui.end_row();

                ui.label("Total Frames");
                ui.label(object.total_frames().to_string());
                ui.end_row();

                ui.label("Flags");
                ui.add_enabled_ui(false, |ui| {
                    ui.vertical(|ui| {
                        ui.checkbox(&mut object.initialized(), "Initialized");
                        ui.checkbox(&mut object.playing(), "Playing");
                        ui.checkbox(
                            &mut object.programmatically_played(),
                            "Programmatically Played",
                        );
                    });
                });
                ui.end_row();

                ui.label("Controls");
                ui.horizontal(|ui| {
                    if ui.button("Play").clicked() {
                        object.play(context);
                    }
                    if ui.button("Stop").clicked() {
                        object.stop(context);
                    }
                    if ui.button("Prev").clicked() {
                        object.prev_frame(context);
                    }
                    if ui.button("Next").clicked() {
                        object.next_frame(context);
                    }
                });
                ui.end_row();
            });

        CollapsingHeader::new("Frame List")
            .id_source(ui.id().with("frames"))
            .show(ui, |ui| {
                Grid::new(ui.id().with("frames"))
                    .num_columns(5)
                    .show(ui, |ui| {
                        let num_frames = object.total_frames();
                        let scenes = object.scenes();

                        ui.label("#");
                        ui.label("Scene");
                        ui.label("Label");
                        ui.label("Has Script");
                        ui.label("Goto-and-");
                        ui.end_row();

                        for frame in 1..=num_frames {
                            ui.label(frame.to_string());
                            ui.label(
                                scenes
                                    .iter()
                                    .find(|s| s.start <= frame && (s.start + s.length) > frame)
                                    .map(|s| s.name.to_string())
                                    .unwrap_or_default(),
                            );
                            ui.label(
                                object
                                    .labels_in_range(frame, frame + 1)
                                    .first()
                                    .map(|(l, _)| l.to_string())
                                    .unwrap_or_default(),
                            );
                            if object.has_frame_script(frame) {
                                ui.add_enabled(false, Button::new("AVM2 Script"));
                            } else {
                                ui.label("");
                            }
                            if object.current_frame() != frame {
                                ui.horizontal(|ui| {
                                    if ui.button("Stop").clicked() {
                                        object.goto_frame(context, frame, true);
                                    }
                                    if ui.button("Play").clicked() {
                                        object.goto_frame(context, frame, false);
                                    }
                                });
                            } else {
                                ui.label("(current)");
                            }
                            ui.end_row();
                        }
                    });
            });
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
                ui.label("Parent");
                if let Some(other) = object.parent() {
                    open_display_object_button(
                        ui,
                        context,
                        messages,
                        other,
                        &mut self.hovered_debug_rect,
                    );
                } else {
                    ui.colored_label(ui.style().visuals.error_fg_color, "Orphaned");
                }
                ui.end_row();

                ui.label("AVM1 Root");
                if object.avm1_root().as_ptr() != object.as_ptr() {
                    open_display_object_button(
                        ui,
                        context,
                        messages,
                        object.avm1_root(),
                        &mut self.hovered_debug_rect,
                    );
                } else {
                    ui.label("Self");
                }
                ui.end_row();

                ui.label("AVM2 Root");
                if let Some(other) = object.avm2_root() {
                    if object.as_ptr() != object.as_ptr() {
                        open_display_object_button(
                            ui,
                            context,
                            messages,
                            other,
                            &mut self.hovered_debug_rect,
                        );
                    } else {
                        ui.label("Self");
                    }
                } else {
                    ui.colored_label(ui.style().visuals.error_fg_color, "None");
                }
                ui.end_row();

                if let Some(other) = object.masker() {
                    ui.label("Masker");
                    open_display_object_button(
                        ui,
                        context,
                        messages,
                        other,
                        &mut self.hovered_debug_rect,
                    );
                    ui.end_row();
                }

                if let Some(other) = object.maskee() {
                    ui.label("Maskee");
                    open_display_object_button(
                        ui,
                        context,
                        messages,
                        other,
                        &mut self.hovered_debug_rect,
                    );
                    ui.end_row();
                }

                ui.label("Cache as Bitmap");
                ui.horizontal(|ui| {
                    if object.filters().is_empty() {
                        let mut enabled = object.is_bitmap_cached_preference();
                        Checkbox::new(&mut enabled, "Enabled").ui(ui);
                        if enabled != object.is_bitmap_cached_preference() {
                            object.set_bitmap_cached_preference(context.gc_context, enabled);
                        }
                    } else {
                        ui.label("Forced due to filters");
                    }
                    if ui.button("Invalidate").clicked() {
                        object.invalidate_cached_bitmap(context.gc_context);
                    }
                });
                ui.end_row();

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

        let filters = object.filters();
        if !filters.is_empty() {
            CollapsingHeader::new(format!("Filters ({})", filters.len()))
                .id_source(ui.id().with("filters"))
                .show(ui, |ui| {
                    for filter in filters {
                        ui.label(format!("{:?}", filter));
                    }
                });
        }
    }

    pub fn show_position<'gc>(
        &mut self,
        ui: &mut Ui,
        context: &mut UpdateContext<'_, 'gc>,
        object: DisplayObject<'gc>,
        messages: &mut Vec<Message>,
    ) {
        Grid::new(ui.id().with("position"))
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("Name");
                // &mut of a temporary thing because we don't want to actually be able to change this
                // If we disable it, the user can't highlight or interact with it, so this makes it readonly but enabled
                ui.text_edit_singleline(&mut object.name().to_string());
                ui.end_row();

                if let crate::avm1::Value::Object(object) = object.object() {
                    ui.label("AVM1 Object");
                    if ui.button(format!("{:p}", object.as_ptr())).clicked() {
                        messages.push(Message::TrackAVM1Object(AVM1ObjectHandle::new(
                            context, object,
                        )));
                    }
                    ui.end_row();
                }

                if let crate::avm2::Value::Object(object) = object.object2() {
                    ui.label("AVM2 Object");
                    if ui.button(format!("{:p}", object.as_ptr())).clicked() {
                        messages.push(Message::TrackAVM2Object(AVM2ObjectHandle::new(
                            context, object,
                        )));
                    }
                    ui.end_row();
                }

                ui.label("Character");
                let id = object.id();
                if let Some(name) =
                    context
                        .library
                        .library_for_movie(object.movie())
                        .and_then(|l| {
                            l.export_characters().iter().find_map(|(k, v)| {
                                if *v == id {
                                    Some(k)
                                } else {
                                    None
                                }
                            })
                        })
                {
                    ui.label(format!("{id} {name}"));
                } else {
                    ui.label(id.to_string());
                }
                ui.end_row();

                ui.label("Movie");
                open_movie_button(ui, &object.movie(), messages);
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
                if object.world_bounds().is_valid() {
                    ui.label(object.world_bounds().to_string());
                } else {
                    ui.weak("Invalid");
                }
                ui.end_row();

                ui.label("Local Bounds");
                if object.local_bounds().is_valid() {
                    ui.label(object.local_bounds().to_string());
                } else {
                    ui.weak("Invalid");
                }
                ui.end_row();

                ui.label("Self Bounds");
                if object.self_bounds().is_valid() {
                    ui.label(object.self_bounds().to_string());
                } else {
                    ui.weak("Invalid");
                }
                ui.end_row();

                ui.label("Scroll Rect");
                if let Some(scroll_rect) = object.scroll_rect() {
                    if scroll_rect.is_valid() {
                        ui.label(scroll_rect.to_string());
                    } else {
                        ui.weak("Invalid");
                    }
                } else {
                    ui.label("None");
                }
                ui.end_row();

                let matrix = *object.base().matrix();
                ui.label("Local Position");
                ui.label(format!("{:.2}, {:.2}", matrix.tx, matrix.ty));
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
        TextEdit::singleline(&mut self.search)
            .hint_text("Search")
            .show(ui);
        // Let's search ascii-insensitive for QOL
        let search = WString::from_utf8(&self.search).to_ascii_lowercase();

        if let Some(ctr) = object.as_container() {
            for child in ctr.iter_render_list() {
                self.show_display_tree(ui, context, child, messages, &search);
            }
        }
    }

    pub fn show_display_tree<'gc>(
        &mut self,
        ui: &mut Ui,
        context: &mut UpdateContext<'_, 'gc>,
        object: DisplayObject<'gc>,
        messages: &mut Vec<Message>,
        search: &WStr,
    ) {
        if !matches_search(object, search) {
            return;
        }
        if let Some(ctr) = object.as_container() {
            CollapsingState::load_with_default_open(ui.ctx(), ui.id().with(object.as_ptr()), false)
                .show_header(ui, |ui| {
                    open_display_object_button(
                        ui,
                        context,
                        messages,
                        object,
                        &mut self.hovered_debug_rect,
                    );
                })
                .body(|ui| {
                    for child in ctr.iter_render_list() {
                        self.show_display_tree(ui, context, child, messages, search);
                    }
                });
        } else {
            // This item is not expandable, but we want to keep
            // the space empty where the expand button would be,
            // so it doesn't look like a sibling of the parent.
            ui.indent(ui.id().with(object.as_ptr()), |ui| {
                open_display_object_button(
                    ui,
                    context,
                    messages,
                    object,
                    &mut self.hovered_debug_rect,
                );
            });
        }
    }
}

fn matches_search(object: DisplayObject, search: &WStr) -> bool {
    if object.name().to_ascii_lowercase().contains(search) {
        return true;
    }

    if let Some(ctr) = object.as_container() {
        for child in ctr.iter_render_list() {
            if matches_search(child, search) {
                return true;
            }
        }
    }

    false
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

fn has_type_specific_tab(object: DisplayObject) -> bool {
    matches!(
        object,
        DisplayObject::MovieClip(_) | DisplayObject::EditText(_)
    )
}

fn summary_name(object: DisplayObject) -> Cow<'static, str> {
    let do_type = display_object_type(object);
    let name = object.name();

    if name.is_empty() {
        Cow::Borrowed(do_type)
    } else {
        Cow::Owned(format!("{do_type} \"{name}\""))
    }
}

fn display_object_type(object: DisplayObject) -> &'static str {
    match object {
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
    }
}

fn blend_mode_name(mode: ExtendedBlendMode) -> &'static str {
    match mode {
        ExtendedBlendMode::Normal => "Normal",
        ExtendedBlendMode::Layer => "Layer",
        ExtendedBlendMode::Multiply => "Multiply",
        ExtendedBlendMode::Screen => "Screen",
        ExtendedBlendMode::Lighten => "Lighten",
        ExtendedBlendMode::Darken => "Darken",
        ExtendedBlendMode::Difference => "Difference",
        ExtendedBlendMode::Add => "Add",
        ExtendedBlendMode::Subtract => "Subtract",
        ExtendedBlendMode::Invert => "Invert",
        ExtendedBlendMode::Alpha => "Alpha",
        ExtendedBlendMode::Erase => "Erase",
        ExtendedBlendMode::Overlay => "Overlay",
        ExtendedBlendMode::HardLight => "HardLight",
        ExtendedBlendMode::Shader => "Shader",
    }
}

pub fn open_display_object_button<'gc>(
    ui: &mut Ui,
    context: &mut UpdateContext<'_, 'gc>,
    messages: &mut Vec<Message>,
    object: DisplayObject<'gc>,
    hover: &mut Option<DisplayObjectHandle>,
) {
    let response = ui.button(summary_name(object));
    if response.hovered() {
        *hover = Some(DisplayObjectHandle::new(context, object));
    }
    if response.clicked() {
        messages.push(Message::TrackDisplayObject(DisplayObjectHandle::new(
            context, object,
        )));
    }
}
