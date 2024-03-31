use crate::gui::text;
use crate::preferences::GlobalPreferences;
use egui::{Align2, Button, Grid, Widget, Window};
use unic_langid::LanguageIdentifier;

pub struct BookmarksDialog {
    preferences: GlobalPreferences,
}

impl BookmarksDialog {
    pub fn new(preferences: GlobalPreferences) -> Self {
        Self { preferences }
    }

    pub fn show(&mut self, locale: &LanguageIdentifier, egui_ctx: &egui::Context) -> bool {
        let mut keep_open = true;
        let mut should_close = false;

        Window::new(text(locale, "bookmarks-dialog"))
            .open(&mut keep_open)
            .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .collapsible(false)
            .resizable(false)
            .show(egui_ctx, |ui| {
                Grid::new("bookmarks-dialog-grid")
                    .num_columns(2)
                    .striped(true)
                    .show(ui, |ui| {
                        enum BookmarkAction {
                            Remove(usize),
                        }

                        let mut action = None;

                        self.preferences.bookmarks(|bookmarks| {
                            // Close the dialog if we have no bookmarks to show.
                            should_close = bookmarks.is_empty();

                            for (index, bookmark) in
                                bookmarks.iter().filter(|x| !x.is_invalid()).enumerate()
                            {
                                ui.label(bookmark.url.as_str());

                                if Button::new(text(locale, "remove")).ui(ui).clicked() {
                                    action = Some(BookmarkAction::Remove(index));
                                }

                                ui.end_row();
                            }
                        });

                        if let Some(action) = action {
                            if let Err(e) =
                                self.preferences.write_bookmarks(|writer| match action {
                                    BookmarkAction::Remove(index) => writer.remove(index),
                                })
                            {
                                tracing::warn!("Couldn't update bookmarks: {e}");
                            }
                        }
                    });
            });

        keep_open && !should_close
    }
}
