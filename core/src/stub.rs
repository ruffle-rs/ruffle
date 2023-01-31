use fnv::FnvHashSet;
use std::borrow::Cow;
use std::collections::hash_set::Iter;
use std::fmt::{Debug, Display, Formatter};

#[cfg(feature = "known_stubs")]
#[linkme::distributed_slice]
pub static KNOWN_STUBS: [Stub] = [..];

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Clone)]
pub enum Stub {
    Avm1Method {
        class: &'static str,
        method: &'static str,
        specifics: Option<&'static str>,
    },
    Avm2Getter {
        class: &'static str,
        field: &'static str,
    },
    Avm2Setter {
        class: &'static str,
        field: &'static str,
    },
    Other(Cow<'static, str>),
}

impl Display for Stub {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Stub::Avm1Method {
                class,
                method,
                specifics: None,
            } => {
                write!(f, "AVM1 {class}.{method}()")
            }
            Stub::Avm1Method {
                class,
                method,
                specifics: Some(specifics),
            } => {
                write!(f, "AVM1 {class}.{method}() {specifics}")
            }
            Stub::Avm2Getter { class, field } => {
                write!(f, "AVM2 {class}.{field} getter")
            }
            Stub::Avm2Setter { class, field } => {
                write!(f, "AVM2 {class}.{field} setter")
            }
            Stub::Other(text) => write!(f, "{text}"),
        }
    }
}

#[derive(Debug, Default)]
pub struct StubCollection {
    inner: FnvHashSet<Stub>,
}

impl StubCollection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn encounter(&mut self, stub: &Stub) {
        if !self.inner.contains(stub) {
            tracing::warn!("Encountered stub: {stub}");
            self.inner.insert(stub.clone());
        }
    }

    pub fn iter(&self) -> Iter<Stub> {
        self.inner.iter()
    }
}
