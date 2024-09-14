use crate::character::Character;
use crate::context::UpdateContext;
use crate::debug_ui::{ItemToSave, Message};
use crate::tag_utils::SwfMovie;
use egui::{CollapsingHeader, Grid, Id, TextEdit, Ui, Window};
use std::sync::Arc;
use swf::CharacterId;
use url::Url;

#[derive(Debug, Eq, PartialEq, Hash, Default, Copy, Clone)]
enum Panel {
    #[default]
    Information,
    Characters,
}

#[derive(Debug, Default)]
pub struct MovieListWindow {}

impl MovieListWindow {
    pub fn show(
        &mut self,
        egui_ctx: &egui::Context,
        context: &mut UpdateContext,
        messages: &mut Vec<Message>,
    ) -> bool {
        let mut keep_open = true;

        Window::new("Known Movie List")
            .open(&mut keep_open)
            .scroll([true, true])
            .show(egui_ctx, |ui| {
                let movies = context.library.known_movies();

                Grid::new("known_movie_list").num_columns(3).show(ui, |ui| {
                    ui.strong("Name");
                    ui.strong("URL");
                    ui.strong("AVM");
                    ui.strong("Size");
                    ui.strong("Save");
                    ui.end_row();

                    for movie in movies {
                        open_movie_button(ui, &movie, messages);
                        ui.label(movie.url());
                        if movie.is_action_script_3() {
                            ui.label("AVM 2");
                        } else {
                            ui.label("AVM 1");
                        }
                        ui.label(movie.uncompressed_len().to_string());
                        if movie.data().is_empty() {
                            ui.weak("(Empty)");
                        } else if ui.button("Save File...").clicked() {
                            save_swf(&movie, messages);
                        }
                        ui.end_row();
                    }
                });
            });
        keep_open
    }
}

#[derive(Debug, Default)]
pub struct MovieWindow {
    open_panel: Panel,
    character_search: String,
}

impl MovieWindow {
    pub fn show(
        &mut self,
        egui_ctx: &egui::Context,
        context: &mut UpdateContext,
        movie: Arc<SwfMovie>,
        messages: &mut Vec<Message>,
    ) -> bool {
        let mut keep_open = true;

        Window::new(movie_name(&movie))
            .id(Id::new(Arc::as_ptr(&movie)))
            .open(&mut keep_open)
            .scroll([true, true])
            .show(egui_ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.open_panel, Panel::Information, "Information");

                    if let Some(library) = context.library.library_for_movie(movie.clone()) {
                        if !library.characters().is_empty() {
                            ui.selectable_value(
                                &mut self.open_panel,
                                Panel::Characters,
                                "Characters",
                            );
                        }
                    }
                });
                ui.separator();

                match self.open_panel {
                    Panel::Information => self.show_information(ui, &movie, messages),
                    Panel::Characters => self.show_characters(ui, context, &movie),
                }
            });
        keep_open
    }

    fn show_characters(&mut self, ui: &mut Ui, context: &mut UpdateContext, movie: &Arc<SwfMovie>) {
        // Cloned up here so we can still use context afterwards
        let (characters, export_characters) = context
            .library
            .library_for_movie(movie.clone())
            .map(|l| (l.characters().clone(), l.export_characters().clone()))
            .unwrap_or_default();

        TextEdit::singleline(&mut self.character_search)
            .hint_text("Search for name or ID")
            .show(ui);

        Grid::new(ui.id().with("characters"))
            .num_columns(3)
            .show(ui, |ui| {
                let mut sorted_keys: Vec<CharacterId> = characters.keys().cloned().collect();
                sorted_keys.sort();

                for id in sorted_keys {
                    let character = characters
                        .get(&id)
                        .expect("Value must exist as we're iterating known keys");

                    let name = export_characters
                        .iter()
                        .find_map(|(k, v)| if *v == id { Some(k) } else { None })
                        .unwrap_or_default()
                        .to_string();

                    let search = self.character_search.to_ascii_lowercase();
                    if !id.to_string().to_ascii_lowercase().contains(&search)
                        && !name.to_ascii_lowercase().contains(&search)
                    {
                        continue;
                    }

                    ui.label(id.to_string());
                    open_character_button(ui, character);
                    ui.label(name);

                    ui.end_row();
                }
            });
    }

    fn show_information(
        &mut self,
        ui: &mut Ui,
        movie: &Arc<SwfMovie>,
        messages: &mut Vec<Message>,
    ) {
        if !movie.data().is_empty() && ui.button("Save File...").clicked() {
            save_swf(movie, messages);
        }

        Grid::new(ui.id().with("information"))
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("URL");
                ui.label(movie.url());
                ui.end_row();

                ui.label("Loader URL");
                ui.label(movie.loader_url().unwrap_or_default());
                ui.end_row();

                ui.label("Version");
                ui.label(movie.version().to_string());
                ui.end_row();

                ui.label("Encoding");
                ui.label(movie.encoding().name());
                ui.end_row();

                ui.label("Size");
                ui.label(format!("{} x {}", movie.width(), movie.height()));
                ui.end_row();

                ui.label("Frame Rate");
                ui.label(movie.frame_rate().to_string());
                ui.end_row();

                ui.label("Compression");
                ui.label(format!("{:?}", movie.header().compression()));
                ui.end_row();

                ui.label("Length");
                ui.label(format!(
                    "{} bytes ({} compressed)",
                    movie.uncompressed_len(),
                    movie.compressed_len()
                ));
                ui.end_row();

                ui.label("Num Frames");
                ui.label(movie.num_frames().to_string());
                ui.end_row();

                ui.label("Flags");
                ui.add_enabled_ui(false, |ui| {
                    ui.vertical(|ui| {
                        if movie.header().is_action_script_3() {
                            ui.label("Uses Actionscript 3");
                        }
                        if movie.header().has_metadata() {
                            ui.label("Has XMP Metadata");
                        }
                        if movie.header().use_direct_blit() {
                            ui.label("Use Direct Blit");
                        }
                        if movie.header().use_gpu() {
                            ui.label("Use GPU");
                        }
                        if movie.header().use_network_sandbox() {
                            ui.label("Use Network Sandbox");
                        }
                    })
                });
                ui.end_row();
            });

        if !movie.parameters().is_empty() {
            CollapsingHeader::new("Parameters")
                .id_salt(ui.id().with("parameters"))
                .default_open(false)
                .show(ui, |ui| {
                    Grid::new(ui.id().with("parameters"))
                        .num_columns(2)
                        .show(ui, |ui| {
                            for (key, value) in movie.parameters() {
                                ui.text_edit_singleline(&mut key.as_str());
                                ui.text_edit_singleline(&mut value.as_str());
                                ui.end_row();
                            }
                        });
                });
        }
    }
}

pub fn movie_name(movie: &Arc<SwfMovie>) -> String {
    format!("SWF {:p}", Arc::as_ptr(movie))
}

pub fn open_movie_button(ui: &mut Ui, movie: &Arc<SwfMovie>, messages: &mut Vec<Message>) {
    if ui.button(movie_name(movie)).clicked() {
        messages.push(Message::TrackMovie(movie.clone()));
    }
}

pub fn open_character_button(ui: &mut Ui, character: &Character) {
    let name = match character {
        Character::EditText(_) => "EditText",
        Character::Graphic(_) => "Graphic",
        Character::MovieClip(_) => "MovieClip",
        Character::Bitmap { .. } => "Bitmap",
        Character::Avm1Button(_) => "Avm1Button",
        Character::Avm2Button(_) => "Avm2Button",
        Character::Font(_) => "Font",
        Character::MorphShape(_) => "MorphShape",
        Character::Text(_) => "Text",
        Character::Sound(_) => "Sound",
        Character::Video(_) => "Video",
        Character::BinaryData(_) => "BinaryData",
    };
    ui.label(name);
}

fn save_swf(movie: &Arc<SwfMovie>, messages: &mut Vec<Message>) {
    let suggested_name = if let Ok(url) = Url::parse(movie.url()) {
        url.path_segments()
            .and_then(|segments| segments.last())
            .map(|str| str.to_string())
    } else {
        None
    };
    let mut data = Vec::new();
    if let Err(e) =
        swf::write::write_swf_raw_tags(movie.header().swf_header(), movie.data(), &mut data)
    {
        tracing::error!("Couldn't write swf: {e}");
    } else {
        messages.push(Message::SaveFile(ItemToSave {
            suggested_name: suggested_name
                .unwrap_or_else(|| format!("{:p}.swf", Arc::as_ptr(movie))),
            data,
        }));
    }
}
