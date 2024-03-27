use crate::log::FilenamePattern;
use crate::preferences::{Bookmark, BookmarksAndDocument, PreferencesAndDocument};
use ruffle_render_wgpu::clap::{GraphicsBackend, PowerPreference};
use toml_edit::{array, value, Table};
use unic_langid::LanguageIdentifier;

pub struct PreferencesWriter<'a>(&'a mut PreferencesAndDocument);

impl<'a> PreferencesWriter<'a> {
    pub(super) fn new(preferences: &'a mut PreferencesAndDocument) -> Self {
        Self(preferences)
    }

    pub fn set_graphics_backend(&mut self, backend: GraphicsBackend) {
        self.0.toml_document["graphics_backend"] = value(backend.as_str());
        self.0.values.graphics_backend = backend;
    }

    pub fn set_graphics_power_preference(&mut self, preference: PowerPreference) {
        self.0.toml_document["graphics_power_preference"] = value(preference.as_str());
        self.0.values.graphics_power_preference = preference;
    }

    pub fn set_language(&mut self, language: LanguageIdentifier) {
        self.0.toml_document["language"] = value(language.to_string());
        self.0.values.language = language;
    }

    pub fn set_output_device(&mut self, name: Option<String>) {
        if let Some(name) = &name {
            self.0.toml_document["output_device"] = value(name);
        } else {
            self.0.toml_document.remove("output_device");
        }
        self.0.values.output_device = name;
    }

    pub fn set_mute(&mut self, mute: bool) {
        self.0.toml_document["mute"] = value(mute);
        self.0.values.mute = mute;
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.0.toml_document["volume"] = value(volume as f64);
        self.0.values.volume = volume;
    }

    pub fn set_log_filename_pattern(&mut self, pattern: FilenamePattern) {
        self.0.toml_document["log"]["filename_pattern"] = value(pattern.as_str());
        self.0.values.log.filename_pattern = pattern;
    }
}

pub struct BookmarksWriter<'a>(&'a mut BookmarksAndDocument);

impl<'a> BookmarksWriter<'a> {
    pub(super) fn new(bookmarks: &'a mut BookmarksAndDocument) -> Self {
        Self(bookmarks)
    }

    pub fn add(&mut self, bookmark: Bookmark) {
        // TODO: if more fields are added, this should use URL matching (e.g. other properties are ignored)
        if !self.0.values.contains(&bookmark) {
            if let Some(array) = self.0.toml_document["bookmark"]
                .or_insert(array())
                .as_array_of_tables_mut()
            {
                // TODO: If we add a BookmarkWriter use this here instead rather than duplicating the table write code.
                let mut table = Table::new();
                table["url"] = value(bookmark.url.to_string());
                array.push(table);
                self.0.values.push(bookmark);
            } else {
                // TODO: There is definitely a better way to handle this, then just logging a warning.
                tracing::warn!("bookmark is not an array of tables, bookmarks will NOT be saved.");
            }
        }
    }

    pub fn remove(&mut self, index: usize) {
        // We need to get the URL to find the bookmark in the TOML file, since index may not correspond to
        // the same table entry (i.e. invalid tables and such that we want to keep intact for compatibility purposes)
        let bookmark = self.0.values.remove(index);

        // Remove the bookmark from the TOML file.
        if let Some(array) = self.0.toml_document["bookmark"]
            .or_insert(array())
            .as_array_of_tables_mut()
        {
            let bookmark_url = bookmark.url.to_string();
            array.retain(|x| {
                if let Some(url) = x.get("url").and_then(|x| x.as_str()) {
                    return url != bookmark_url;
                }

                true
            });
        } else {
            // TODO: We should add a way to return an error from write methods.
            tracing::warn!("bookmark is not an array of tables, bookmarks will NOT be saved.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fluent_templates::loader::langid;

    macro_rules! define_serialization_test_helpers {
        ($read_method:ident, $doc_struct:ident, $writer:ident) => {
            fn parse(input: &str) -> $doc_struct {
                let (result, document) = $read_method(input);
                $doc_struct {
                    toml_document: document,
                    values: result.result,
                }
            }

            fn check_roundtrip(preferences: &mut $doc_struct) {
                let read_result = $read_method(&preferences.toml_document.to_string());
                assert_eq!(
                    preferences.values, read_result.0.result,
                    "roundtrip failed: expected != actual"
                );
            }

            fn test(original: &str, fun: impl FnOnce(&mut $writer), expected: &str) {
                let mut preferences = parse(original);
                let mut writer = $writer::new(&mut preferences);
                fun(&mut writer);
                check_roundtrip(&mut preferences);
                assert_eq!(expected, preferences.toml_document.to_string());
            }
        };
    }

    mod preferences {
        use super::*;
        use crate::preferences::read::read_preferences;

        define_serialization_test_helpers!(
            read_preferences,
            PreferencesAndDocument,
            PreferencesWriter
        );

        #[test]
        fn set_graphics_backend() {
            test(
                "",
                |writer| writer.set_graphics_backend(GraphicsBackend::Default),
                "graphics_backend = \"default\"\n",
            );

            test(
                "graphics_backend = \"fast\"",
                |writer| writer.set_graphics_backend(GraphicsBackend::Vulkan),
                "graphics_backend = \"vulkan\"\n",
            );
        }

        #[test]
        fn set_graphics_power_preference() {
            test(
                "",
                |writer| writer.set_graphics_power_preference(PowerPreference::High),
                "graphics_power_preference = \"high\"\n",
            );

            test(
                "graphics_power_preference = \"fast\"",
                |writer| writer.set_graphics_power_preference(PowerPreference::Low),
                "graphics_power_preference = \"low\"\n",
            );
        }

        #[test]
        fn set_language() {
            test(
                "",
                |writer| writer.set_language(langid!("en-US")),
                "language = \"en-US\"\n",
            );

            test(
                "language = \"???\"",
                |writer| writer.set_language(langid!("en-Latn-US-valencia")),
                "language = \"en-Latn-US-valencia\"\n",
            );
        }

        #[test]
        fn set_output_device() {
            test(
                "",
                |writer| writer.set_output_device(Some("Speakers".to_string())),
                "output_device = \"Speakers\"\n",
            );

            test(
                "output_device = \"Speakers\"",
                |writer| writer.set_output_device(None),
                "",
            );
        }

        #[test]
        fn set_volume() {
            test("", |writer| writer.set_volume(0.5), "volume = 0.5\n");
        }

        #[test]
        fn set_mute() {
            test("", |writer| writer.set_mute(true), "mute = true\n");
            test(
                "mute = true",
                |writer| writer.set_mute(false),
                "mute = false\n",
            );
        }

        #[test]
        fn set_log_filename_pattern() {
            test(
                "",
                |writer| writer.set_log_filename_pattern(FilenamePattern::WithTimestamp),
                "log = { filename_pattern = \"with_timestamp\" }\n",
            );
            test(
                "log = { filename_pattern = \"with_timestamp\" }\n",
                |writer| writer.set_log_filename_pattern(FilenamePattern::SingleFile),
                "log = { filename_pattern = \"single_file\" }\n",
            );
            test(
                "[log]\nfilename_pattern = \"with_timestamp\"\n",
                |writer| writer.set_log_filename_pattern(FilenamePattern::SingleFile),
                "[log]\nfilename_pattern = \"single_file\"\n",
            );
        }
    }

    #[allow(clippy::unwrap_used)]
    mod bookmarks {
        use super::*;
        use crate::preferences::read::read_bookmarks;
        use std::str::FromStr;

        define_serialization_test_helpers!(read_bookmarks, BookmarksAndDocument, BookmarksWriter);

        #[test]
        fn add_bookmark() {
            test(
                "",
                |writer| {
                    writer.add(Bookmark {
                        url: url::Url::from_str("file:///home/user/example.swf").unwrap(),
                    })
                },
                "[[bookmark]]\nurl = \"file:///home/user/example.swf\"\n",
            );
            test("[[bookmark]]\nurl = \"file:///home/user/example.swf\"\n", |writer| writer.add(Bookmark {
            url: url::Url::from_str("file:///home/user/another_file.swf").unwrap()
        }), "[[bookmark]]\nurl = \"file:///home/user/example.swf\"\n\n[[bookmark]]\nurl = \"file:///home/user/another_file.swf\"\n");
        }

        #[test]
        fn remove_bookmark() {
            test(
            "[[bookmark]]\nurl = \"file://home/user/example.swf\"\n\n[[bookmark]]\nurl = \"https://ruffle.rs/logo-anim.swf\"\n\n[[bookmark]]\nurl = \"file:///another_file.swf\"\n",
            |writer| {
                writer.remove(1);
            },
            "[[bookmark]]\nurl = \"file://home/user/example.swf\"\n\n[[bookmark]]\nurl = \"file:///another_file.swf\"\n",
        );

            // Test that we leave invalid bookmark tables intact when removing a bookmark.
            test("[[bookmark]]\nurl = \"file://home/user/example.swf\"\n\n[[bookmark]]\n\n[[bookmark]]\nurl = \"https://ruffle.rs/logo-anim.swf\"\n\n[[bookmark]]\nurl = \"invalid\"\n", |writer| {
            writer.remove(1);
        }, "[[bookmark]]\nurl = \"file://home/user/example.swf\"\n\n[[bookmark]]\n\n[[bookmark]]\nurl = \"invalid\"\n")
        }
    }
}
