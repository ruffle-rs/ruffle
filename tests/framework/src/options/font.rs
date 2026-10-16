use ruffle_core::Player;
use ruffle_core::font::{DefaultFont, FontQuery, FontType};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Default, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct FontOptions {
    pub family: String,
    pub path: String,
    pub bold: bool,
    pub italic: bool,
}

impl FontOptions {
    pub fn to_font_query(&self) -> FontQuery {
        FontQuery::new(
            FontType::Device,
            self.family.clone(),
            self.bold,
            self.italic,
        )
    }
}

#[derive(Deserialize, Default, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct FontSortOptions {
    pub family: String,
    pub bold: bool,
    pub italic: bool,
    pub sort: Vec<String>,
}

#[derive(Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct DefaultFontsOptions {
    pub sans: Vec<String>,
    pub serif: Vec<String>,
    pub typewriter: Vec<String>,
    pub japanese_gothic: Vec<String>,
    pub japanese_gothic_mono: Vec<String>,
    pub japanese_mincho: Vec<String>,
}

impl DefaultFontsOptions {
    pub fn to_map(&self) -> HashMap<DefaultFont, Vec<String>> {
        HashMap::from([
            (DefaultFont::Sans, self.sans.clone()),
            (DefaultFont::Serif, self.serif.clone()),
            (DefaultFont::Typewriter, self.typewriter.clone()),
            (DefaultFont::JapaneseGothic, self.japanese_gothic.clone()),
            (
                DefaultFont::JapaneseGothicMono,
                self.japanese_gothic_mono.clone(),
            ),
            (DefaultFont::JapaneseMincho, self.japanese_mincho.clone()),
        ])
    }

    pub fn apply(&self, player: &mut Player) {
        self.apply_default_font(player, DefaultFont::Sans, &self.sans);
        self.apply_default_font(player, DefaultFont::Serif, &self.serif);
        self.apply_default_font(player, DefaultFont::Typewriter, &self.typewriter);
        self.apply_default_font(player, DefaultFont::JapaneseGothic, &self.japanese_gothic);
        self.apply_default_font(
            player,
            DefaultFont::JapaneseGothicMono,
            &self.japanese_gothic_mono,
        );
        self.apply_default_font(player, DefaultFont::JapaneseMincho, &self.japanese_mincho);
    }

    fn apply_default_font(&self, player: &mut Player, font: DefaultFont, names: &[String]) {
        if !names.is_empty() {
            player.set_default_font(font, names.to_owned());
        }
    }
}
