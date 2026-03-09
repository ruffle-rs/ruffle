use crate::avm1::Object as Avm1Object;
use crate::avm1::globals::local_connection::LocalConnection as Avm1LocalConnectionObject;
use crate::avm2::Domain as Avm2Domain;
use crate::avm2::object::LocalConnectionObject;
use crate::backend::local_connection::ExternalLocalConnectionMessage;
use crate::context::UpdateContext;
use crate::string::AvmString;
use flash_lso::types::{AMFVersion, Element, Lso, Value as AmfValue};
use fnv::FnvHashMap;
use gc_arena::Collect;
use gc_arena::collect::Trace;
use ruffle_macros::istr;
use ruffle_wstr::{WStr, WString};
use std::borrow::Cow;
use std::rc::Rc;
use std::str::FromStr;

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
    pub fn send_status(&self, status: AvmString<'gc>, context: &mut UpdateContext<'gc>) {
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
        context: &mut UpdateContext<'gc>,
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
    RemoteSuccess,
    Message {
        #[collect(require_static)]
        connection_name: WString,
        method_name: AvmString<'gc>,
        #[collect(require_static)]
        arguments: Vec<AmfValue>,
    },
}

impl<'gc> QueuedMessageKind<'gc> {
    pub fn deliver(self, source: LocalConnectionKind<'gc>, context: &mut UpdateContext<'gc>) {
        match self {
            QueuedMessageKind::Failure => {
                source.send_status(istr!(context, "error"), context);
            }
            QueuedMessageKind::RemoteSuccess => {
                source.send_status(istr!(context, "status"), context);
            }
            QueuedMessageKind::Message {
                connection_name,
                method_name,
                arguments,
            } => {
                if let Some(receiver) = context.local_connections.find_listener(&connection_name) {
                    source.send_status(istr!(context, "status"), context);
                    receiver.run_method(context, method_name, arguments);
                } else {
                    source.send_status(istr!(context, "error"), context);
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

// TODO(moulins): use gc_arena::Static to avoid unsafe impl?
unsafe impl<'gc> Collect<'gc> for LocalConnections<'gc> {
    fn trace<C: Trace<'gc>>(&self, cc: &mut C) {
        for (_, v) in self.connections.iter() {
            cc.trace(v);
        }
        cc.trace(&self.messages);
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
        backend: &mut dyn crate::backend::local_connection::LocalConnectionBackend,
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

        let key_utf8 = key.to_utf8_lossy();
        if self.connections.contains_key(&key) || backend.has_remote_listener(&key_utf8) {
            None
        } else {
            // Notify the backend so other contexts can discover this listener.
            backend.register_listener(&key_utf8);
            self.connections.insert(key.to_owned(), connection.into());
            Some(LocalConnectionHandle(key))
        }
    }

    pub fn close(
        &mut self,
        handle: LocalConnectionHandle,
        backend: &mut dyn crate::backend::local_connection::LocalConnectionBackend,
    ) {
        // Notify the backend that this listener is gone.
        backend.unregister_listener(&handle.0.to_utf8_lossy());
        self.connections.remove(&handle.0);
    }

    /// Maximum size of serialized LocalConnection message data (40KB), matching Flash Player.
    const MAX_MESSAGE_SIZE: usize = 40 * 1024;

    pub fn send<C: Into<LocalConnectionKind<'gc>>>(
        &mut self,
        domain: &str,
        source: C,
        connection_name: AvmString<'gc>,
        method_name: AvmString<'gc>,
        arguments: Vec<AmfValue>,
        backend: &mut dyn crate::backend::local_connection::LocalConnectionBackend,
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

        let source = source.into();

        // Serialize arguments and enforce 40KB size limit.
        let amf_data = match Self::serialize_arguments(&arguments) {
            Some(data) => {
                if data.len() > Self::MAX_MESSAGE_SIZE {
                    tracing::warn!(
                        "LocalConnection message exceeds 40KB limit ({} bytes), dropping",
                        data.len()
                    );
                    self.messages.push(QueuedMessage {
                        source,
                        kind: QueuedMessageKind::Failure,
                    });
                    return;
                }
                Some(data)
            }
            None => None,
        };

        if connection_name.iter().any(|c| c > 0x7F) {
            tracing::warn!(
                "LocalConnection name contains non-ASCII characters; cross-tab delivery may fail"
            );
        }

        // Broadcast to other contexts.
        if let Some(ref amf_data) = amf_data {
            backend.send_message(
                &connection_name.to_utf8_lossy(),
                &method_name.to_utf8_lossy(),
                amf_data,
            );
        }

        let connection_name_utf8 = connection_name.to_utf8_lossy();
        let kind = if self.find_listener(&connection_name).is_some() {
            QueuedMessageKind::Message {
                connection_name,
                method_name,
                arguments,
            }
        } else if backend.has_remote_listener(&connection_name_utf8) {
            // No local listener, but a remote one was recently seen in localStorage.
            // We assume success here, though there's a small chance the listener has
            // gone away since it last refreshed its heartbeat timestamp.
            QueuedMessageKind::RemoteSuccess
        } else {
            QueuedMessageKind::Failure
        };
        self.messages.push(QueuedMessage { source, kind });
    }

    fn find_listener(&self, name: &WStr) -> Option<LocalConnectionKind<'gc>> {
        self.connections.get(name).cloned()
    }

    pub fn update_connections(context: &mut UpdateContext<'gc>) {
        // 1. Poll for incoming external messages from other contexts.
        let external_messages = context.local_connection_backend.poll_incoming();
        for msg in external_messages {
            Self::deliver_external_message(context, msg);
        }

        // 2. Deliver locally queued messages (existing logic).
        if context.local_connections.messages.is_empty() {
            return;
        }

        for message in std::mem::take(&mut context.local_connections.messages) {
            message.kind.deliver(message.source, context);
        }
    }

    /// Deliver a message received from another Ruffle context via the backend.
    fn deliver_external_message(
        context: &mut UpdateContext<'gc>,
        msg: ExternalLocalConnectionMessage,
    ) {
        let connection_name = WString::from_utf8(&msg.connection_name);
        if let Some(receiver) = context.local_connections.find_listener(&connection_name) {
            let method_name = AvmString::new_utf8(context.gc_context, &msg.method_name);
            let arguments = Self::deserialize_arguments(&msg.amf_data);
            receiver.run_method(context, method_name, arguments);
        }
    }

    /// Serialize AMF arguments to bytes for cross-context transport.
    fn serialize_arguments(arguments: &[AmfValue]) -> Option<Vec<u8>> {
        let elements: Vec<Element> = arguments
            .iter()
            .enumerate()
            .map(|(i, v)| Element::new(format!("arg{i}"), Rc::new(v.clone())))
            .collect();
        let mut lso = Lso::new(elements, "", AMFVersion::AMF0);
        flash_lso::write::write_to_bytes(&mut lso).ok()
    }

    /// Deserialize AMF arguments from bytes received from another context.
    fn deserialize_arguments(data: &[u8]) -> Vec<AmfValue> {
        match flash_lso::read::Reader::default().parse(data) {
            Ok(lso) => lso.body.into_iter().map(|e| (*e.value()).clone()).collect(),
            Err(e) => {
                tracing::error!("Failed to deserialize cross-tab LocalConnection arguments: {e}");
                vec![]
            }
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
            Cow::Borrowed("unknown") // this is surely an error but it'll hopefully highlight this case in issues for us
        }
    }

    /// Extract the "superdomain" from a domain string.
    /// Flash uses the last two dot-separated segments as the superdomain:
    /// - "www.someDomain.com" → "someDomain.com"
    /// - "someDomain.com" → "someDomain.com"
    /// - "localhost" → "localhost"
    pub fn get_superdomain(domain: &str) -> &str {
        if std::net::IpAddr::from_str(domain).is_ok() {
            return domain;
        }

        let mut dots_seen = 0;
        for (i, b) in domain.bytes().rev().enumerate() {
            if b == b'.' {
                dots_seen += 1;
                if dots_seen == 2 {
                    return &domain[domain.len() - i..];
                }
            }
        }
        domain // 0 or 1 dots — already a superdomain
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_superdomain() {
        assert_eq!(
            LocalConnections::get_superdomain("www.someDomain.com"),
            "someDomain.com"
        );
        assert_eq!(
            LocalConnections::get_superdomain("someDomain.com"),
            "someDomain.com"
        );
        assert_eq!(LocalConnections::get_superdomain("localhost"), "localhost");
        assert_eq!(LocalConnections::get_superdomain("a.b.c.d.com"), "d.com");
        assert_eq!(LocalConnections::get_superdomain("com"), "com");
        assert_eq!(LocalConnections::get_superdomain(""), "");
        assert_eq!(LocalConnections::get_superdomain("127.0.0.1"), "127.0.0.1");
        assert_eq!(
            LocalConnections::get_superdomain("192.168.1.100"),
            "192.168.1.100"
        );
    }

    #[test]
    fn test_get_domain() {
        assert_eq!(
            &*LocalConnections::get_domain("http://www.adobe.com/foo.swf"),
            "www.adobe.com"
        );
        assert_eq!(
            &*LocalConnections::get_domain("file:///C:/foo.swf"),
            "localhost"
        );
        assert_eq!(
            &*LocalConnections::get_domain("http://localhost/foo.swf"),
            "localhost"
        );
    }
}
