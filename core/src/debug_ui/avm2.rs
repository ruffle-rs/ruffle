use crate::avm2::property::Property;
use crate::avm2::{
    Activation, ArrayStorage, ClassObject, Error, Namespace, Object, TObject, Value,
};
use crate::context::UpdateContext;
use crate::debug_ui::display_object::open_display_object_button;
use crate::debug_ui::handle::{AVM2ObjectHandle, DisplayObjectHandle};
use crate::debug_ui::{ItemToSave, Message};
use egui::{Align, Checkbox, Grid, Id, Layout, TextEdit, Ui, Window};
use egui_extras::{Column, TableBody, TableBuilder, TableRow};
use fnv::FnvHashMap;
use gc_arena::Mutation;
use std::borrow::Cow;

use super::movie::open_movie_button;

#[derive(Debug, Eq, PartialEq, Hash, Default, Copy, Clone)]
enum Panel {
    Information,
    #[default]
    Properties,
    Elements,
    Class,
}

#[derive(Debug, Default)]
pub struct Avm2ObjectWindow {
    hovered_debug_rect: Option<DisplayObjectHandle>,
    show_private_items: bool,
    call_getters: bool,
    getter_values: FnvHashMap<(String, String), Option<ValueResultWidget>>,
    search: String,
    open_panel: Panel,
}

impl Avm2ObjectWindow {
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
        let domain = context.avm2.stage_domain();
        let mut activation = Activation::from_domain(context, domain);
        Window::new(object_name(activation.context.gc_context, object))
            .id(Id::new(object.as_ptr()))
            .open(&mut keep_open)
            .scroll([true, true])
            .show(egui_ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.open_panel, Panel::Information, "Information");
                    ui.selectable_value(&mut self.open_panel, Panel::Properties, "Properties");
                    if object.as_array_storage().is_some() {
                        ui.selectable_value(&mut self.open_panel, Panel::Elements, "Elements");
                    }
                    if object.as_class_object().is_some() {
                        ui.selectable_value(&mut self.open_panel, Panel::Class, "Class Info");
                    }
                });
                ui.separator();

                match self.open_panel {
                    Panel::Information => {
                        self.show_information(object, messages, &mut activation, ui)
                    }
                    Panel::Properties => {
                        self.show_properties(object, messages, &mut activation, ui)
                    }
                    Panel::Elements => {
                        if let Some(array) = object.as_array_storage() {
                            self.show_elements(array, messages, activation.context, ui)
                        }
                    }
                    Panel::Class => {
                        if let Some(class) = object.as_class_object() {
                            self.show_class(class, messages, &mut activation, ui)
                        }
                    }
                }
            });
        keep_open
    }

    fn show_information<'gc>(
        &mut self,
        object: Object<'gc>,
        messages: &mut Vec<Message>,
        activation: &mut Activation<'_, 'gc>,
        ui: &mut Ui,
    ) {
        Grid::new(ui.id().with("info"))
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                if let Some(class) = object.instance_class().class_object() {
                    ui.label("Instance Of");
                    show_avm2_value(ui, activation.context, class.into(), messages);
                    ui.end_row();
                }

                if let Some(object) = object.as_display_object() {
                    ui.label("Display Object");
                    open_display_object_button(
                        ui,
                        activation.context,
                        messages,
                        object,
                        &mut self.hovered_debug_rect,
                    );
                    ui.end_row();
                }

                if let Some(bmd) = object.as_bitmap_data() {
                    ui.label("Bitmap Data Size");
                    ui.label(format!("{} x {}", bmd.width(), bmd.height()));
                    ui.end_row();

                    ui.label("Bitmap Data Status");
                    if bmd.disposed() {
                        ui.label("Disposed");
                    } else {
                        ui.horizontal(|ui| {
                            ui.label("Alive");
                            ui.add_space(10.0);
                            if ui.button("Save File...").clicked() {
                                let mut data = Vec::new();
                                let mut encoder =
                                    png::Encoder::new(&mut data, bmd.width(), bmd.height());
                                encoder.set_color(png::ColorType::Rgba);
                                encoder.set_depth(png::BitDepth::Eight);
                                if let Err(e) = encoder.write_header().and_then(|mut w| {
                                    w.write_image_data(
                                        &bmd.sync(activation.context.renderer).read().pixels_rgba(),
                                    )
                                }) {
                                    tracing::error!("Couldn't create png: {e}");
                                } else {
                                    messages.push(Message::SaveFile(ItemToSave {
                                        suggested_name: format!("{:p}.png", object.as_ptr()),
                                        data,
                                    }));
                                }
                            }
                        });
                    }
                    ui.end_row();

                    ui.label("Bitmap Data Transparency");
                    ui.add_enabled(false, Checkbox::new(&mut bmd.transparency(), "Transparent"));
                    ui.end_row();

                    ui.label("Bitmap Data Sync");
                    ui.label(bmd.debug_sync_status());
                    ui.end_row();
                }

                if let Some(ba) = object.as_bytearray() {
                    ui.label("Byte Array");

                    if ba.len() > 0 {
                        ui.horizontal(|ui| {
                            ui.label(format!("{} bytes", ba.len()));
                            ui.add_space(10.0);
                            if ui.button("Sync & Save PNG...").clicked() {
                                messages.push(Message::SaveFile(ItemToSave {
                                    suggested_name: format!("{:p}.png", object.as_ptr()),
                                    data: ba.bytes().to_vec(),
                                }));
                            }
                        });
                    } else {
                        ui.label("0 bytes");
                    }

                    ui.end_row();
                }
            });
    }

    fn show_elements<'gc>(
        &mut self,
        array: std::cell::Ref<ArrayStorage<'gc>>,
        messages: &mut Vec<Message>,
        context: &mut UpdateContext<'gc>,
        ui: &mut Ui,
    ) {
        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .column(Column::initial(40.0))
            .column(Column::remainder())
            .auto_shrink([true, true])
            .cell_layout(Layout::left_to_right(Align::Center))
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("Index");
                });
                header.col(|ui| {
                    ui.strong("Value");
                });
            })
            .body(|mut body| {
                for (index, value) in array.iter().enumerate() {
                    body.row(18.0, |mut row| {
                        row.col(|ui| {
                            ui.label(index.to_string());
                        });
                        row.col(|ui| {
                            if let Some(value) = value {
                                show_avm2_value(ui, context, value, messages);
                            } else {
                                // Array hole.
                                ui.weak("(Empty)");
                            }
                        });
                    });
                }
            });
    }

    fn show_class<'gc>(
        &mut self,
        class: ClassObject<'gc>,
        messages: &mut Vec<Message>,
        activation: &mut Activation<'_, 'gc>,
        ui: &mut Ui,
    ) {
        Grid::new(ui.id().with("class"))
            .num_columns(2)
            .striped(true)
            .spacing([8.0, 8.0])
            .show(ui, |ui| {
                let definition = class.inner_class_definition();
                let name = definition.name();

                ui.label("Namespace");
                ui.text_edit_singleline(&mut name.namespace().as_uri().to_string().as_str());
                ui.end_row();

                ui.label("Name");
                ui.text_edit_singleline(&mut name.local_name().to_string().as_str());
                ui.end_row();

                if let Some(tuint) = class.translation_unit() {
                    ui.label("Movie");
                    open_movie_button(ui, &tuint.movie(), messages);
                    ui.end_row();
                }

                ui.label("Super Chain");
                ui.vertical(|ui| {
                    let mut superclass = Some(class);
                    while let Some(class) = superclass {
                        show_avm2_value(ui, activation.context, class.into(), messages);
                        superclass = class.superclass_object();
                    }
                });
                ui.end_row();

                ui.label("Interfaces");
                ui.vertical(|ui| {
                    for interface in &*class.inner_class_definition().all_interfaces() {
                        ui.text_edit_singleline(
                            &mut interface
                                .name()
                                .to_qualified_name_err_message(activation.context.gc_context)
                                .to_string()
                                .as_str(),
                        );
                    }
                });
                ui.end_row();
            });
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
        let vtable = object.vtable();

        for (name, ns, prop) in vtable.resolved_traits().iter() {
            entries.push((name.to_string(), ns, *prop));
        }
        entries.sort_by(|a, b| a.0.cmp(&b.0));

        ui.horizontal(|ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.checkbox(&mut self.show_private_items, "Show Private Items");
                ui.checkbox(&mut self.call_getters, "Call Getters");
                ui.add_sized(
                    ui.available_size(),
                    TextEdit::singleline(&mut self.search).hint_text("Search..."),
                );
            });
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
                    if (self.show_private_items || ns.is_public())
                        && name.to_ascii_lowercase().contains(&search)
                    {
                        self.show_property(
                            object, messages, activation, &mut body, &name, ns, prop,
                        );
                    }
                }
            });
    }

    #[allow(clippy::too_many_arguments)]
    fn show_property<'gc>(
        &mut self,
        object: Object<'gc>,
        messages: &mut Vec<Message>,
        activation: &mut Activation<'_, 'gc>,
        body: &mut TableBody,
        name: &str,
        ns: Namespace<'gc>,
        prop: Property,
    ) {
        let label_col = |row: &mut TableRow| {
            row.col(|ui| {
                ui.label(name).on_hover_ui(|ui| {
                    ui.label(format!("{ns:?}"));
                });
            });
        };
        match prop {
            Property::Slot { slot_id } | Property::ConstSlot { slot_id } => {
                body.row(18.0, |mut row| {
                    label_col(&mut row);
                    row.col(|ui| {
                        let value = object.get_slot(slot_id);
                        ValueResultWidget::new(activation, value).show(ui, messages);
                    });
                    row.col(|_| {});
                });
            }
            Property::Virtual { get: Some(get), .. } => {
                let key = (ns.as_uri().to_string(), name.to_string());
                body.row(18.0, |mut row| {
                    label_col(&mut row);
                    row.col(|ui| {
                        if self.call_getters {
                            let value = object.call_method(get, &[], activation);
                            ValueResultWidget::new(activation, value).show(ui, messages);
                        } else {
                            let value = self.getter_values.get_mut(&key);
                            if let Some(value) = value {
                                // Empty entry means we want to refresh it,
                                // so let's do that now
                                let widget = value.get_or_insert_with(|| {
                                    let value = object.call_method(get, &[], activation);
                                    ValueResultWidget::new(activation, value)
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

#[derive(Debug, Clone)]
enum ValueWidget {
    String(String),
    Object(AVM2ObjectHandle, String),
    Other(Cow<'static, str>),
}

impl ValueWidget {
    fn new<'gc>(context: &mut UpdateContext<'gc>, value: Value<'gc>) -> Self {
        match value {
            Value::Undefined => ValueWidget::Other(Cow::Borrowed("Undefined")),
            Value::Null => ValueWidget::Other(Cow::Borrowed("Null")),
            Value::Bool(value) => ValueWidget::Other(Cow::Owned(value.to_string())),
            Value::Number(value) => ValueWidget::Other(Cow::Owned(value.to_string())),
            Value::Integer(value) => ValueWidget::Other(Cow::Owned(value.to_string())),
            Value::String(value) => ValueWidget::String(value.to_string()),
            Value::Object(value) => ValueWidget::Object(
                AVM2ObjectHandle::new(context, value),
                object_name(context.gc_context, value),
            ),
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
        }
    }
}

#[derive(Debug, Clone)]
enum ValueResultWidget {
    Value(ValueWidget),
    Error(String),
}

impl ValueResultWidget {
    fn new<'gc>(
        activation: &mut Activation<'_, 'gc>,
        value: Result<Value<'gc>, Error<'gc>>,
    ) -> Self {
        match value {
            Ok(value) => Self::Value(ValueWidget::new(activation.context, value)),
            Err(error) => Self::Error(format!("{error:?})")),
        }
    }

    fn show(&self, ui: &mut Ui, messages: &mut Vec<Message>) {
        match self {
            Self::Value(value) => {
                value.show(ui, messages);
            }
            Self::Error(error) => {
                ui.colored_label(ui.style().visuals.error_fg_color, error);
            }
        }
    }
}

pub fn show_avm2_value<'gc>(
    ui: &mut Ui,
    context: &mut UpdateContext<'gc>,
    value: Value<'gc>,
    messages: &mut Vec<Message>,
) {
    ValueWidget::new(context, value).show(ui, messages)
}

fn object_name<'gc>(mc: &Mutation<'gc>, object: Object<'gc>) -> String {
    if let Some(class) = object.as_class_object() {
        class
            .inner_class_definition()
            .name()
            .to_qualified_name_err_message(mc)
            .to_string()
    } else {
        let name = object.instance_class().name().local_name().to_string();
        format!("{} {:p}", name, object.as_ptr())
    }
}
