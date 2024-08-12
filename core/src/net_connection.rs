use crate::avm1::globals::netconnection::NetConnection as Avm1NetConnectionObject;
use crate::avm1::Object as Avm1Object;
use crate::avm2::object::{
    NetConnectionObject as Avm2NetConnectionObject, ResponderObject as Avm2ResponderObject,
};
use crate::avm2::{Activation as Avm2Activation, Avm2, EventObject as Avm2EventObject};
use crate::backend::navigator::{ErrorResponse, NavigatorBackend, OwnedFuture, Request};
use crate::context::UpdateContext;
use crate::loader::Error;
use crate::string::AvmString;
use crate::Player;
use flash_lso::packet::{Header, Message, Packet};
use flash_lso::types::{AMFVersion, Value as AmfValue};
use gc_arena::{Collect, DynamicRoot, Rootable};
use slotmap::{new_key_type, SlotMap};
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use std::sync::{Mutex, Weak};

new_key_type! {
    pub struct NetConnectionHandle;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum ResponderCallback {
    Result,
    Status,
}

#[derive(Clone)]
pub enum ResponderHandle {
    Avm2(DynamicRoot<Rootable![Avm2ResponderObject<'_>]>),
    Avm1(DynamicRoot<Rootable![Avm1Object<'_>]>),
}

impl Debug for ResponderHandle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ResponderHandle::Avm2(_) => write!(f, "ResponderHandle::Avm2"),
            ResponderHandle::Avm1(_) => write!(f, "ResponderHandle::Avm1"),
        }
    }
}

impl ResponderHandle {
    pub fn call(
        &self,
        context: &mut UpdateContext<'_>,
        callback: ResponderCallback,
        message: Rc<AmfValue>,
    ) {
        match self {
            ResponderHandle::Avm2(handle) => {
                let object = context.dynamic_root.fetch(handle);
                if let Err(e) = object.send_callback(context, callback, &message) {
                    tracing::error!("Unhandled error sending {callback:?} callback: {e}");
                }
            }
            ResponderHandle::Avm1(handle) => {
                let object = context.dynamic_root.fetch(handle);
                if let Err(e) =
                    Avm1NetConnectionObject::send_callback(context, *object, callback, &message)
                {
                    tracing::error!("Unhandled error sending {callback:?} callback: {e}");
                }
            }
        }
    }
}

#[derive(Copy, Clone, Collect)]
#[collect(no_drop)]
pub enum NetConnectionObject<'gc> {
    Avm2(Avm2NetConnectionObject<'gc>),
    Avm1(Avm1Object<'gc>),
}

impl<'gc> NetConnectionObject<'gc> {
    pub fn set_handle(&self, handle: Option<NetConnectionHandle>) -> Option<NetConnectionHandle> {
        match self {
            NetConnectionObject::Avm2(object) => object.set_handle(handle),
            NetConnectionObject::Avm1(object) => {
                if let Some(net_connection) = Avm1NetConnectionObject::cast((*object).into()) {
                    net_connection.set_handle(handle)
                } else {
                    None
                }
            }
        }
    }
}

impl<'gc> From<Avm2NetConnectionObject<'gc>> for NetConnectionObject<'gc> {
    fn from(value: Avm2NetConnectionObject<'gc>) -> Self {
        NetConnectionObject::Avm2(value)
    }
}

impl<'gc> From<Avm1Object<'gc>> for NetConnectionObject<'gc> {
    fn from(value: Avm1Object<'gc>) -> Self {
        NetConnectionObject::Avm1(value)
    }
}

/// Manages the collection of NetConnections.
pub struct NetConnections<'gc> {
    connections: SlotMap<NetConnectionHandle, NetConnection<'gc>>,
}

unsafe impl<'gc> Collect for NetConnections<'gc> {
    fn trace(&self, cc: &gc_arena::Collection) {
        for (_, connection) in self.connections.iter() {
            connection.trace(cc)
        }
    }
}

impl<'gc> Default for NetConnections<'gc> {
    fn default() -> Self {
        Self {
            connections: SlotMap::with_key(),
        }
    }
}

impl<'gc> NetConnections<'gc> {
    pub fn connect_to_local<O: Into<NetConnectionObject<'gc>>>(
        context: &mut UpdateContext<'gc>,
        target: O,
    ) {
        let target = target.into();
        let connection = NetConnection {
            object: target,
            protocol: NetConnectionProtocol::Local,
        };
        let handle = context.net_connections.connections.insert(connection);

        if let Some(existing_handle) = target.set_handle(Some(handle)) {
            NetConnections::close(context, existing_handle, false);
        }

        match target {
            NetConnectionObject::Avm2(object) => {
                let mut activation = Avm2Activation::from_nothing(context);
                let event = Avm2EventObject::net_status_event(
                    &mut activation,
                    "netStatus",
                    vec![
                        ("code", "NetConnection.Connect.Success"),
                        ("level", "status"),
                    ],
                );
                Avm2::dispatch_event(activation.context, event, object.into());
            }
            NetConnectionObject::Avm1(object) => {
                if let Err(e) = Avm1NetConnectionObject::on_status_event(
                    context,
                    object,
                    "NetConnection.Connect.Success",
                ) {
                    tracing::error!("Unhandled error sending connection callback: {e}");
                }
            }
        }
    }

    pub fn connect_to_flash_remoting<O: Into<NetConnectionObject<'gc>>>(
        context: &mut UpdateContext<'gc>,
        target: O,
        url: String,
    ) {
        let target = target.into();
        let connection = NetConnection {
            object: target,
            protocol: NetConnectionProtocol::FlashRemoting(FlashRemoting {
                url,
                headers: vec![],
                outgoing_queue: vec![],
            }),
        };
        let handle = context.net_connections.connections.insert(connection);

        if let Some(existing_handle) = target.set_handle(Some(handle)) {
            NetConnections::close(context, existing_handle, false);
        }

        // No open event here
    }

    pub fn close(context: &mut UpdateContext<'gc>, handle: NetConnectionHandle, is_explicit: bool) {
        let Some(connection) = context.net_connections.connections.remove(handle) else {
            return;
        };

        match connection.object {
            NetConnectionObject::Avm2(object) => {
                let mut activation = Avm2Activation::from_nothing(context);
                let event = Avm2EventObject::net_status_event(
                    &mut activation,
                    "netStatus",
                    vec![
                        ("code", "NetConnection.Connect.Closed"),
                        ("level", "status"),
                    ],
                );
                Avm2::dispatch_event(activation.context, event, object.into());

                if is_explicit
                    && matches!(connection.protocol, NetConnectionProtocol::FlashRemoting(_))
                {
                    // [NA] I have no idea why, but a NetConnection receives a second and nonsensical event on close
                    let event = Avm2EventObject::net_status_event(
                        &mut activation,
                        "netStatus",
                        vec![
                            ("code", ""),
                            ("description", ""),
                            ("details", ""),
                            ("level", "status"),
                        ],
                    );
                    Avm2::dispatch_event(activation.context, event, object.into());
                }
            }
            NetConnectionObject::Avm1(object) => {
                if let Err(e) = Avm1NetConnectionObject::on_status_event(
                    context,
                    object,
                    "NetConnection.Connect.Closed",
                ) {
                    tracing::error!("Unhandled error sending connection callback: {e}");
                }
                if is_explicit
                    && matches!(connection.protocol, NetConnectionProtocol::FlashRemoting(_))
                {
                    if let Err(e) = Avm1NetConnectionObject::on_empty_status_event(context, object)
                    {
                        tracing::error!("Unhandled error sending connection callback: {e}");
                    }
                }
            }
        }
    }

    pub fn update_connections(context: &mut UpdateContext<'gc>) {
        for (handle, connection) in context.net_connections.connections.iter_mut() {
            connection.update(handle, context.navigator, context.player.clone());
        }
    }

    pub fn send_without_response(
        context: &mut UpdateContext<'gc>,
        handle: NetConnectionHandle,
        command: String,
        message: AmfValue,
    ) {
        if let Some(connection) = context.net_connections.connections.get_mut(handle) {
            connection.send(command, None, message);
        }
    }

    pub fn send_avm2(
        context: &mut UpdateContext<'gc>,
        handle: NetConnectionHandle,
        command: String,
        message: AmfValue,
        responder: Avm2ResponderObject<'gc>,
    ) {
        if let Some(connection) = context.net_connections.connections.get_mut(handle) {
            let responder_handle =
                ResponderHandle::Avm2(context.dynamic_root.stash(context.gc_context, responder));
            connection.send(command, Some(responder_handle), message);
        }
    }

    pub fn send_avm1(
        context: &mut UpdateContext<'gc>,
        handle: NetConnectionHandle,
        command: String,
        message: AmfValue,
        responder: Avm1Object<'gc>,
    ) {
        if let Some(connection) = context.net_connections.connections.get_mut(handle) {
            let responder_handle =
                ResponderHandle::Avm1(context.dynamic_root.stash(context.gc_context, responder));
            connection.send(command, Some(responder_handle), message);
        }
    }

    pub fn set_header(&mut self, handle: NetConnectionHandle, header: Header) {
        if let Some(connection) = self.connections.get_mut(handle) {
            connection.set_header(header);
        }
    }

    pub fn is_connected(&self, handle: NetConnectionHandle) -> bool {
        self.connections
            .get(handle)
            .map(|c| c.is_connected())
            .unwrap_or_default()
    }

    pub fn get_connected_proxy_type(&self, handle: NetConnectionHandle) -> Option<&'static str> {
        self.connections
            .get(handle)
            .and_then(|c| c.connected_proxy_type())
    }

    pub fn get_far_id(&self, handle: NetConnectionHandle) -> Option<&'static str> {
        self.connections.get(handle).and_then(|c| c.far_id())
    }

    pub fn get_far_nonce(&self, handle: NetConnectionHandle) -> Option<&'static str> {
        self.connections.get(handle).and_then(|c| c.far_nonce())
    }

    pub fn get_near_id(&self, handle: NetConnectionHandle) -> Option<&'static str> {
        self.connections.get(handle).and_then(|c| c.near_id())
    }

    pub fn get_near_nonce(&self, handle: NetConnectionHandle) -> Option<&'static str> {
        self.connections.get(handle).and_then(|c| c.near_nonce())
    }

    pub fn get_protocol(&self, handle: NetConnectionHandle) -> Option<&'static str> {
        self.connections.get(handle).and_then(|c| c.protocol())
    }

    pub fn get_uri(&self, handle: NetConnectionHandle) -> Option<String> {
        self.connections.get(handle).and_then(|c| c.uri())
    }

    pub fn is_using_tls(&self, handle: NetConnectionHandle) -> Option<bool> {
        self.connections.get(handle).and_then(|c| c.using_tls())
    }
}

#[derive(Collect)]
#[collect(no_drop)]
pub struct NetConnection<'gc> {
    object: NetConnectionObject<'gc>,

    #[collect(require_static)]
    protocol: NetConnectionProtocol,
}

impl<'gc> NetConnection<'gc> {
    pub fn is_connected(&self) -> bool {
        match self.protocol {
            NetConnectionProtocol::Local => true,
            NetConnectionProtocol::FlashRemoting(_) => false,
        }
    }

    pub fn connected_proxy_type(&self) -> Option<&'static str> {
        match self.protocol {
            NetConnectionProtocol::Local => Some("none"),
            NetConnectionProtocol::FlashRemoting(_) => None,
        }
    }

    pub fn far_id(&self) -> Option<&'static str> {
        match self.protocol {
            NetConnectionProtocol::Local => Some(""),
            NetConnectionProtocol::FlashRemoting(_) => None,
        }
    }

    pub fn far_nonce(&self) -> Option<&'static str> {
        match self.protocol {
            NetConnectionProtocol::Local => {
                Some("0000000000000000000000000000000000000000000000000000000000000000")
            }
            NetConnectionProtocol::FlashRemoting(_) => None,
        }
    }

    pub fn near_id(&self) -> Option<&'static str> {
        match self.protocol {
            NetConnectionProtocol::Local => Some(""),
            NetConnectionProtocol::FlashRemoting(_) => None,
        }
    }

    pub fn near_nonce(&self) -> Option<&'static str> {
        match self.protocol {
            NetConnectionProtocol::Local => {
                Some("0000000000000000000000000000000000000000000000000000000000000000")
            }
            NetConnectionProtocol::FlashRemoting(_) => None,
        }
    }

    pub fn protocol(&self) -> Option<&'static str> {
        match self.protocol {
            NetConnectionProtocol::Local => Some("rtmp"),
            NetConnectionProtocol::FlashRemoting(_) => None,
        }
    }

    pub fn uri(&self) -> Option<String> {
        match &self.protocol {
            NetConnectionProtocol::Local => Some("null".to_string()), // Yes, it's a string "null", not a real null.
            NetConnectionProtocol::FlashRemoting(remoting) => Some(remoting.url.to_string()),
        }
    }

    pub fn using_tls(&self) -> Option<bool> {
        match &self.protocol {
            NetConnectionProtocol::Local => Some(false),
            NetConnectionProtocol::FlashRemoting(_) => None,
        }
    }

    pub fn send(
        &mut self,
        command: String,
        responder_handle: Option<ResponderHandle>,
        message: AmfValue,
    ) {
        match &mut self.protocol {
            NetConnectionProtocol::Local => {}
            NetConnectionProtocol::FlashRemoting(remoting) => {
                remoting.send(command, responder_handle, message)
            }
        }
    }

    pub fn update(
        &mut self,
        self_handle: NetConnectionHandle,
        navigator: &mut dyn NavigatorBackend,
        player: Weak<Mutex<Player>>,
    ) {
        match &mut self.protocol {
            NetConnectionProtocol::Local => {}
            NetConnectionProtocol::FlashRemoting(remoting) => {
                if remoting.has_pending_packet() {
                    navigator.spawn_future(remoting.flush_queue(self_handle, player));
                }
            }
        }
    }

    pub fn set_header(&mut self, header: Header) {
        match &mut self.protocol {
            NetConnectionProtocol::Local => {}
            NetConnectionProtocol::FlashRemoting(remoting) => {
                remoting.set_header(header);
            }
        }
    }
}

#[derive(Debug)]
pub enum NetConnectionProtocol {
    /// A "local" connection, caused by connecting to null
    Local,

    /// Flash Remoting protocol, caused by connecting to a `http://` address.
    FlashRemoting(FlashRemoting),
}

#[derive(Debug)]
pub struct FlashRemoting {
    url: String,
    headers: Vec<Header>,
    outgoing_queue: Vec<(Message, Option<ResponderHandle>)>,
}

impl FlashRemoting {
    pub fn send(
        &mut self,
        command: String,
        responder_handle: Option<ResponderHandle>,
        message: AmfValue,
    ) {
        self.outgoing_queue.push((
            Message {
                target_uri: command,
                response_uri: format!("/{}", self.outgoing_queue.len() + 1), // Flash is 1-based... simplifies tests to stay the same
                contents: Rc::new(message),
            },
            responder_handle,
        ));
    }

    pub fn has_pending_packet(&self) -> bool {
        !self.outgoing_queue.is_empty()
    }

    pub fn set_header(&mut self, header: Header) {
        // Only one header of the same name (case insensitive) should exist
        self.headers
            .retain(|h| !h.name.eq_ignore_ascii_case(&header.name));

        self.headers.push(header);
    }

    pub fn flush_queue(
        &mut self,
        self_handle: NetConnectionHandle,
        player: Weak<Mutex<Player>>,
    ) -> OwnedFuture<(), Error> {
        let queue = std::mem::take(&mut self.outgoing_queue);
        let (messages, responder_handles): (Vec<_>, Vec<_>) = queue.into_iter().unzip();
        let packet = Packet {
            version: AMFVersion::AMF0,
            headers: self.headers.clone(),
            messages,
        };
        let url = self.url.clone();

        Box::pin(async move {
            let player = player
                .upgrade()
                .expect("Could not upgrade weak reference to player");
            let bytes = flash_lso::packet::write::write_to_bytes(&packet, true)
                .expect("Must be able to serialize a packet");
            let request = Request::post(url, Some((bytes, "application/x-amf".to_string())));
            let fetch = player.lock().unwrap().navigator().fetch(request);
            let response: Result<_, ErrorResponse> = async {
                let response = fetch.await?;
                let url = response.url().to_string();
                let body = response
                    .body()
                    .await
                    .map_err(|error| ErrorResponse { url, error })?;

                Ok(body)
            }
            .await;
            let response = match response {
                Ok(response) => response,
                Err(response) => {
                    player.lock().unwrap().update(|uc| {
                        tracing::error!(
                            "Couldn't submit AMF Packet to {}: {:?}",
                            response.url,
                            response.error
                        );
                        if let Some(connection) = uc.net_connections.connections.get(self_handle) {
                            match connection.object {
                                NetConnectionObject::Avm2(object) => {
                                    let mut activation = Avm2Activation::from_nothing(uc);
                                    let url = AvmString::new_utf8(
                                        activation.context.gc_context,
                                        response.url,
                                    );
                                    let event = Avm2EventObject::net_status_event(
                                        &mut activation,
                                        "netStatus",
                                        vec![
                                            ("code", "NetConnection.Call.Failed".into()),
                                            ("level", "error".into()),
                                            ("details", url),
                                            ("description", "HTTP: Failed".into()),
                                        ],
                                    );
                                    Avm2::dispatch_event(activation.context, event, object.into());
                                }
                                NetConnectionObject::Avm1(object) => {
                                    if let Err(e) =
                                        Avm1NetConnectionObject::on_empty_status_event(uc, object)
                                    {
                                        tracing::error!(
                                            "Unhandled error sending connection callback: {e}"
                                        );
                                    }
                                }
                            }
                        }
                    });
                    return Ok(());
                }
            };

            // Flash completely ignores invalid responses, it seems
            if let Ok(response_packet) = flash_lso::packet::read::parse(&response) {
                player.lock().unwrap().update(|uc| {
                    for message in response_packet.messages {
                        if let Some(target_uri) = message.target_uri.strip_prefix('/') {
                            let mut responder = None;
                            if let Some(index) = target_uri
                                .strip_suffix("/onStatus")
                                .and_then(|str| str::parse::<usize>(str).ok())
                            {
                                responder = responder_handles
                                    .get(index.wrapping_sub(1))
                                    .cloned()
                                    .flatten()
                                    .map(|handle| (handle, ResponderCallback::Status));
                            } else if let Some(index) = target_uri
                                .strip_suffix("/onResult")
                                .and_then(|str| str::parse::<usize>(str).ok())
                            {
                                responder = responder_handles
                                    .get(index.wrapping_sub(1))
                                    .cloned()
                                    .flatten()
                                    .map(|handle| (handle, ResponderCallback::Result));
                            }

                            if let Some((responder_handle, callback)) = responder {
                                responder_handle.call(uc, callback, message.contents);
                            }
                        }
                    }
                });
            }

            Ok(())
        })
    }
}
