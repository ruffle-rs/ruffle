use crate::avm2::property::Property;
use crate::avm2::{Activation, Error, Namespace, Object, TObject, Value};
use crate::context::UpdateContext;
use crate::debug_ui::handle::{AVM2ObjectHandle, DisplayObjectHandle};
use crate::debug_ui::Message;
use egui::{Align, Id, Layout, TextEdit, Ui, Widget, Window};
use egui_extras::{Column, TableBuilder};
use fnv::FnvHashMap;
use gc_arena::MutationContext;
use std::borrow::Cow;

#[derive(Debug, Default)]
pub struct Avm2ObjectWindow {
    hovered_debug_rect: Option<DisplayObjectHandle>,
    show_private_items: bool,
    call_getters: bool,
    getter_values: FnvHashMap<(String, String), Option<ValueWidget>>,
    search: String,
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
            .scroll2([false, false]) // Table will provide its own scrolling
            .show(egui_ctx, |ui| {
                self.show_properties(object, messages, &mut activation, ui);
            });
        keep_open
    }

    fn show_properties<'gc>(
        &mut self,
        object: Object<'gc>,
        messages: &mut Vec<Message>,
        activation: &mut Activation<'_, 'gc>,
        ui: &mut Ui,
    ) {
        let mut entries = Vec::<(String, Namespace<'gc>, Property)>::new();
        // We can't access things whilst we iterate the vtable, so clone and sort it all here
        if let Some(vtable) = object.vtable() {
            for (name, ns, prop) in vtable.resolved_traits().iter() {
                entries.push((name.to_string(), ns, *prop));
            }
        }
        entries.sort_by(|a, b| a.0.cmp(&b.0));

        // [NA] Adding these on the same line seems to break the width of the table :(
        TextEdit::singleline(&mut self.search)
            .hint_text("Search...")
            .ui(ui);
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.show_private_items, "Show Private Items");
            ui.checkbox(&mut self.call_getters, "Call Getters");
        });

        let search = self.search.to_ascii_lowercase();

        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .column(Column::auto())
            .column(Column::remainder())
            .column(Column::exact(75.0))
            .auto_shrink([true, true])
            .cell_layout(Layout::left_to_right(Align::Center))
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("Name");
                });
                header.col(|ui| {
                    ui.strong("Value");
                });
                header.col(|ui| {
                    ui.strong("Controls");
                });
            })
            .body(|mut body| {
                for (name, ns, prop) in entries {
                    if (ns.is_public() || self.show_private_items)
                        && name.to_ascii_lowercase().contains(&search)
                    {
                        match prop {
                            Property::Slot { slot_id } | Property::ConstSlot { slot_id } => {
                                body.row(18.0, |mut row| {
                                    row.col(|ui| {
                                        ui.label(&name).on_hover_ui(|ui| {
                                            ui.label(format!("{ns:?}"));
                                        });
                                    });
                                    row.col(|ui| {
                                        let value = object.get_slot(slot_id);
                                        ValueWidget::new(activation, value).show(ui, messages);
                                    });
                                    row.col(|_| {});
                                });
                            }
                            Property::Virtual { get: Some(get), .. } => {
                                let key = (ns.as_uri().to_string(), name.clone());
                                body.row(18.0, |mut row| {
                                    row.col(|ui| {
                                        ui.label(&name).on_hover_ui(|ui| {
                                            ui.label(format!("{ns:?}"));
                                        });
                                    });
                                    row.col(|ui| {
                                        if self.call_getters {
                                            let value = object.call_method(get, &[], activation);
                                            ValueWidget::new(activation, value).show(ui, messages);
                                        } else {
                                            let value = self.getter_values.get_mut(&key);
                                            if let Some(value) = value {
                                                // Empty entry means we want to refresh it,
                                                // so let's do that now
                                                let widget = value.get_or_insert_with(|| {
                                                    let value =
                                                        object.call_method(get, &[], activation);
                                                    ValueWidget::new(activation, value)
                                                });
                                                widget.show(ui, messages);
                                            }
                                        }
                                    });
                                    row.col(|ui| {
                                        if ui.button("Call Getter").clicked() {
                                            self.getter_values.insert(key, None);
                                        }
                                    });
                                });
                            }
                            _ => {}
                        }
                    }
                }
            });
    }
}

#[derive(Debug, Clone)]
enum ValueWidget {
    String(String),
    Object(AVM2ObjectHandle, String),
    Other(Cow<'static, str>),
    Error(String),
}

impl ValueWidget {
    fn new<'gc>(
        activation: &mut Activation<'_, 'gc>,
        value: Result<Value<'gc>, Error<'gc>>,
    ) -> Self {
        match value {
            Ok(Value::Undefined) => ValueWidget::Other(Cow::Borrowed("Undefined")),
            Ok(Value::Null) => ValueWidget::Other(Cow::Borrowed("Null")),
            Ok(Value::Bool(value)) => ValueWidget::Other(Cow::Owned(value.to_string())),
            Ok(Value::Number(value)) => ValueWidget::Other(Cow::Owned(value.to_string())),
            Ok(Value::Integer(value)) => ValueWidget::Other(Cow::Owned(value.to_string())),
            Ok(Value::String(value)) => ValueWidget::String(value.to_string()),
            Ok(Value::Object(value)) => ValueWidget::Object(
                AVM2ObjectHandle::new(&mut activation.context, value),
                object_name(activation.context.gc_context, value),
            ),
            Err(e) => ValueWidget::Error(e.to_string()),
        }
    }

    fn show(&self, ui: &mut Ui, messages: &mut Vec<Message>) {
        match self {
            ValueWidget::String(value) => {
                // Readonly
                TextEdit::singleline(&mut value.as_str()).show(ui);
            }
            ValueWidget::Object(value, name) => {
                if ui.button(name).clicked() {
                    messages.push(Message::TrackAVM2Object(value.clone()));
                }
            }
            ValueWidget::Other(value) => {
                ui.label(value.as_ref());
            }
            ValueWidget::Error(value) => {
                ui.colored_label(ui.style().visuals.error_fg_color, value);
            }
        }
    }
}

fn object_name<'gc>(mc: MutationContext<'gc, '_>, object: Object<'gc>) -> String {
    let name = object
        .instance_of_class_definition()
        .map(|r| Cow::Owned(r.read().name().to_qualified_name(mc).to_string()))
        .unwrap_or(Cow::Borrowed("Object"));
    format!("{} {:p}", name, object.as_ptr())
}
