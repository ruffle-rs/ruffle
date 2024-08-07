use egui::{collapsing_header::CollapsingState, CollapsingHeader, TextEdit, Ui, Window};

use crate::{avm2::Domain, context::UpdateContext};

use super::{
    handle::{AVM2ObjectHandle, DomainHandle},
    Message,
};

#[derive(Debug, Default)]
pub struct DomainListWindow {
    search: String,
}

impl DomainListWindow {
    pub fn show(
        &mut self,
        egui_ctx: &egui::Context,
        context: &mut UpdateContext,
        messages: &mut Vec<Message>,
    ) -> bool {
        let mut keep_open = true;

        Window::new("Domain List")
            .open(&mut keep_open)
            .show(egui_ctx, |ui| {
                TextEdit::singleline(&mut self.search)
                    .hint_text("Search")
                    .show(ui);
                ui.add_space(10.0);
                // Let's search ascii-insensitive for QOL
                let search = self.search.to_ascii_lowercase();
                let domain = context.avm2.playerglobals_domain();
                egui::ScrollArea::both().show(ui, |ui| {
                    self.show_domain(ui, context, domain, messages, &search)
                });
            });
        keep_open
    }

    #[allow(clippy::only_used_in_recursion)]
    pub fn show_domain<'gc>(
        &mut self,
        ui: &mut Ui,
        context: &mut UpdateContext<'gc>,
        domain: Domain<'gc>,
        messages: &mut Vec<Message>,
        search: &str,
    ) {
        CollapsingState::load_with_default_open(ui.ctx(), ui.id().with(domain.as_ptr()), false)
            .show_header(ui, |ui| {
                open_domain_button(ui, context, messages, domain);
            })
            .body(|ui| {
                let class_props = domain.classes();
                let mut classes: Vec<_> = class_props.iter().collect();
                classes.sort_by_key(|(name, _, _)| *name);

                for (_, _, class) in classes {
                    let class_name = class.name().to_qualified_name(context.gc_context);
                    if !class_name.to_string().to_ascii_lowercase().contains(search) {
                        continue;
                    }

                    CollapsingHeader::new(format!("Class {class_name}"))
                        .id_salt(ui.id().with(class.0.as_ptr()))
                        .show(ui, |ui| {
                            for class_obj in &*class.class_objects() {
                                let button = ui.button(format!("{class_obj:?}"));
                                if button.clicked() {
                                    messages.push(Message::TrackAVM2Object(AVM2ObjectHandle::new(
                                        context,
                                        (*class_obj).into(),
                                    )));
                                }
                            }
                        });
                }
                drop(class_props);

                for child_domain in domain.children(context.gc_context) {
                    self.show_domain(ui, context, child_domain, messages, search);
                }
            });
    }
}

pub fn open_domain_button<'gc>(
    ui: &mut Ui,
    context: &mut UpdateContext<'gc>,
    messages: &mut Vec<Message>,
    domain: Domain<'gc>,
) {
    let response = ui.button(format!("Domain {:?}", domain.as_ptr()));
    if response.clicked() {
        messages.push(Message::TrackDomain(DomainHandle::new(context, domain)));
    }
}

#[derive(Debug, Default)]
pub struct DomainWindow {}
