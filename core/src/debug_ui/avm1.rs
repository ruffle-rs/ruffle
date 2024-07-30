use crate::avm1::{Activation, ActivationIdentifier, Error, Object, TObject, Value};
use crate::context::UpdateContext;
use crate::debug_ui::display_object::open_display_object_button;
use crate::debug_ui::handle::{AVM1ObjectHandle, DisplayObjectHandle};
use crate::debug_ui::Message;
use crate::string::AvmString;
use egui::{Grid, Id, TextBuffer, TextEdit, Ui, Window};

#[derive(Debug, Default)]
pub struct Avm1ObjectWindow {
    hovered_debug_rect: Option<DisplayObjectHandle>,
    key_filter_string: String,
    edited_key: Option<String>,
    value_edit_buf: String,
    /// True if the active text edit should be focused (after clicking 'edit', etc.)
    focus_text_edit: bool,
}

impl Avm1ObjectWindow {
    pub fn hovered_debug_rect(&self) -> Option<DisplayObjectHandle> {
        self.hovered_debug_rect.clone()
    }

    pub fn show<'gc>(
        &mut self,
        egui_ctx: &egui::Context,
        context: &mut UpdateContext<'gc>,
        object: Object<'gc>,
        messages: &mut Vec<Message>,
    ) -> bool {
        let mut keep_open = true;
        let base_clip = context.stage.into();
        let mut activation =
            Activation::from_nothing(context, ActivationIdentifier::root("Debug"), base_clip);
        Window::new(object_name(object))
            .id(Id::new(object.as_ptr()))
            .open(&mut keep_open)
            .scroll([true, true])
            .show(egui_ctx, |ui| {
                Grid::new(ui.id().with("properties"))
                    .num_columns(2)
                    .show(ui, |ui| {
                        let mut keys = object.get_keys(&mut activation, true);
                        keys.sort();
                        ui.add(
                            egui::TextEdit::singleline(&mut self.key_filter_string)
                                .hint_text("🔍 Filter"),
                        );
                        ui.end_row();
                        keys.retain(|key| {
                            self.key_filter_string.is_empty()
                                || key
                                    .to_string()
                                    .to_ascii_lowercase()
                                    .contains(&self.key_filter_string.to_ascii_lowercase())
                        });

                        for key in keys {
                            let value = object.get(key, &mut activation);

                            ui.label(key.to_string());
                            if let Some(new) =
                                self.show_avm1_value(ui, &mut activation, &key, value, messages)
                            {
                                if let Err(e) = object.set(key, new, &mut activation) {
                                    tracing::error!("Failed to set key {key}: {e}");
                                }
                            }
                            ui.end_row();
                        }
                    });
            });
        keep_open
    }
    /// Shows an egui widget to inspect and (for certain value types) edit an AVM1 value.
    ///
    /// Optionally returns the updated value, if the user edited it.
    pub fn show_avm1_value<'gc>(
        &mut self,
        ui: &mut Ui,
        activation: &mut Activation<'_, 'gc>,
        key: &AvmString,
        value: Result<Value<'gc>, Error<'gc>>,
        messages: &mut Vec<Message>,
    ) -> Option<Value<'gc>> {
        match value {
            Ok(Value::Undefined) => {
                ui.label("Undefined");
            }
            Ok(Value::Null) => {
                ui.label("Null");
            }
            Ok(Value::Bool(mut value)) => {
                if ui.checkbox(&mut value, "").clicked() {
                    return Some(Value::Bool(value));
                }
            }
            Ok(Value::Number(value)) => {
                return self.num_edit_ui(ui, key, value).map(Value::Number);
            }
            Ok(Value::String(value)) => {
                return self
                    .string_edit_ui(ui, key, value)
                    .map(|string| Value::String(AvmString::new_utf8(activation.gc(), string)));
            }
            Ok(Value::Object(value)) => {
                if value.as_executable().is_some() {
                    ui.label("Function");
                } else if ui.button(object_name(value)).clicked() {
                    messages.push(Message::TrackAVM1Object(AVM1ObjectHandle::new(
                        &mut activation.context,
                        value,
                    )));
                }
            }
            Ok(Value::MovieClip(value)) => {
                if let Some((_, _, object)) = value.resolve_reference(activation) {
                    open_display_object_button(
                        ui,
                        &mut activation.context,
                        messages,
                        object,
                        &mut self.hovered_debug_rect,
                    );
                } else {
                    ui.colored_label(
                        ui.style().visuals.error_fg_color,
                        format!("Unknown movieclip {}", value.path()),
                    );
                }
            }
            Err(e) => {
                ui.colored_label(ui.style().visuals.error_fg_color, e.to_string());
            }
        }
        None
    }
    fn num_edit_ui(&mut self, ui: &mut Ui, key: &AvmString, num: f64) -> Option<f64> {
        let mut new_val = None;
        if self
            .edited_key
            .as_ref()
            .is_some_and(|edit_key| *edit_key == key.to_utf8_lossy())
        {
            ui.horizontal(|ui| {
                let re = ui
                    .add(egui::TextEdit::singleline(&mut self.value_edit_buf).desired_width(96.0));
                if self.focus_text_edit {
                    re.request_focus();
                    self.focus_text_edit = false;
                }
                match self.value_edit_buf.parse::<f64>() {
                    Ok(num) => {
                        if ui.input(|inp| inp.key_pressed(egui::Key::Enter))
                            || ui.set_button().clicked()
                        {
                            new_val = Some(num);
                            self.edited_key = None;
                        }
                    }
                    Err(e) => {
                        ui.add_enabled(false, egui::Button::new(CHECKMARK_ICON))
                            .on_disabled_hover_text(e.to_string());
                    }
                }
                if ui.cancel_button().clicked() {
                    self.edited_key = None;
                }
            });
        } else {
            ui.horizontal(|ui| {
                let num_str = num.to_string();
                ui.label(&num_str);
                if ui.edit_button().clicked() {
                    self.edited_key = Some(key.to_utf8_lossy().into_owned());
                    self.value_edit_buf = num_str;
                    self.focus_text_edit = true;
                }
            });
        }
        new_val
    }
    fn string_edit_ui(
        &mut self,
        ui: &mut Ui,
        key: &AvmString,
        string: AvmString,
    ) -> Option<String> {
        let mut new_val = None;
        ui.horizontal(|ui| {
            if self
                .edited_key
                .as_ref()
                .is_some_and(|edit_key| *edit_key == key.to_utf8_lossy())
            {
                let re = ui.add(TextEdit::singleline(&mut self.value_edit_buf).desired_width(96.0));
                if self.focus_text_edit {
                    re.request_focus();
                    self.focus_text_edit = false;
                }
                if ui.set_button().clicked() {
                    new_val = Some(self.value_edit_buf.take());
                    self.edited_key = None;
                }
                if ui.cancel_button().clicked() {
                    self.edited_key = None;
                }
            } else {
                ui.label(string.to_utf8_lossy());
                if ui.edit_button().clicked() {
                    self.value_edit_buf = string.to_string();
                    self.edited_key = Some(key.to_string());
                    self.focus_text_edit = true;
                }
            }
        });
        new_val
    }
}

const PENCIL_ICON: &str = "✏";
const CHECKMARK_ICON: &str = "✔";
const CANCEL_ICON: &str = "🗙";

trait UiExt {
    fn edit_button(&mut self) -> egui::Response;
    fn set_button(&mut self) -> egui::Response;
    fn cancel_button(&mut self) -> egui::Response;
}

impl UiExt for egui::Ui {
    fn edit_button(&mut self) -> egui::Response {
        self.button(PENCIL_ICON).on_hover_text("Edit")
    }
    fn set_button(&mut self) -> egui::Response {
        self.button(CHECKMARK_ICON).on_hover_text("Set")
    }
    fn cancel_button(&mut self) -> egui::Response {
        self.button(CANCEL_ICON).on_hover_text("Cancel")
    }
}

fn object_name(object: Object) -> String {
    // TODO: Find a way to give more meaningful names here.
    // Matching __proto__ to a constant and taking the constants name works, but is super expensive
    if object.as_executable().is_some() {
        format!("Function {:p}", object.as_ptr())
    } else if object.as_array_object().is_some() {
        format!("Array {:p}", object.as_ptr())
    } else {
        format!("Object {:p}", object.as_ptr())
    }
}
