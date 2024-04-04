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
