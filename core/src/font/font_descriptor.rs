use gc_arena::Collect;
use std::hash::Hash;
use std::hash::Hasher;

/// Structure which identifies a particular font by name and properties.
#[derive(Debug, Clone, Ord, PartialOrd, Collect)]
#[collect(require_static)]
pub struct FontDescriptor {
    /// The name of the font.
    /// This is set by the author of the SWF and does not correlate to any opentype names.
    name: String,

    // All name comparisons ignore case, so this is for easy comparisons.
    lowercase_name: String,

    is_bold: bool,
    is_italic: bool,
}

impl PartialEq for FontDescriptor {
    fn eq(&self, other: &Self) -> bool {
        self.lowercase_name == other.lowercase_name
            && self.is_italic == other.is_italic
            && self.is_bold == other.is_bold
    }
}

impl Eq for FontDescriptor {}

impl Hash for FontDescriptor {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.lowercase_name.hash(state);
        self.is_bold.hash(state);
        self.is_italic.hash(state);
    }
}

impl FontDescriptor {
    /// Obtain a font descriptor from a SWF font tag.
    pub fn from_swf_tag(val: &swf::Font, encoding: &'static swf::Encoding) -> Self {
        let name = val.name.to_string_lossy(encoding);
        let lowercase_name = name.to_lowercase();

        Self {
            name,
            lowercase_name,
            is_bold: val.flags.contains(swf::FontFlag::IS_BOLD),
            is_italic: val.flags.contains(swf::FontFlag::IS_ITALIC),
        }
    }

    /// Obtain a font descriptor from a name/bold/italic triplet.
    pub fn from_parts(name: &str, is_bold: bool, is_italic: bool) -> Self {
        let mut name = name.to_string();

        if let Some(first_null) = name.find('\0') {
            name.truncate(first_null);
        };
        let lowercase_name = name.to_lowercase();

        Self {
            name,
            lowercase_name,
            is_bold,
            is_italic,
        }
    }

    /// Get the name of the font this descriptor identifies.
    pub fn name(&self) -> &str {
        &self.name
    }

    // Get the lowercase name.
    pub fn lowercase_name(&self) -> &str {
        &self.lowercase_name
    }

    /// Get the boldness of the described font.
    pub fn bold(&self) -> bool {
        self.is_bold
    }

    /// Get the italic-ness of the described font.
    pub fn italic(&self) -> bool {
        self.is_italic
    }
}
