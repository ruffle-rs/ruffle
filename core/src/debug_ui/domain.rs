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
                ui.push_id("domain_search", |ui| {
                    TextEdit::singleline(&mut self.search)
                        .hint_text("Search")
                        .show(ui);
                });

                ui.add_space(10.0);
                // Let's search ascii-insensitive for QOL
                let search = self.search.to_ascii_lowercase();
                let domain = context.avm2.playerglobals_domain();
                egui::ScrollArea::both().show(ui, |ui| {
                    ui.push_id("domain_scroll_content", |ui| {
                        Self::show_domain(ui, context, domain, messages, &search, 0)
                    });
                });
            });
        keep_open
    }

    pub fn show_domain<'gc>(
        ui: &mut Ui,
        context: &mut UpdateContext<'gc>,
        domain: Domain<'gc>,
        messages: &mut Vec<Message>,
        search: &str,
        depth: usize,
    ) {
        let domain_id = format!("domain_{}_{:p}", depth, domain.as_ptr());

        ui.push_id(domain_id, |ui| {
            CollapsingState::load_with_default_open(ui.ctx(), ui.id().with("domain_header"), false)
                .show_header(ui, |ui| {
                    open_domain_button(ui, context, messages, domain);
                })
                .body(|ui| {
                    let class_props = domain.classes();
                    let mut classes: Vec<_> = class_props.iter().collect();
                    classes.sort_by_key(|(name, _, _)| *name);

                    for (class_index, (_, _, class)) in classes.iter().enumerate() {
                        let class_name = class.name().to_qualified_name(context.gc());
                        if !class_name.to_string().to_ascii_lowercase().contains(search) {
                            continue;
                        }

                        let class_id =
                            format!("class_{}_{}_{:p}", depth, class_index, class.as_ptr());

                        ui.push_id(class_id, |ui| {
                            CollapsingHeader::new(format!("Class {class_name}")).show(ui, |ui| {
                                for (obj_index, class_obj) in
                                    class.class_objects().iter().enumerate()
                                {
                                    ui.push_id(
                                        format!("class_obj_{}_{}", class_index, obj_index),
                                        |ui| {
                                            let button = ui.button(format!("{class_obj:?}"));
                                            if button.clicked() {
                                                messages.push(Message::TrackAVM2Object(
                                                    AVM2ObjectHandle::new(
                                                        context,
                                                        (*class_obj).into(),
                                                    ),
                                                ));
                                            }
                                        },
                                    );
                                }
                            });
                        });
                    }

                    drop(class_props);

                    for (child_index, child_domain) in
                        domain.children(context.gc()).into_iter().enumerate()
                    {
                        ui.push_id(format!("child_domain_{}_{}", depth, child_index), |ui| {
                            Self::show_domain(
                                ui,
                                context,
                                child_domain,
                                messages,
                                search,
                                depth + 1,
                            );
                        });
                    }
                });
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
