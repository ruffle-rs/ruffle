use std::fmt;
use std::fmt::Formatter;
use std::ops::Deref;
use std::str::FromStr;
use toml_edit::{ArrayOfTables, DocumentMut, Item, Table, TableLike, TomlError};

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
    pub warnings: Vec<ParseWarning>,
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
    pub fn values(&self) -> &T {
        &self.result
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseWarning {
    InvalidToml(TomlError),
    UnexpectedType {
        expected: &'static str,
        actual: &'static str,
        path: String,
    },
    UnsupportedValue {
        value: String,
        path: String,
    },
}

impl fmt::Display for ParseWarning {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ParseWarning::InvalidToml(e) => write!(f, "Invalid TOML: {e}"),
            ParseWarning::UnexpectedType {
                expected,
                actual,
                path,
            } => write!(f, "Invalid {path}: expected {expected} but found {actual}"),
            ParseWarning::UnsupportedValue { value, path } => {
                write!(f, "Invalid {path}: unsupported value {value:?}")
            }
        }
    }
}

#[derive(Default)]
pub struct ParseContext<'a> {
    pub warnings: Vec<ParseWarning>,
    /// Path of the current item being parsed
    path: Vec<&'a str>,
}

impl<'a> ParseContext<'a> {
    pub fn push_key(&mut self, key: &'a str) {
        self.path.push(key);
    }

    pub fn pop_key(&mut self) {
        let _ = self.path.pop();
    }

    pub fn path(&self) -> String {
        self.path.join(".")
    }

    /// Emits an unexpected type warning.
    pub fn unexpected_type(&mut self, expected: &'static str, actual: &'static str) {
        self.warnings.push(ParseWarning::UnexpectedType {
            expected,
            actual,
            path: self.path(),
        })
    }

    /// Emits an unsupported value warning.
    pub fn unsupported_value(&mut self, value: String) {
        self.warnings.push(ParseWarning::UnsupportedValue {
            value,
            path: self.path(),
        })
    }
}

pub trait ReadExt<'a> {
    fn get_impl(&'a self, key: &str) -> Option<&'a Item>;

    fn get_table_like<R>(
        &'a self,
        cx: &mut ParseContext<'a>,
        key: &'static str,
        fun: impl FnOnce(&mut ParseContext<'a>, &'a dyn TableLike) -> R,
    ) -> Option<R> {
        let mut result = None;
        if let Some(item) = self.get_impl(key) {
            cx.push_key(key);

            if let Some(table) = item.as_table_like() {
                result = Some(fun(cx, table));
            } else {
                cx.unexpected_type("table", item.type_name());
            }

            cx.pop_key();
        }
        result
    }

    fn get_array_of_tables<R>(
        &'a self,
        cx: &mut ParseContext<'a>,
        key: &'static str,
        fun: impl FnOnce(&mut ParseContext<'a>, &'a ArrayOfTables) -> R,
    ) -> Option<R> {
        let mut result = None;
        if let Some(item) = self.get_impl(key) {
            cx.push_key(key);

            if let Some(array) = item.as_array_of_tables() {
                result = Some(fun(cx, array));
            } else {
                cx.unexpected_type("array of tables", item.type_name());
            }

            cx.pop_key();
        }
        result
    }

    fn parse_from_str<T: FromStr>(&'a self, cx: &mut ParseContext, key: &'static str) -> Option<T> {
        let mut result = None;

        cx.push_key(key);
        if let Some(str) = self.get_impl(key).and_then(|item| item.as_str_or_warn(cx)) {
            if let Ok(value) = str.parse::<T>() {
                result = Some(value)
            } else {
                cx.unsupported_value(str.to_owned());
            }
        }
        cx.pop_key();

        result
    }

    fn get_bool(&'a self, cx: &mut ParseContext, key: &'static str) -> Option<bool> {
        cx.push_key(key);
        let result = self.get_impl(key).and_then(|x| x.as_bool_or_warn(cx));
        cx.pop_key();

        result
    }

    fn get_float(&'a self, cx: &mut ParseContext, key: &'static str) -> Option<f64> {
        cx.push_key(key);
        let result = self.get_impl(key).and_then(|x| x.as_float_or_warn(cx));
        cx.pop_key();

        result
    }

    fn get_integer(&'a self, cx: &mut ParseContext, key: &'static str) -> Option<i64> {
        cx.push_key(key);
        let result = self.get_impl(key).and_then(|x| x.as_integer_or_warn(cx));
        cx.pop_key();

        result
    }

    /// Similar to [`ReadExt::get_float`], but also returns integers as floats.
    fn get_float_like(&'a self, cx: &mut ParseContext, key: &'static str) -> Option<f64> {
        let mut result = None;
        cx.push_key(key);

        if let Some(item) = self.get_impl(key) {
            if let Some(float) = item.as_float() {
                result = Some(float);
            } else if let Some(integer) = item.as_integer() {
                result = Some(integer as f64)
            } else {
                cx.unexpected_type("float or integer", item.type_name());
            }
        }

        cx.pop_key();
        result
    }
}

/// Extension trait to provide casting methods with warning capabilities.
pub trait ItemExt<'a> {
    fn as_str_or_warn(&'a self, cx: &mut ParseContext) -> Option<&'a str>;
    fn as_bool_or_warn(&self, cx: &mut ParseContext) -> Option<bool>;
    fn as_float_or_warn(&self, cx: &mut ParseContext) -> Option<f64>;
    fn as_integer_or_warn(&self, cx: &mut ParseContext) -> Option<i64>;
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

impl<'a> ItemExt<'a> for Item {
    fn as_bool_or_warn(&self, cx: &mut ParseContext) -> Option<bool> {
        if let Some(value) = self.as_bool() {
            return Some(value);
        } else {
            cx.unexpected_type("boolean", self.type_name());
        }

        None
    }

    fn as_float_or_warn(&self, cx: &mut ParseContext) -> Option<f64> {
        if let Some(value) = self.as_float() {
            return Some(value);
        } else {
            cx.unexpected_type("float", self.type_name());
        }

        None
    }

    fn as_str_or_warn(&'a self, cx: &mut ParseContext) -> Option<&'a str> {
        if let Some(value) = self.as_str() {
            return Some(value);
        } else {
            cx.unexpected_type("string", self.type_name());
        }

        None
    }

    fn as_integer_or_warn(&self, cx: &mut ParseContext) -> Option<i64> {
        if let Some(value) = self.as_integer() {
            return Some(value);
        } else {
            cx.unexpected_type("integer", self.type_name());
        }

        None
    }
}
