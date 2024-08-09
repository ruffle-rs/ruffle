use crate::avm1::{Activation, ActivationIdentifier, Error, Object, TObject, Value};
use crate::context::UpdateContext;
use crate::debug_ui::display_object::open_display_object_button;
use crate::debug_ui::handle::{AVM1ObjectHandle, DisplayObjectHandle};
use crate::debug_ui::Message;
use egui::{Grid, Id, TextEdit, Ui, Window};

#[derive(Debug, Default)]
pub struct Avm1ObjectWindow {
    hovered_debug_rect: Option<DisplayObjectHandle>,
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

                        for key in keys {
                            let value = object.get(key, &mut activation);

                            ui.label(key.to_string());
                            show_avm1_value(
                                ui,
                                &mut activation,
                                value,
                                messages,
                                &mut self.hovered_debug_rect,
                            );
                            ui.end_row();
                        }
                    });
            });
        keep_open
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

pub fn show_avm1_value<'gc>(
    ui: &mut Ui,
    activation: &mut Activation<'_, 'gc>,
    value: Result<Value<'gc>, Error<'gc>>,
    messages: &mut Vec<Message>,
    hover: &mut Option<DisplayObjectHandle>,
) {
    match value {
        Ok(Value::Undefined) => {
            ui.label("Undefined");
        }
        Ok(Value::Null) => {
            ui.label("Null");
        }
        Ok(Value::Bool(value)) => {
            ui.label(value.to_string());
        }
        Ok(Value::Number(value)) => {
            ui.label(value.to_string());
        }
        Ok(Value::String(value)) => {
            TextEdit::singleline(&mut value.to_string()).show(ui);
        }
        Ok(Value::Object(value)) => {
            if value.as_executable().is_some() {
                ui.label("Function");
            } else if ui.button(object_name(value)).clicked() {
                messages.push(Message::TrackAVM1Object(AVM1ObjectHandle::new(
                    activation.context,
                    value,
                )));
            }
        }
        Ok(Value::MovieClip(value)) => {
            if let Some((_, _, object)) = value.resolve_reference(activation) {
                open_display_object_button(ui, activation.context, messages, object, hover);
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
}
