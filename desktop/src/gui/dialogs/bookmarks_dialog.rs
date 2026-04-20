use crate::gui::widgets::path_or_url_field::PathOrUrlField;
use crate::gui::{FilePicker, LocalizableText, text};
use crate::preferences::GlobalPreferences;
use crate::{custom_event::RuffleEvent, player::LaunchOptions};
use egui::{Align2, Button, Grid, Label, Layout, Sense, Ui, Widget, Window};
use egui_extras::{Column, TableBuilder};
use ruffle_frontend_utils::bookmarks::Bookmark;
use ruffle_frontend_utils::content::ContentDescriptor;
use unic_langid::LanguageIdentifier;
use winit::event_loop::EventLoopProxy;

pub struct BookmarkAddDialog {
    preferences: GlobalPreferences,
    name: String,
    url: PathOrUrlField,
}

impl BookmarkAddDialog {
    pub fn new(
        preferences: GlobalPreferences,
        content_descriptor: Option<ContentDescriptor>,
        picker: FilePicker,
    ) -> Self {
        Self {
            preferences,
            name: content_descriptor
                .as_ref()
                .map(|desc| &desc.url)
                .map(|url| ruffle_frontend_utils::url_to_readable_name(url).into_owned())
                .unwrap_or_default(),
            // TODO: Hint.
            url: PathOrUrlField::new(
                content_descriptor,
                LocalizableText::NonLocalizedText("".into()),
                picker,
            ),
        }
    }

    fn is_valid(&self) -> bool {
        self.url.result().is_some() && !self.name.is_empty()
    }

    pub fn show(&mut self, locale: &LanguageIdentifier, egui_ctx: &egui::Context) -> bool {
        let mut keep_open = true;
        let mut should_close = false;

        Window::new(text(locale, "bookmark-dialog-add"))
            .open(&mut keep_open)
            .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .collapsible(false)
            .resizable(false)
            .show(egui_ctx, |ui| {
                Grid::new("bookmarks-dialog-add-grid")
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label(text(locale, "bookmarks-dialog-name"));
                        ui.text_edit_singleline(&mut self.name);
                        ui.end_row();

                        ui.label(text(locale, "bookmarks-dialog-location"));
                        self.url.ui(locale, ui);
                        ui.end_row();
                    });

                ui.separator();
                ui.horizontal(|ui| {
                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add_enabled(self.is_valid(), Button::new(text(locale, "save")))
                            .clicked()
                        {
                            should_close = true;

                            if let Err(e) = self.preferences.write_bookmarks(|writer| {
                                writer.add(Bookmark {
                                    name: self.name.clone(),
                                    content_descriptor: self
                                        .url
                                        .result()
                                        .expect("is_valid() ensured value exists"),
                                })
                            }) {
                                tracing::warn!("Couldn't update bookmarks: {e}");
                            }
                        }

                        if Button::new(text(locale, "cancel")).ui(ui).clicked() {
                            should_close = true;
                        }
                    });
                });
            });

        keep_open && !should_close
    }
}

struct SelectedBookmark {
    index: usize,
    name: String,
    url: PathOrUrlField,
}

pub struct BookmarksDialog {
    event_loop: EventLoopProxy<RuffleEvent>,
    picker: FilePicker,
    preferences: GlobalPreferences,
    selected_bookmark: Option<SelectedBookmark>,
}

impl BookmarksDialog {
    pub fn new(
        preferences: GlobalPreferences,
        picker: FilePicker,
        event_loop: EventLoopProxy<RuffleEvent>,
    ) -> Self {
        Self {
            picker,
            event_loop,
            preferences,
            selected_bookmark: None,
        }
    }

    pub fn show(&mut self, locale: &LanguageIdentifier, egui_ctx: &egui::Context) -> bool {
        let mut keep_open = true;
        let mut should_close = false;

        Window::new(text(locale, "bookmarks-dialog"))
            .open(&mut keep_open)
            .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .collapsible(false)
            .resizable(false)
            .default_width(600.0)
            .default_height(400.0)
            .show(egui_ctx, |ui| {
                egui::TopBottomPanel::top("bookmark-dialog-top-panel")
                    .resizable(true)
                    .min_height(100.0)
                    .show_inside(ui, |ui| {
                        if self.preferences.have_bookmarks() {
                            should_close = self.show_bookmark_table(locale, ui);
                        } else {
                            ui.centered_and_justified(|ui| {
                                ui.label(text(locale, "bookmarks-dialog-no-bookmarks"));
                            });
                        }
                    });

                self.show_bookmark_panel(locale, ui);
            });

        keep_open && !should_close
    }

    fn show_bookmark_table(&mut self, locale: &LanguageIdentifier, ui: &mut Ui) -> bool {
        let text_height = egui::TextStyle::Body
            .resolve(ui.style())
            .size
            .max(ui.spacing().interact_size.y);

        enum BookmarkAction {
            Remove(usize),
            Start(ContentDescriptor),
        }

        let mut action = None;

        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .column(Column::auto())
            .column(Column::remainder())
            .sense(Sense::click())
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong(text(locale, "bookmarks-dialog-name"));
                });
                header.col(|ui| {
                    ui.strong(text(locale, "bookmarks-dialog-location"));
                });
            })
            .body(|mut body| {
                self.preferences.bookmarks(|bookmarks| {
                    for (index, bookmark) in bookmarks.iter().enumerate() {
                        if bookmark.is_invalid() {
                            continue;
                        }

                        body.row(text_height, |mut row| {
                            if let Some(selected) = &self.selected_bookmark {
                                row.set_selected(index == selected.index);
                            }

                            row.col(|ui| {
                                ui.add(
                                    Label::new(&bookmark.name)
                                        .selectable(false)
                                        .wrap_mode(egui::TextWrapMode::Extend),
                                );
                            });
                            row.col(|ui| {
                                ui.add(
                                    Label::new(bookmark.content_descriptor.url.as_str())
                                        .selectable(false)
                                        .wrap_mode(egui::TextWrapMode::Extend),
                                );
                            });

                            let response = row.response();
                            response.context_menu(|ui| {
                                if ui.button(text(locale, "start")).clicked() {
                                    ui.close();
                                    action = Some(BookmarkAction::Start(
                                        bookmark.content_descriptor.clone(),
                                    ))
                                }
                                if ui.button(text(locale, "remove")).clicked() {
                                    ui.close();
                                    action = Some(BookmarkAction::Remove(index));
                                }
                            });
                            if response.clicked() {
                                self.selected_bookmark = Some(SelectedBookmark {
                                    index,
                                    // TODO: Hint.
                                    name: bookmark.name.clone(),
                                    url: PathOrUrlField::new(
                                        Some(bookmark.content_descriptor.clone()),
                                        LocalizableText::NonLocalizedText("".into()),
                                        self.picker.clone(),
                                    ),
                                });
                            }
                            if response.double_clicked() {
                                action = Some(BookmarkAction::Start(
                                    bookmark.content_descriptor.clone(),
                                ));
                            }
                        });
                    }
                });
            });

        match action {
            Some(BookmarkAction::Remove(index)) => {
                if let Err(e) = self.preferences.write_bookmarks(|writer| {
                    // TODO: Recalculate the index for the selected bookmark, if it survives, otherwise just set to None.
                    self.selected_bookmark = None;
                    writer.remove(index);
                }) {
                    tracing::warn!("Couldn't update bookmarks: {e}");
                }
                false
            }
            Some(BookmarkAction::Start(content_descriptor)) => {
                let _ = self.event_loop.send_event(RuffleEvent::Open(
                    content_descriptor,
                    Box::new(LaunchOptions::from(&self.preferences)),
                ));
                true
            }
            None => false,
        }
    }

    fn show_bookmark_panel(&mut self, locale: &LanguageIdentifier, ui: &mut Ui) {
        if let Some(bookmark) = &mut self.selected_bookmark {
            Grid::new("bookmarks-dialog-panel-grid")
                .num_columns(2)
                .show(ui, |ui| {
                    ui.label(text(locale, "bookmarks-dialog-name"));
                    if ui.text_edit_singleline(&mut bookmark.name).lost_focus()
                        && let Err(e) = self.preferences.write_bookmarks(|writer| {
                            writer.set_name(bookmark.index, bookmark.name.clone());
                        })
                    {
                        tracing::warn!("Couldn't update bookmarks: {e}");
                    }
                    ui.end_row();

                    // TODO Do not ignore root content directory.
                    let previous_desc = bookmark.url.result();

                    ui.label(text(locale, "bookmarks-dialog-location"));
                    let current_desc = bookmark.url.ui(locale, ui).result();

                    // TODO: Change the UrlOrPathField widget to return a response instead, so we can update when we lose the focus, removes the need to clone every redraw.
                    if previous_desc != current_desc
                        && let Some(content_descriptor) = current_desc
                        && let Err(e) = self.preferences.write_bookmarks(|writer| {
                            writer.set_content_descriptor(bookmark.index, content_descriptor);
                        })
                    {
                        tracing::warn!("Couldn't update bookmarks: {e}");
                    }

                    ui.end_row();
                });
        } else {
            ui.vertical_centered_justified(|ui| {
                ui.label(text(locale, "bookmarks-dialog-not-selected"));
            });
        }
    }
}
