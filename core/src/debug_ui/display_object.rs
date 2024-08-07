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
    AutoSizeMode, Bitmap, DisplayObject, EditText, InteractiveObject, MovieClip, Stage,
    TDisplayObject, TDisplayObjectContainer, TInteractiveObject,
};
use crate::focus_tracker::Highlight;
use egui::collapsing_header::CollapsingState;
use egui::{
    Button, Checkbox, CollapsingHeader, ComboBox, DragValue, Grid, Id, Label, Sense, TextEdit, Ui,
    Widget, Window,
};
use ruffle_wstr::{WStr, WString};
use std::borrow::Cow;
use swf::{Color, ColorTransform, Fixed8, Rectangle, Twips};

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
    Interactive,
    TypeSpecific,
}

#[derive(Debug)]
pub struct DisplayObjectWindow {
    open_panel: Panel,
    debug_rect_color: [f32; 3],
    debug_rect_visible: bool,
    hovered_debug_rect: Option<DisplayObjectHandle>,
    hovered_bounds: Option<Rectangle<Twips>>,
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
            hovered_bounds: None,
            search: Default::default(),
        }
    }
}

impl DisplayObjectWindow {
    pub fn debug_rect_color(&self) -> Option<Color> {
        if self.debug_rect_visible {
            Some(Color {
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

    pub fn hovered_bounds(&self) -> Option<Rectangle<Twips>> {
        self.hovered_bounds.clone()
    }

    pub fn show<'gc>(
        &mut self,
        egui_ctx: &egui::Context,
        context: &mut UpdateContext<'gc>,
        object: DisplayObject<'gc>,
        messages: &mut Vec<Message>,
    ) -> bool {
        let mut keep_open = true;
        self.hovered_debug_rect = None;

        Window::new(summary_name(object))
            .id(Id::new(object.as_ptr()))
            .open(&mut keep_open)
            .scroll([true, true])
            .show(egui_ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.open_panel, Panel::Position, "Position");
                    ui.selectable_value(&mut self.open_panel, Panel::Display, "Display");
                    if object.as_interactive().is_some() {
                        ui.selectable_value(
                            &mut self.open_panel,
                            Panel::Interactive,
                            "Interactive",
                        );
                    }
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
                            self.show_edit_text(ui, context, object)
                        } else if let DisplayObject::Bitmap(object) = object {
                            self.show_bitmap(ui, context, object)
                        } else if let DisplayObject::Stage(object) = object {
                            self.show_stage(ui, context, object, messages)
                        }
                    }
                    Panel::Interactive => {
                        if let Some(int) = object.as_interactive() {
                            self.show_interactive(ui, context, int)
                        }
                    }
                }
            });
        keep_open
    }

    pub fn show_interactive<'gc>(
        &mut self,
        ui: &mut Ui,
        context: &mut UpdateContext<'gc>,
        object: InteractiveObject<'gc>,
    ) {
        Grid::new(ui.id().with("interactive"))
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("Mouse Enabled");
                ui.horizontal(|ui| {
                    let mut enabled = object.mouse_enabled();
                    Checkbox::new(&mut enabled, "Enabled").ui(ui);
                    if enabled != object.mouse_enabled() {
                        object.set_mouse_enabled(context.gc_context, enabled);
                    }
                });
                ui.end_row();

                ui.label("Double-click Enabled");
                ui.horizontal(|ui| {
                    let mut enabled = object.double_click_enabled();
                    Checkbox::new(&mut enabled, "Enabled").ui(ui);
                    if enabled != object.double_click_enabled() {
                        object.set_double_click_enabled(context.gc_context, enabled);
                    }
                });
                ui.end_row();

                ui.label("Tab Enabled");
                {
                    let mut enabled = object.tab_enabled(context);
                    Checkbox::new(&mut enabled, "Enabled").ui(ui);
                    if enabled != object.tab_enabled(context) {
                        object.set_tab_enabled(context, enabled);
                    }
                }
                ui.end_row();

                ui.label("Tab Index");
                ui.horizontal(|ui| {
                    let tab_index = object.tab_index();
                    let mut enabled = tab_index.is_some();
                    Checkbox::without_text(&mut enabled).ui(ui);
                    if enabled != tab_index.is_some() {
                        if enabled {
                            object.set_tab_index(context, Some(0));
                        } else {
                            object.set_tab_index(context, None);
                        }
                    }

                    if let Some(tab_index) = tab_index.map(|i| i as usize) {
                        let mut new_tab_index: usize = tab_index;
                        DragValue::new(&mut new_tab_index).ui(ui);
                        if new_tab_index != tab_index {
                            object.set_tab_index(context, Some(new_tab_index as i32));
                        }
                    } else {
                        ui.label("None");
                    }
                });
                ui.end_row();

                ui.label("Focus Rect");
                let focus_rect = object.focus_rect();
                let mut new_focus_rect = focus_rect;
                ComboBox::from_id_salt(ui.id().with("focus_rect"))
                    .selected_text(optional_boolean_switch_value(focus_rect))
                    .show_ui(ui, |ui| {
                        for value in [None, Some(true), Some(false)] {
                            ui.selectable_value(
                                &mut new_focus_rect,
                                value,
                                optional_boolean_switch_value(value),
                            );
                        }
                    });
                if new_focus_rect != focus_rect {
                    object.set_focus_rect(context.gc(), new_focus_rect);
                }
                ui.end_row();

                ui.label("Highlight Bounds");
                bounds_label(ui, object.highlight_bounds(), &mut self.hovered_bounds);
                ui.end_row();

                ui.label("Derived Properties");
                ui.add_enabled_ui(false, |ui| {
                    ui.vertical(|ui| {
                        ui.checkbox(&mut object.has_focus(), "Has Focus");
                        ui.checkbox(&mut object.is_focusable(context), "Focusable");
                        ui.checkbox(&mut object.is_tabbable(context), "Tabbable");
                        ui.checkbox(&mut object.is_highlightable(context), "Highlightable");
                    });
                });
                ui.end_row();

                ui.label("Actions");
                ui.horizontal(|ui| {
                    if ui.button("Focus").clicked() {
                        let focus_tracker = context.focus_tracker;
                        focus_tracker.set(None, context);
                        focus_tracker.set(Some(object), context);
                    }
                });
                ui.end_row();
            });
    }

    pub fn show_edit_text<'gc>(
        &mut self,
        ui: &mut Ui,
        context: &mut UpdateContext<'gc>,
        object: EditText<'gc>,
    ) {
        Grid::new(ui.id().with("edittext"))
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("Border");
                ui.horizontal(|ui| {
                    let mut has_border = object.has_border();
                    Checkbox::without_text(&mut has_border).ui(ui);
                    if has_border != object.has_border() {
                        object.set_has_border(context.gc(), has_border);
                    }
                    let mut border_color = object.border_color();
                    color_edit_button(ui, &mut border_color);
                    if border_color != object.border_color() {
                        object.set_border_color(context.gc(), border_color);
                    }
                });
                ui.end_row();

                ui.label("Background");
                ui.horizontal(|ui| {
                    let mut has_background = object.has_background();
                    Checkbox::without_text(&mut has_background).ui(ui);
                    if has_background != object.has_background() {
                        object.set_has_background(context.gc(), has_background);
                    }
                    let mut background_color = object.background_color();
                    color_edit_button(ui, &mut background_color);
                    if background_color != object.background_color() {
                        object.set_background_color(context.gc(), background_color);
                    }
                });
                ui.end_row();

                ui.label("Editable");
                ui.horizontal(|ui| {
                    let mut editable = object.is_editable();
                    ui.checkbox(&mut editable, "Enabled");
                    if editable != object.is_editable() {
                        object.set_editable(editable, context);
                    }
                });
                ui.end_row();

                ui.label("HTML");
                ui.horizontal(|ui| {
                    let mut is_html = object.is_html();
                    ui.checkbox(&mut is_html, "Enabled");
                    if is_html != object.is_html() {
                        object.set_is_html(context, is_html);
                    }
                });
                ui.end_row();

                ui.label("Selectable");
                ui.horizontal(|ui| {
                    let mut selectable = object.is_selectable();
                    ui.checkbox(&mut selectable, "Enabled");
                    if selectable != object.is_selectable() {
                        object.set_selectable(selectable, context);
                    }
                });
                ui.end_row();

                ui.label("Word Wrap");
                ui.horizontal(|ui| {
                    let mut word_wrap = object.is_word_wrap();
                    ui.checkbox(&mut word_wrap, "Enabled");
                    if word_wrap != object.is_word_wrap() {
                        object.set_word_wrap(word_wrap, context);
                    }
                });
                ui.end_row();

                ui.label("Multiline");
                ui.horizontal(|ui| {
                    let mut is_multiline = object.is_multiline();
                    ui.checkbox(&mut is_multiline, "Enabled");
                    if is_multiline != object.is_multiline() {
                        object.set_multiline(is_multiline, context);
                    }
                });
                ui.end_row();

                ui.label("Password");
                ui.horizontal(|ui| {
                    let mut is_password = object.is_password();
                    ui.checkbox(&mut is_password, "Enabled");
                    if is_password != object.is_password() {
                        object.set_password(is_password, context);
                    }
                });
                ui.end_row();

                ui.label("Autosize");
                ui.horizontal(|ui| {
                    let mut autosize = object.autosize();
                    ComboBox::from_id_salt(ui.id().with("autosize"))
                        .selected_text(format!("{:?}", autosize))
                        .show_ui(ui, |ui| {
                            for value in [
                                AutoSizeMode::None,
                                AutoSizeMode::Left,
                                AutoSizeMode::Center,
                                AutoSizeMode::Right,
                            ] {
                                ui.selectable_value(&mut autosize, value, format!("{:?}", value));
                            }
                        });
                    if autosize != object.autosize() {
                        object.set_autosize(autosize, context);
                    }
                });
                ui.end_row();

                ui.label("Max Chars");
                ui.horizontal(|ui| {
                    let mut max_chars = object.max_chars();
                    DragValue::new(&mut max_chars).ui(ui);
                    if max_chars != object.max_chars() {
                        object.set_max_chars(max_chars, context);
                    }
                });
                ui.end_row();

                ui.label("Restrict");
                ui.horizontal(|ui| {
                    let restrict = object.restrict();

                    let restrict_enabled = restrict.is_some();
                    let mut new_restrict_enabled = restrict_enabled;
                    Checkbox::without_text(&mut new_restrict_enabled).ui(ui);
                    if new_restrict_enabled != restrict_enabled {
                        let new_restrict = if new_restrict_enabled {
                            Some(WStr::empty())
                        } else {
                            None
                        };
                        object.set_restrict(new_restrict, context);
                    }

                    if let Some(original_restrict) = restrict {
                        let original_restrict = original_restrict.to_string();
                        let mut restrict = original_restrict.clone();
                        ui.text_edit_singleline(&mut restrict);
                        if restrict != original_restrict {
                            object.set_restrict(Some(&WString::from_utf8(&restrict)), context);
                        }
                    } else {
                        ui.weak("Disabled");
                    }
                });
                ui.end_row();

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

                ui.label("Draw Layout Boxes");
                ui.horizontal(|ui| {
                    let mut draw_layout_boxes = object.draw_layout_boxes();
                    ui.checkbox(&mut draw_layout_boxes, "Enabled");
                    if draw_layout_boxes != object.draw_layout_boxes() {
                        object.set_draw_layout_boxes(context, draw_layout_boxes);
                    }
                });
                ui.end_row();
            });

        CollapsingHeader::new("Span List")
            .id_salt(ui.id().with("spans"))
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
                            ui.label(format.font.face.to_string());

                            if format.style.bold && format.style.italic {
                                ui.label("Bold Italic");
                            } else if format.style.bold {
                                ui.label("Bold");
                            } else if format.style.italic {
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

    pub fn show_bitmap<'gc>(
        &mut self,
        ui: &mut Ui,
        context: &mut UpdateContext<'gc>,
        object: Bitmap<'gc>,
    ) {
        let bitmap_data = object.bitmap_data(context.renderer);
        let bitmap_data = bitmap_data.read();
        let mut egui_texture = bitmap_data.egui_texture.borrow_mut();
        let texture = egui_texture.get_or_insert_with(|| {
            let image = egui::ColorImage::from_rgba_premultiplied(
                [bitmap_data.width() as usize, bitmap_data.height() as usize],
                &bitmap_data.pixels_rgba(),
            );
            ui.ctx().load_texture(
                format!("bitmap-{:?}", object.as_ptr()),
                image,
                Default::default(),
            )
        });
        ui.image((texture.id(), texture.size_vec2()));
    }

    pub fn show_movieclip<'gc>(
        &mut self,
        ui: &mut Ui,
        context: &mut UpdateContext<'gc>,
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
            .id_salt(ui.id().with("frames"))
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

    pub fn show_stage<'gc>(
        &mut self,
        ui: &mut Ui,
        context: &mut UpdateContext<'gc>,
        object: Stage<'gc>,
        messages: &mut Vec<Message>,
    ) {
        let focus_tracker = object.focus_tracker();
        let focus = focus_tracker.get();
        Grid::new(ui.id().with("stage"))
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("Stage Focus Rect");
                let mut new_stage_focus_rect = object.stage_focus_rect();
                ui.checkbox(&mut new_stage_focus_rect, "Enabled");
                if new_stage_focus_rect != object.stage_focus_rect() {
                    object.set_stage_focus_rect(context.gc(), new_stage_focus_rect);
                }
                ui.end_row();

                ui.label("Current Focus");
                ui.vertical(|ui| {
                    if let Some(focus) = focus.map(|o| o.as_displayobject()) {
                        open_display_object_button(
                            ui,
                            context,
                            messages,
                            focus,
                            &mut self.hovered_debug_rect,
                        );
                        ui.horizontal(|ui| {
                            if ui.button("Clear").clicked() {
                                focus_tracker.set(None, context);
                            }
                            if ui.button("Re-focus").clicked() {
                                focus_tracker.set(None, context);
                                focus_tracker.set(focus.as_interactive(), context);
                            }
                        });
                    } else {
                        ui.label("None");
                    }
                });
                ui.end_row();

                let highlight = focus_tracker.highlight();
                ui.label("Focus Highlight");
                let highlight_text = match highlight {
                    Highlight::Inactive => "Inactive",
                    Highlight::ActiveHidden => "Active, Hidden",
                    Highlight::ActiveVisible => "Active, Visible",
                };
                ui.label(highlight_text);
                ui.end_row();
            });

        let tab_order = focus_tracker.tab_order(context);
        let tab_order_suffix = if tab_order.is_custom() {
            "custom"
        } else {
            "automatic"
        };
        CollapsingHeader::new(format!("Tab Order ({})", tab_order_suffix))
            .id_salt(ui.id().with("tab_order"))
            .show(ui, |ui| {
                Grid::new(ui.id().with("tab_order_grid"))
                    .num_columns(3)
                    .show(ui, |ui| {
                        ui.label("#");
                        ui.label("Object");
                        ui.label("Actions");
                        ui.label("Tab Index");
                        ui.end_row();

                        for (i, object) in tab_order.iter().enumerate() {
                            if Some(*object) == focus {
                                ui.label(format!("{}.*", i + 1));
                            } else {
                                ui.label(format!("{}.", i + 1));
                            }
                            open_display_object_button(
                                ui,
                                context,
                                messages,
                                object.as_displayobject(),
                                &mut self.hovered_debug_rect,
                            );
                            if ui.button("Focus").clicked() {
                                focus_tracker.set(Some(*object), context);
                            }
                            if let Some(tab_index) = object.tab_index() {
                                ui.label(tab_index.to_string());
                            } else {
                                ui.label("(none)");
                            }
                            ui.end_row();
                        }
                    });
            });
    }

    pub fn show_display<'gc>(
        &mut self,
        ui: &mut Ui,
        context: &mut UpdateContext<'gc>,
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

                ui.label("Is Root");
                if object.is_root() {
                    ui.label("Yes");
                } else {
                    ui.label("No");
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
                    object.set_visible(context, is_visible);
                }

                ui.label("Blend mode");
                let old_blend = object.blend_mode();
                let mut new_blend = old_blend;
                ComboBox::from_id_salt(ui.id().with("blendmode"))
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

                if let Some(obj) = object.as_container() {
                    ui.label("Mouse children enabled");
                    ui.horizontal(|ui| {
                        let mut enabled = obj.raw_container().mouse_children();
                        Checkbox::new(&mut enabled, "Enabled").ui(ui);
                        if enabled != obj.raw_container().mouse_children() {
                            obj.raw_container_mut(context.gc_context)
                                .set_mouse_children(enabled);
                        }
                    });
                    ui.end_row();

                    ui.label("Tab children enabled");
                    ui.horizontal(|ui| {
                        let mut enabled = obj.is_tab_children(context);
                        Checkbox::new(&mut enabled, "Enabled").ui(ui);
                        if enabled != obj.is_tab_children(context) {
                            obj.set_tab_children(context, enabled);
                        }
                    });
                    ui.end_row();
                }
            });

        let filters = object.filters();
        if !filters.is_empty() {
            CollapsingHeader::new(format!("Filters ({})", filters.len()))
                .id_salt(ui.id().with("filters"))
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
        context: &mut UpdateContext<'gc>,
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
                bounds_label(ui, object.world_bounds(), &mut self.hovered_bounds);
                ui.end_row();

                ui.label("Local Bounds");
                bounds_label(ui, object.local_bounds(), &mut None);
                ui.end_row();

                ui.label("Self Bounds");
                bounds_label(ui, object.self_bounds(), &mut None);
                ui.end_row();

                ui.label("Scroll Rect");
                if let Some(scroll_rect) = object.scroll_rect() {
                    bounds_label(ui, scroll_rect, &mut None);
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
        context: &mut UpdateContext<'gc>,
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
        context: &mut UpdateContext<'gc>,
        object: DisplayObject<'gc>,
        messages: &mut Vec<Message>,
        search: &WStr,
    ) {
        if !matches_search(object, search) {
            return;
        }
        if let Some(ctr) = object.as_container().filter(|x| x.num_children() > 0) {
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
        DisplayObject::MovieClip(_)
            | DisplayObject::EditText(_)
            | DisplayObject::Bitmap(_)
            | DisplayObject::Stage(_)
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

fn optional_boolean_switch_value(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => "Enabled",
        Some(false) => "Disabled",
        None => "Default",
    }
}

fn color_edit_button(ui: &mut Ui, color: &mut Color) {
    use egui::Color32;

    let original_color32 = Color32::from_rgba_unmultiplied(color.r, color.g, color.b, color.a);
    let mut new_color32 = original_color32;
    ui.color_edit_button_srgba(&mut new_color32);
    if original_color32 != new_color32 {
        let [r, g, b, a] = new_color32.to_srgba_unmultiplied();
        color.r = r;
        color.g = g;
        color.b = b;
        color.a = a;
    }
}

fn bounds_label(ui: &mut Ui, bounds: Rectangle<Twips>, hover: &mut Option<Rectangle<Twips>>) {
    if !bounds.is_valid() {
        ui.weak("Invalid");
        return;
    }

    let label = Label::new(bounds.to_string()).sense(Sense::hover());
    if ui.add(label).hovered() {
        *hover = Some(bounds);
    } else {
        *hover = None;
    }
}

pub fn open_display_object_button<'gc>(
    ui: &mut Ui,
    context: &mut UpdateContext<'gc>,
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
