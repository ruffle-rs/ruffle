use crate::avm2::property::Property;
use crate::avm2::{Activation, Error, Namespace, Object, TObject, Value};
use crate::context::UpdateContext;
use crate::debug_ui::handle::{AVM2ObjectHandle, DisplayObjectHandle};
use crate::debug_ui::Message;
use egui::{Grid, Id, TextEdit, Ui, Window};
use gc_arena::MutationContext;
use std::borrow::Cow;

#[derive(Debug, Default)]
pub struct Avm2ObjectWindow {
    hovered_debug_rect: Option<DisplayObjectHandle>,
}

impl Avm2ObjectWindow {
    pub fn hovered_debug_rect(&self) -> Option<DisplayObjectHandle> {
        self.hovered_debug_rect.clone()
    }

    pub fn show<'gc>(
        &mut self,
        egui_ctx: &egui::Context,
        context: &mut UpdateContext<'_, 'gc>,
        object: Object<'gc>,
        messages: &mut Vec<Message>,
    ) -> bool {
        let mut keep_open = true;
        let domain = context.avm2.stage_domain();
        let mut activation = Activation::from_domain(context.reborrow(), domain);
        Window::new(object_name(activation.context.gc_context, object))
            .id(Id::new(object.as_ptr()))
            .open(&mut keep_open)
            .scroll2([true, true])
            .show(egui_ctx, |ui| {
                if let Some(vtable) = object.vtable() {
                    let mut entries = Vec::<(String, Namespace<'gc>, Property)>::new();
                    for (name, ns, prop) in vtable.resolved_traits().iter() {
                        entries.push((name.to_string(), ns, *prop));
                    }
                    entries.sort_by(|a, b| a.0.cmp(&b.0));

                    Grid::new(ui.id().with("properties"))
                        .num_columns(3)
                        .show(ui, |ui| {
                            for (name, ns, prop) in entries {
                                // TODO: filtering
                                if ns.is_public() {
                                    match prop {
                                        Property::Slot { slot_id }
                                        | Property::ConstSlot { slot_id } => {
                                            let value = object.get_slot(slot_id);
                                            ui.label(name).on_hover_ui(|ui| {
                                                ui.label(format!("{ns:?}"));
                                            });
                                            show_avm2_value(ui, &mut activation, value, messages);
                                            ui.end_row();
                                        }
                                        Property::Virtual { get: Some(get), .. } => {
                                            let value =
                                                object.call_method(get, &[], &mut activation);
                                            ui.label(name).on_hover_ui(|ui| {
                                                ui.label(format!("{ns:?}"));
                                            });
                                            show_avm2_value(ui, &mut activation, value, messages);
                                            ui.end_row();
                                        }
                                        Property::Method { .. } => {}
                                        Property::Virtual { get: None, set: _ } => {}
                                    }
                                }
                            }
                        });
                }
            });
        keep_open
    }
}

fn object_name<'gc>(mc: MutationContext<'gc, '_>, object: Object<'gc>) -> String {
    let name = object
        .instance_of_class_definition()
        .map(|r| Cow::Owned(r.read().name().to_qualified_name(mc).to_string()))
        .unwrap_or(Cow::Borrowed("Object"));
    format!("{} {:p}", name, object.as_ptr())
}

pub fn show_avm2_value<'gc>(
    ui: &mut Ui,
    activation: &mut Activation<'_, 'gc>,
    value: Result<Value<'gc>, Error<'gc>>,
    messages: &mut Vec<Message>,
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
        Ok(Value::Integer(value)) => {
            ui.label(value.to_string());
        }
        Ok(Value::String(value)) => {
            TextEdit::singleline(&mut value.to_string()).show(ui);
        }
        Ok(Value::Object(value)) => {
            if value.as_executable().is_some() {
                ui.label("Function");
            } else if ui
                .button(object_name(activation.context.gc_context, value))
                .clicked()
            {
                messages.push(Message::TrackAVM2Object(AVM2ObjectHandle::new(
                    &mut activation.context,
                    value,
                )));
            }
        }
        Err(e) => {
            ui.colored_label(ui.style().visuals.error_fg_color, e.to_string());
        }
    }
}
