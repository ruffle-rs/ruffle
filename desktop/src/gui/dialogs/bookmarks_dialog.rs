use crate::gui::text;
use crate::gui::widgets::PathOrUrlField;
use crate::preferences::GlobalPreferences;
use egui::{Align2, Button, Grid, Label, Layout, Sense, Ui, Widget, Window};
use egui_extras::{Column, TableBuilder};
use ruffle_frontend_utils::bookmarks::Bookmark;
use unic_langid::LanguageIdentifier;
use url::Url;

pub struct BookmarkAddDialog {
    preferences: GlobalPreferences,
    name: String,
    url: PathOrUrlField,
}

impl BookmarkAddDialog {
    pub fn new(preferences: GlobalPreferences, initial_url: Option<Url>) -> Self {
        Self {
            preferences,
            name: initial_url
                .as_ref()
                .map(|x| ruffle_frontend_utils::url_to_readable_name(x).into_owned())
                .unwrap_or_default(),
            // TODO: hint.
            url: PathOrUrlField::new(initial_url, ""),
        }
    }

    fn is_valid(&self) -> bool {
        self.url.value().is_some() && !self.name.is_empty()
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
                                    url: self
                                        .url
                                        .value()
                                        .cloned()
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
    preferences: GlobalPreferences,
    selected_bookmark: Option<SelectedBookmark>,
}

impl BookmarksDialog {
    pub fn new(preferences: GlobalPreferences) -> Self {
        Self {
            preferences,
            selected_bookmark: None,
        }
    }

    pub fn show(&mut self, locale: &LanguageIdentifier, egui_ctx: &egui::Context) -> bool {
        let mut keep_open = true;
        let should_close = false;

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
                            self.show_bookmark_table(locale, ui);
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

    fn show_bookmark_table(&mut self, locale: &LanguageIdentifier, ui: &mut Ui) {
        let text_height = egui::TextStyle::Body
            .resolve(ui.style())
            .size
            .max(ui.spacing().interact_size.y);

        enum BookmarkAction {
            Remove(usize),
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
                                    Label::new(bookmark.url.as_str())
                                        .selectable(false)
                                        .wrap_mode(egui::TextWrapMode::Extend),
                                );
                            });

                            let response = row.response();
                            response.context_menu(|ui| {
                                if ui.button(text(locale, "remove")).clicked() {
                                    ui.close_menu();
                                    action = Some(BookmarkAction::Remove(index));
                                }
                            });
                            if response.clicked() {
                                self.selected_bookmark = Some(SelectedBookmark {
                                    index,
                                    // TODO: set hint
                                    name: bookmark.name.clone(),
                                    url: PathOrUrlField::new(Some(bookmark.url.clone()), ""),
                                });
                            }
                        });
                    }
                });
            });

        if let Some(action) = action {
            if let Err(e) = self.preferences.write_bookmarks(|writer| match action {
                BookmarkAction::Remove(index) => {
                    // TODO: Recalculate the index for the selected bookmark, if it survives, otherwise just set to None.
                    self.selected_bookmark = None;
                    writer.remove(index);
                }
            }) {
                tracing::warn!("Couldn't update bookmarks: {e}");
            }
        }
    }

    fn show_bookmark_panel(&mut self, locale: &LanguageIdentifier, ui: &mut Ui) {
        if let Some(bookmark) = &mut self.selected_bookmark {
            Grid::new("bookmarks-dialog-panel-grid")
                .num_columns(2)
                .show(ui, |ui| {
                    ui.label(text(locale, "bookmarks-dialog-name"));
                    if ui.text_edit_singleline(&mut bookmark.name).lost_focus() {
                        if let Err(e) = self.preferences.write_bookmarks(|writer| {
                            writer.set_name(bookmark.index, bookmark.name.clone());
                        }) {
                            tracing::warn!("Couldn't update bookmarks: {e}");
                        }
                    }
                    ui.end_row();

                    let previous_url = bookmark.url.value().cloned();

                    ui.label(text(locale, "bookmarks-dialog-location"));
                    let current_url = bookmark.url.ui(locale, ui).value();

                    // TOOD: Change the UrlOrPathField widget to return a response instead, so we can update when we lose the focus, removes the need to clone every redraw.
                    if previous_url.as_ref() != current_url {
                        if let Some(url) = current_url {
                            if let Err(e) = self.preferences.write_bookmarks(|writer| {
                                writer.set_url(bookmark.index, url.clone());
                            }) {
                                tracing::warn!("Couldn't update bookmarks: {e}");
                            }
                        }
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
