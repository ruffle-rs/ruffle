use std::fmt;
use std::fmt::Formatter;
use std::ops::Deref;
use std::str::FromStr;
use toml_edit::{ArrayOfTables, DocumentMut, Item, Table, TableLike};

/// A holder over values that may be read and potentially written back to disk.
///
/// Two versions of Ruffle may have different preferences, or different values available for each preference.
/// For this reason, we store both the original toml document *and* the parsed values as we understand them.
/// Whenever we persist values back to the toml, we only edit the values we changed and leave the remaining
/// values as they originally were.
/// This way, switching between different versions will *not* wipe your settings or get Ruffle into an
/// invalid state.
pub struct DocumentHolder<T> {
    /// The original toml document
    toml_document: DocumentMut,

    /// The actual values stored within the toml document, as this version of Ruffle understands them.
    inner: T,
}

impl<T> Deref for DocumentHolder<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: Default> Default for DocumentHolder<T> {
    fn default() -> Self {
        Self {
            toml_document: Default::default(),
            inner: Default::default(),
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for DocumentHolder<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("DocumentHolder")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<T> DocumentHolder<T> {
    pub fn new(values: T, document: DocumentMut) -> Self {
        Self {
            inner: values,
            toml_document: document,
        }
    }

    /// Allows editing of this DocumentHolder.
    ///
    /// The provided function is expected to:
    /// - Only edit the toml **as little as is required** for the given change
    /// - Ensure that the toml and values are kept in sync, such that reading the toml again provides the same values
    pub fn edit(&mut self, fun: impl FnOnce(&mut T, &mut DocumentMut)) {
        fun(&mut self.inner, &mut self.toml_document)
    }

    /// Takes the saved values from this DocumentHolder, discarding the document.
    /// Useful when you need the result of parsing, without needing to write to it again later.
    pub fn take(self) -> T {
        self.inner
    }

    pub fn serialize(&self) -> String {
        self.toml_document.to_string()
    }
}

pub struct ParseDetails<T> {
    pub result: DocumentHolder<T>,
    pub warnings: Vec<String>,
}

impl<T: fmt::Debug> fmt::Debug for ParseDetails<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("ParseResult")
            .field("result", &self.result)
            .field("warnings", &self.warnings)
            .finish()
    }
}

impl<T> ParseDetails<T> {
    pub fn add_warning(&mut self, message: String) {
        self.warnings.push(message);
    }

    pub fn values(&self) -> &T {
        &self.result
    }
}

#[derive(Default)]
pub struct ParseContext {
    pub warnings: Vec<String>,
    /// Path of the current item being parsed
    path: Vec<&'static str>,
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

    fn get_table_like<R>(
        &'a self,
        cx: &mut ParseContext,
        key: &'static str,
        fun: impl FnOnce(&mut ParseContext, &dyn TableLike) -> R,
    ) -> Option<R> {
        let mut result = None;
        if let Some(item) = self.get_impl(key) {
            cx.push_key(key);

            if let Some(table) = item.as_table_like() {
                result = Some(fun(cx, table));
            } else {
                cx.add_warning(format!(
                    "Invalid {}: expected table but found {}",
                    cx.path(),
                    item.type_name()
                ));
            }

            cx.pop_key();
        }
        result
    }

    fn get_array_of_tables<R>(
        &'a self,
        cx: &mut ParseContext,
        key: &'static str,
        fun: impl FnOnce(&mut ParseContext, &ArrayOfTables) -> R,
    ) -> Option<R> {
        let mut result = None;
        if let Some(item) = self.get_impl(key) {
            cx.push_key(key);

            if let Some(array) = item.as_array_of_tables() {
                result = Some(fun(cx, array));
            } else {
                cx.add_warning(format!(
                    "Invalid {}: expected array of tables but found {}",
                    cx.path(),
                    item.type_name()
                ));
            }

            cx.pop_key();
        }
        result
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
