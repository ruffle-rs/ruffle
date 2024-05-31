use crate::avm1::globals::local_connection::LocalConnection as Avm1LocalConnectionObject;
use crate::avm1::Object as Avm1Object;
use crate::avm2::object::LocalConnectionObject;
use crate::avm2::Domain as Avm2Domain;
use crate::context::UpdateContext;
use crate::string::AvmString;
use flash_lso::types::Value as AmfValue;
use fnv::FnvHashMap;
use gc_arena::Collect;
use ruffle_wstr::{WStr, WString};
use std::borrow::Cow;

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub enum LocalConnectionKind<'gc> {
    Avm2(Avm2Domain<'gc>, LocalConnectionObject<'gc>),
    Avm1(Avm1Object<'gc>),
}

impl<'gc> From<(Avm2Domain<'gc>, LocalConnectionObject<'gc>)> for LocalConnectionKind<'gc> {
    fn from(obj: (Avm2Domain<'gc>, LocalConnectionObject<'gc>)) -> Self {
        Self::Avm2(obj.0, obj.1)
    }
}

impl<'gc> From<Avm1Object<'gc>> for LocalConnectionKind<'gc> {
    fn from(obj: Avm1Object<'gc>) -> Self {
        Self::Avm1(obj)
    }
}

impl<'gc> LocalConnectionKind<'gc> {
    pub fn send_status(&self, context: &mut UpdateContext<'_, 'gc>, status: &'static str) {
        match self {
            LocalConnectionKind::Avm2(_domain, object) => {
                object.send_status(context, status);
            }
            LocalConnectionKind::Avm1(object) => {
                if let Err(e) = Avm1LocalConnectionObject::send_status(context, *object, status) {
                    tracing::error!("Unhandled AVM1 error during LocalConnection onStatus: {e}");
                }
            }
        }
    }

    pub fn run_method(
        &self,
        context: &mut UpdateContext<'_, 'gc>,
        method_name: AvmString<'gc>,
        arguments: Vec<AmfValue>,
    ) {
        match self {
            LocalConnectionKind::Avm2(domain, object) => {
                object.run_method(context, *domain, method_name, arguments);
            }
            LocalConnectionKind::Avm1(object) => {
                if let Err(e) =
                    Avm1LocalConnectionObject::run_method(context, *object, method_name, arguments)
                {
                    tracing::error!("Unhandled AVM1 error during LocalConnection onStatus: {e}");
                }
            }
        }
    }
}

#[derive(Collect)]
#[collect(no_drop)]
pub struct QueuedMessage<'gc> {
    source: LocalConnectionKind<'gc>,
    kind: QueuedMessageKind<'gc>,
}

#[derive(Collect)]
#[collect(no_drop)]
pub enum QueuedMessageKind<'gc> {
    Failure,
    Message {
        #[collect(require_static)]
        connection_name: WString,
        method_name: AvmString<'gc>,
        #[collect(require_static)]
        arguments: Vec<AmfValue>,
    },
}

impl<'gc> QueuedMessageKind<'gc> {
    pub fn deliver(self, source: LocalConnectionKind<'gc>, context: &mut UpdateContext<'_, 'gc>) {
        match self {
            QueuedMessageKind::Failure => {
                source.send_status(context, "error");
            }
            QueuedMessageKind::Message {
                connection_name,
                method_name,
                arguments,
            } => {
                if let Some(receiver) = context.local_connections.find_listener(&connection_name) {
                    source.send_status(context, "status");
                    receiver.run_method(context, method_name, arguments);
                } else {
                    source.send_status(context, "error");
                }
            }
        }
    }
}

/// An opaque handle to an actively listening LocalConnection.
/// Owning this handle represents ownership of a LocalConnection;
/// However, a LocalConnection must be manually closed, you can't just Drop this handle.
#[derive(Debug)]
pub struct LocalConnectionHandle(WString);

/// Manages the collection of local connections.
pub struct LocalConnections<'gc> {
    connections: FnvHashMap<WString, LocalConnectionKind<'gc>>,
    messages: Vec<QueuedMessage<'gc>>,
}

unsafe impl Collect for LocalConnections<'_> {
    fn trace(&self, cc: &gc_arena::Collection) {
        for (_, v) in self.connections.iter() {
            v.trace(cc);
        }
        for m in self.messages.iter() {
            m.trace(cc);
        }
    }
}

impl<'gc> LocalConnections<'gc> {
    pub fn empty() -> Self {
        Self {
            connections: Default::default(),
            messages: Default::default(),
        }
    }

    pub fn connect<C: Into<LocalConnectionKind<'gc>>>(
        &mut self,
        domain: &str,
        connection: C,
        name: &WStr,
    ) -> Option<LocalConnectionHandle> {
        let key = if name.starts_with(b'_') {
            name.to_ascii_lowercase()
        } else {
            let mut key = WString::from_utf8(Self::get_superdomain(domain));
            key.push_char(':');
            key.push_str(name);
            key.make_ascii_lowercase();
            key
        };

        if self.connections.contains_key(&key) {
            None
        } else {
            self.connections.insert(key.to_owned(), connection.into());
            Some(LocalConnectionHandle(key.to_owned()))
        }
    }

    pub fn close(&mut self, handle: LocalConnectionHandle) {
        self.connections.remove(&handle.0);
    }

    pub fn send<C: Into<LocalConnectionKind<'gc>>>(
        &mut self,
        domain: &str,
        source: C,
        connection_name: AvmString<'gc>,
        method_name: AvmString<'gc>,
        arguments: Vec<AmfValue>,
    ) {
        // There's two checks for "is connected":
        // 1 - At `send()` time, if there's no connections, just immediately queue up a failure
        // 2 - At `update_connections()` time, if the connection couldn't be found, queue up a failure
        // Even if one becomes available between send and update, it won't be used
        // Similarly, if one becomes unavailable between send and update, it'll error
        // If something *else* takes its place between send and update, it'll use that instead

        let mut connection_name = connection_name.to_ascii_lowercase();
        if !connection_name.contains(b':') && !connection_name.starts_with(b'_') {
            let mut result = WString::from_utf8(Self::get_superdomain(domain));
            result.push_char(':');
            result.push_str(&connection_name);
            connection_name = result;
        }

        let kind = if self.find_listener(&connection_name).is_some() {
            QueuedMessageKind::Message {
                connection_name,
                method_name,
                arguments,
            }
        } else {
            QueuedMessageKind::Failure
        };
        self.messages.push(QueuedMessage {
            source: source.into(),
            kind,
        });
    }

    fn find_listener(&self, name: &WStr) -> Option<LocalConnectionKind<'gc>> {
        self.connections.get(name).cloned()
    }

    pub fn update_connections(context: &mut UpdateContext<'_, 'gc>) {
        if context.local_connections.messages.is_empty() {
            return;
        }

        for message in std::mem::take(&mut context.local_connections.messages) {
            message.kind.deliver(message.source, context);
        }
    }

    pub fn get_domain(url: &str) -> Cow<'static, str> {
        if let Ok(url) = url::Url::parse(url) {
            if url.scheme() == "file" {
                Cow::Borrowed("localhost")
            } else if let Some(domain) = url.domain() {
                Cow::Owned(domain.to_owned())
            } else {
                // no domain?
                Cow::Borrowed("localhost")
            }
        } else {
            tracing::error!("LocalConnection: Unable to parse movie URL: {url}");
            return Cow::Borrowed("unknown"); // this is surely an error but it'll hopefully highlight this case in issues for us
        }
    }

    pub fn get_superdomain(domain: &str) -> &str {
        domain.rsplit_once('.').map(|(_, b)| b).unwrap_or(domain)
    }
}
