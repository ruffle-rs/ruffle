use crate::log::FilenamePattern;
use crate::preferences::storage::StorageBackend;
use crate::preferences::SavedGlobalPreferences;
use ruffle_frontend_utils::bookmarks::{Bookmark, Bookmarks};
use ruffle_frontend_utils::parse::DocumentHolder;
use ruffle_render_wgpu::clap::{GraphicsBackend, PowerPreference};
use toml_edit::{array, value, ArrayOfTables, DocumentMut, Table};
use unic_langid::LanguageIdentifier;

pub struct PreferencesWriter<'a>(&'a mut DocumentHolder<SavedGlobalPreferences>);

impl<'a> PreferencesWriter<'a> {
    pub(super) fn new(preferences: &'a mut DocumentHolder<SavedGlobalPreferences>) -> Self {
        Self(preferences)
    }

    pub fn set_graphics_backend(&mut self, backend: GraphicsBackend) {
        self.0.edit(|values, toml_document| {
            toml_document["graphics_backend"] = value(backend.as_str());
            values.graphics_backend = backend;
        })
    }

    pub fn set_graphics_power_preference(&mut self, preference: PowerPreference) {
        self.0.edit(|values, toml_document| {
            toml_document["graphics_power_preference"] = value(preference.as_str());
            values.graphics_power_preference = preference;
        })
    }

    pub fn set_language(&mut self, language: LanguageIdentifier) {
        self.0.edit(|values, toml_document| {
            toml_document["language"] = value(language.to_string());
            values.language = language;
        })
    }

    pub fn set_output_device(&mut self, name: Option<String>) {
        self.0.edit(|values, toml_document| {
            if let Some(name) = &name {
                toml_document["output_device"] = value(name);
            } else {
                toml_document.remove("output_device");
            }
            values.output_device = name;
        })
    }

    pub fn set_mute(&mut self, mute: bool) {
        self.0.edit(|values, toml_document| {
            toml_document["mute"] = value(mute);
            values.mute = mute;
        })
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.0.edit(|values, toml_document| {
            toml_document["volume"] = value(volume as f64);
            values.volume = volume;
        })
    }

    pub fn set_log_filename_pattern(&mut self, pattern: FilenamePattern) {
        self.0.edit(|values, toml_document| {
            toml_document["log"]["filename_pattern"] = value(pattern.as_str());
            values.log.filename_pattern = pattern;
        })
    }

    pub fn set_storage_backend(&mut self, backend: StorageBackend) {
        self.0.edit(|values, toml_document| {
            toml_document["storage"]["backend"] = value(backend.as_str());
            values.storage.backend = backend;
        })
    }
}

pub struct BookmarksWriter<'a>(&'a mut DocumentHolder<Bookmarks>);

impl<'a> BookmarksWriter<'a> {
    pub(super) fn new(bookmarks: &'a mut DocumentHolder<Bookmarks>) -> Self {
        Self(bookmarks)
    }

    fn with_underlying_table(&mut self, fun: impl FnOnce(&mut Bookmarks, &mut ArrayOfTables)) {
        fn find_table(toml_document: &mut DocumentMut) -> &mut ArrayOfTables {
            if toml_document.contains_array_of_tables("bookmark") {
                return toml_document["bookmark"]
                    .as_array_of_tables_mut()
                    .expect("type was just verified");
            }

            tracing::warn!("missing or invalid bookmark array, recreating..");
            toml_document.insert("bookmark", array());
            toml_document["bookmark"]
                .as_array_of_tables_mut()
                .expect("type was just created")
        }

        self.0.edit(|values, toml_document| {
            let table = find_table(toml_document);
            fun(values, table)
        })
    }

    fn with_bookmark_table(&mut self, index: usize, fun: impl FnOnce(&mut Bookmarks, &mut Table)) {
        self.with_underlying_table(|values, array_of_tables| {
            let table = array_of_tables
                .get_mut(index)
                .expect("invalid bookmark index");
            fun(values, table)
        })
    }

    pub fn add(&mut self, bookmark: Bookmark) {
        self.with_underlying_table(|values, table| {
            let mut bookmark_table = Table::new();
            bookmark_table["url"] = value(bookmark.url.to_string());
            bookmark_table["name"] = value(&bookmark.name);
            table.push(bookmark_table);
            values.push(bookmark);
        })
    }

    pub fn set_url(&mut self, index: usize, url: url::Url) {
        self.with_bookmark_table(index, |values, table| {
            table["url"] = value(url.as_str());
            values[index].url = url;
        })
    }

    pub fn set_name(&mut self, index: usize, name: String) {
        self.with_bookmark_table(index, |values, table| {
            table["name"] = value(&name);
            values[index].name = name;
        })
    }

    pub fn remove(&mut self, index: usize) {
        self.with_underlying_table(|values, table| {
            table.remove(index);
            values.remove(index);
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fluent_templates::loader::langid;
    use std::ops::Deref;

    macro_rules! define_serialization_test_helpers {
        ($read_method:ident, $doc_struct:ty, $writer:ident) => {
            fn check_roundtrip(preferences: &DocumentHolder<$doc_struct>) {
                let read_result = $read_method(&preferences.serialize());
                assert_eq!(
                    preferences.deref(),
                    read_result.values(),
                    "roundtrip failed: expected != actual"
                );
            }

            fn test(original: &str, fun: impl FnOnce(&mut $writer), expected: &str) {
                let mut preferences = $read_method(original).result;
                let mut writer = $writer::new(&mut preferences);
                fun(&mut writer);
                check_roundtrip(&preferences);
                assert_eq!(expected, preferences.serialize());
            }
        };
    }

    mod preferences {
        use super::*;
        use crate::preferences::read::read_preferences;

        define_serialization_test_helpers!(
            read_preferences,
            SavedGlobalPreferences,
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

        #[test]
        fn set_storage_backend() {
            test(
                "",
                |writer| writer.set_storage_backend(StorageBackend::Disk),
                "storage = { backend = \"disk\" }\n",
            );
            test(
                "storage = { backend = \"disk\" }\n",
                |writer| writer.set_storage_backend(StorageBackend::Memory),
                "storage = { backend = \"memory\" }\n",
            );
            test(
                "[storage]\nbackend = \"disk\"\n",
                |writer| writer.set_storage_backend(StorageBackend::Memory),
                "[storage]\nbackend = \"memory\"\n",
            );
        }
    }

    #[allow(clippy::unwrap_used)]
    mod bookmarks {
        use super::*;
        use crate::preferences::read::read_bookmarks;
        use std::str::FromStr;
        use url::Url;

        define_serialization_test_helpers!(read_bookmarks, Bookmarks, BookmarksWriter);

        #[test]
        fn add_bookmark() {
            test(
                "",
                |writer| {
                    writer.add(Bookmark {
                        url: Url::from_str("file:///home/user/example.swf").unwrap(),
                        name: "example.swf".to_string(),
                    })
                },
                "[[bookmark]]\nurl = \"file:///home/user/example.swf\"\nname = \"example.swf\"\n",
            );
            test("[[bookmark]]\nurl = \"file:///home/user/example.swf\"\n", |writer| writer.add(Bookmark {
            url: Url::from_str("file:///home/user/another_file.swf").unwrap(),
            name: "another_file.swf".to_string(),
        }), "[[bookmark]]\nurl = \"file:///home/user/example.swf\"\n\n[[bookmark]]\nurl = \"file:///home/user/another_file.swf\"\nname = \"another_file.swf\"\n");
        }

        #[test]
        fn modify_bookmark() {
            test(
                "[[bookmark]]\nurl = \"file:///example.swf\"\n",
                |writer| writer.set_name(0, "Custom Name".to_string()),
                "[[bookmark]]\nurl = \"file:///example.swf\"\nname = \"Custom Name\"\n",
            );
            test(
                "[[bookmark]]\nurl = \"file:///example.swf\"\nname = \"example.swf\"",
                |writer| writer.set_url(0, Url::parse("https://ruffle.rs/logo-anim.swf").unwrap()),
                "[[bookmark]]\nurl = \"https://ruffle.rs/logo-anim.swf\"\nname = \"example.swf\"\n",
            );
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
            test("[[bookmark]]\nurl = \"file://home/user/example.swf\"\n\n[[bookmark]]\n\n[[bookmark]]\nurl = \"https://ruffle.rs/logo-anim.swf\"\n\n[[bookmark]]\nurl = \"invalid\"\n", |writer| {
            writer.remove(2);
        }, "[[bookmark]]\nurl = \"file://home/user/example.swf\"\n\n[[bookmark]]\n\n[[bookmark]]\nurl = \"invalid\"\n");

            // check if we can remove invalid entries.
            test("[[bookmark]]", |writer| writer.remove(0), "");
        }

        #[test]
        fn overwrite_invalid_bookmark_type() {
            test(
                "[bookmark]",
                |writer| {
                    writer.add(Bookmark {
                        url: Url::from_str("file:///test.swf").unwrap(),
                        name: "test.swf".to_string(),
                    })
                },
                "[[bookmark]]\nurl = \"file:///test.swf\"\nname = \"test.swf\"\n",
            );

            test(
                "bookmark = 1010",
                |writer| {
                    writer.add(Bookmark {
                        url: Url::from_str("file:///test.swf").unwrap(),
                        name: "test.swf".to_string(),
                    })
                },
                "[[bookmark]]\nurl = \"file:///test.swf\"\nname = \"test.swf\"\n",
            );
        }
    }
}
