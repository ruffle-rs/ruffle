use toml_edit::{array, ArrayOfTables, Table};

pub trait TableExt {
    /// Gets an existing array of tables, or creates a new one if it does not exist or type is different.
    fn get_or_create_array_of_tables(&mut self, key: &str) -> &mut ArrayOfTables;
}

impl TableExt for Table {
    fn get_or_create_array_of_tables(&mut self, key: &str) -> &mut ArrayOfTables {
        if self.contains_array_of_tables(key) {
            return self[key]
                .as_array_of_tables_mut()
                .expect("type was just verified");
        }

        tracing::warn!("missing or invalid '{key}' array, recreating..");
        self.insert(key, array());
        self[key]
            .as_array_of_tables_mut()
            .expect("type was just created")
    }
}

#[macro_export]
macro_rules! define_serialization_test_helpers {
    ($read_method:ident, $doc_struct:ty, $writer:ident) => {
        fn check_roundtrip(preferences: &DocumentHolder<$doc_struct>) {
            use std::ops::Deref;
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
