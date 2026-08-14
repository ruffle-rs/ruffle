use crate::avm2::Class;
use crate::context::UpdateContext;
use crate::debug_ui::Message;
use crate::debug_ui::handle::ClassHandle;
use crate::debug_ui::movie::open_movie_button;
use egui::{Grid, Id, Ui, Window};
use gc_arena::Mutation;

#[derive(Debug, Eq, PartialEq, Hash, Default, Copy, Clone)]
enum Panel {
    #[default]
    Class,
    Superclasses,
}

#[derive(Debug, Default)]
pub struct Avm2ClassWindow {
    open_panel: Panel,
}

impl Avm2ClassWindow {
    pub fn show<'gc>(
        &mut self,
        egui_ctx: &egui::Context,
        context: &mut UpdateContext<'gc>,
        class: Class<'gc>,
        messages: &mut Vec<Message>,
    ) -> bool {
        let mut keep_open = true;
        Window::new(class_name(context.gc(), class))
            .id(Id::new(class.as_ptr()))
            .open(&mut keep_open)
            .scroll([true, true])
            .show(egui_ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.open_panel, Panel::Class, "Class Info");
                    ui.selectable_value(
                        &mut self.open_panel,
                        Panel::Superclasses,
                        "Superclasses and Interfaces",
                    );
                });
                ui.separator();

                match self.open_panel {
                    Panel::Class => self.show_information(class, messages, context, ui),
                    Panel::Superclasses => self.show_superclasses(class, messages, context, ui),
                }
            });
        keep_open
    }

    fn show_information<'gc>(
        &mut self,
        class: Class<'gc>,
        messages: &mut Vec<Message>,
        context: &mut UpdateContext<'gc>,
        ui: &mut Ui,
    ) {
        Grid::new(ui.id().with("class"))
            .num_columns(2)
            .striped(true)
            .spacing([8.0, 8.0])
            .show(ui, |ui| {
                let name = class.name();

                ui.label("Namespace");
                let namespace = name.namespace().as_uri(&mut context.strings);
                ui.text_edit_singleline(&mut namespace.to_string().as_str());
                ui.end_row();

                ui.label("Name");
                ui.text_edit_singleline(&mut name.local_name().to_string().as_str());
                ui.end_row();

                if let Some(tu) = class.translation_unit() {
                    ui.label("Movie");
                    open_movie_button(ui, &tu.movie(), messages);
                    ui.end_row();
                }
            });
    }

    fn show_superclasses<'gc>(
        &mut self,
        class: Class<'gc>,
        messages: &mut Vec<Message>,
        context: &mut UpdateContext<'gc>,
        ui: &mut Ui,
    ) {
        Grid::new(ui.id().with("class"))
            .num_columns(2)
            .striped(true)
            .spacing([8.0, 8.0])
            .show(ui, |ui| {
                ui.label("Super Chain");
                ui.vertical(|ui| {
                    let mut superclass = Some(class);
                    while let Some(class) = superclass {
                        show_avm2_class(ui, context, class, messages);
                        superclass = class.super_class();
                    }
                });
                ui.end_row();

                ui.label("Interfaces");
                ui.vertical(|ui| {
                    for interface in class.all_interfaces() {
                        show_avm2_class(ui, context, *interface, messages);
                    }
                });
                ui.end_row();
            });
    }
}

#[derive(Debug, Clone)]
struct ClassWidget(ClassHandle, String);

impl ClassWidget {
    fn new<'gc>(context: &mut UpdateContext<'gc>, class: Class<'gc>) -> Self {
        let name = class_name(context.gc(), class);

        ClassWidget(ClassHandle::new(context, class), name)
    }

    fn show(&self, ui: &mut Ui, messages: &mut Vec<Message>) {
        if ui.button(&self.1).clicked() {
            messages.push(Message::TrackAVM2Class(self.0.clone()));
        }
    }
}

pub fn show_avm2_class<'gc>(
    ui: &mut Ui,
    context: &mut UpdateContext<'gc>,
    class: Class<'gc>,
    messages: &mut Vec<Message>,
) {
    ClassWidget::new(context, class).show(ui, messages)
}

pub fn class_name<'gc>(mc: &Mutation<'gc>, class: Class<'gc>) -> String {
    class.name().to_qualified_name_err_message(mc).to_string()
}
