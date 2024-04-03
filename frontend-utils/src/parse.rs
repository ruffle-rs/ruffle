use std::fmt;
use std::str::FromStr;
use toml_edit::{ArrayOfTables, DocumentMut, Item, Table, TableLike};

#[derive(Debug, PartialEq)]
pub struct ParseResult<T: PartialEq + fmt::Debug> {
    pub result: T,
    pub warnings: Vec<String>,
}

impl<T: fmt::Debug + PartialEq> ParseResult<T> {
    pub fn add_warning(&mut self, message: String) {
        self.warnings.push(message);
    }
}

#[derive(Default)]
pub struct ParseContext {
    pub warnings: Vec<String>,
    /// Path of the current item being parsed
    pub path: Vec<&'static str>,
}

impl ParseContext {
    pub fn push_key(&mut self, key: &'static str) {
        self.path.push(key);
    }

    pub fn pop_key(&mut self) {
        let _ = self.path.pop();
    }

    pub fn path(&self) -> String {
        self.path.join(".")
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
}

pub trait ReadExt<'a> {
    fn get_impl(&'a self, key: &str) -> Option<&'a Item>;

    fn get_table_like(
        &'a self,
        cx: &mut ParseContext,
        key: &'static str,
        fun: impl FnOnce(&mut ParseContext, &dyn TableLike),
    ) {
        if let Some(item) = self.get_impl(key) {
            cx.push_key(key);

            if let Some(table) = item.as_table_like() {
                fun(cx, table);
            } else {
                cx.add_warning(format!(
                    "Invalid {}: expected table but found {}",
                    cx.path(),
                    item.type_name()
                ));
            }

            cx.pop_key();
        }
    }

    fn get_array_of_tables(
        &'a self,
        cx: &mut ParseContext,
        key: &'static str,
        fun: impl FnOnce(&mut ParseContext, &ArrayOfTables),
    ) {
        if let Some(item) = self.get_impl(key) {
            cx.push_key(key);

            if let Some(array) = item.as_array_of_tables() {
                fun(cx, array);
            } else {
                cx.add_warning(format!(
                    "Invalid {}: expected array of tables but found {}",
                    cx.path(),
                    item.type_name()
                ));
            }

            cx.pop_key();
        }
    }

    fn parse_from_str<T: FromStr>(&'a self, cx: &mut ParseContext, key: &'static str) -> Option<T> {
        cx.push_key(key);

        let res = if let Some(item) = self.get_impl(key) {
            if let Some(str) = item.as_str() {
                if let Ok(value) = str.parse::<T>() {
                    Some(value)
                } else {
                    cx.add_warning(format!("Invalid {}: unsupported value {str:?}", cx.path()));
                    None
                }
            } else {
                cx.add_warning(format!(
                    "Invalid {}: expected string but found {}",
                    cx.path(),
                    item.type_name()
                ));
                None
            }
        } else {
            None
        };

        cx.pop_key();

        res
    }

    fn get_bool(&'a self, cx: &mut ParseContext, key: &'static str) -> Option<bool> {
        cx.push_key(key);

        let res = if let Some(item) = self.get_impl(key) {
            if let Some(value) = item.as_bool() {
                Some(value)
            } else {
                cx.add_warning(format!(
                    "Invalid {}: expected boolean but found {}",
                    cx.path(),
                    item.type_name()
                ));
                None
            }
        } else {
            None
        };

        cx.pop_key();

        res
    }

    fn get_float(&'a self, cx: &mut ParseContext, key: &'static str) -> Option<f64> {
        cx.push_key(key);

        let res = if let Some(item) = self.get_impl(key) {
            if let Some(value) = item.as_float() {
                Some(value)
            } else {
                cx.add_warning(format!(
                    "Invalid {}: expected float but found {}",
                    cx.path(),
                    item.type_name()
                ));
                None
            }
        } else {
            None
        };

        cx.pop_key();

        res
    }
}

// Implementations for toml_edit types.

impl<'a> ReadExt<'a> for DocumentMut {
    fn get_impl(&'a self, key: &str) -> Option<&'a Item> {
        self.get(key)
    }
}

impl<'a> ReadExt<'a> for Item {
    fn get_impl(&'a self, key: &str) -> Option<&'a Item> {
        self.get(key)
    }
}

impl<'a> ReadExt<'a> for Table {
    fn get_impl(&'a self, key: &str) -> Option<&'a Item> {
        self.get(key)
    }
}

impl<'a> ReadExt<'a> for dyn TableLike + 'a {
    fn get_impl(&'a self, key: &str) -> Option<&'a Item> {
        self.get(key)
    }
}
