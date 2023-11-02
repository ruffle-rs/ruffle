use crate::avm2::object::NetConnectionObject as Avm2NetConnectionObject;
use crate::avm2::{Activation as Avm2Activation, Avm2, EventObject as Avm2EventObject};
use crate::context::UpdateContext;
use gc_arena::Collect;
use generational_arena::{Arena, Index};

pub type NetConnectionHandle = Index;

#[derive(Copy, Clone, Collect)]
#[collect(no_drop)]
pub enum NetConnectionObject<'gc> {
    Avm2(Avm2NetConnectionObject<'gc>),
}

impl<'gc> NetConnectionObject<'gc> {
    pub fn set_handle(&self, handle: Option<NetConnectionHandle>) -> Option<NetConnectionHandle> {
        match self {
            NetConnectionObject::Avm2(object) => object.set_handle(handle),
        }
    }
}

impl<'gc> From<Avm2NetConnectionObject<'gc>> for NetConnectionObject<'gc> {
    fn from(value: Avm2NetConnectionObject<'gc>) -> Self {
        NetConnectionObject::Avm2(value)
    }
}

/// Manages the collection of NetConnections.
pub struct NetConnections<'gc> {
    connections: Arena<NetConnection<'gc>>,
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
            connections: Arena::new(),
        }
    }
}

impl<'gc> NetConnections<'gc> {
    pub fn connect_to_local<O: Into<NetConnectionObject<'gc>>>(
        context: &mut UpdateContext<'_, 'gc>,
        target: O,
    ) {
        let target = target.into();
        let connection = NetConnection {
            object: target,
            protocol: NetConnectionProtocol::Local,
        };
        let handle = context.net_connections.connections.insert(connection);

        if let Some(existing_handle) = target.set_handle(Some(handle)) {
            NetConnections::close(context, existing_handle)
        }

        match target {
            NetConnectionObject::Avm2(object) => {
                let mut activation = Avm2Activation::from_nothing(context.reborrow());
                let event = Avm2EventObject::net_status_event(
                    &mut activation,
                    "netStatus",
                    vec![
                        ("code", "NetConnection.Connect.Success"),
                        ("level", "status"),
                    ],
                );
                Avm2::dispatch_event(&mut activation.context, event, object.into());
            }
        }
    }

    pub fn connect_to_flash_remoting<O: Into<NetConnectionObject<'gc>>>(
        context: &mut UpdateContext<'_, 'gc>,
        target: O,
        url: String,
    ) {
        let target = target.into();
        let connection = NetConnection {
            object: target,
            protocol: NetConnectionProtocol::FlashRemoting(FlashRemoting { url }),
        };
        let handle = context.net_connections.connections.insert(connection);

        if let Some(existing_handle) = target.set_handle(Some(handle)) {
            NetConnections::close(context, existing_handle)
        }

        // No open event here
    }

    pub fn close(context: &mut UpdateContext<'_, 'gc>, handle: NetConnectionHandle) {
        let Some(connection) = context.net_connections.connections.remove(handle) else {
            return;
        };

        match connection.object {
            NetConnectionObject::Avm2(object) => {
                let mut activation = Avm2Activation::from_nothing(context.reborrow());
                let event = Avm2EventObject::net_status_event(
                    &mut activation,
                    "netStatus",
                    vec![
                        ("code", "NetConnection.Connect.Closed"),
                        ("level", "status"),
                    ],
                );
                Avm2::dispatch_event(&mut activation.context, event, object.into());

                if matches!(connection.protocol, NetConnectionProtocol::FlashRemoting(_)) {
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
                    Avm2::dispatch_event(&mut activation.context, event, object.into());
                }
            }
        }
    }
}

#[derive(Collect)]
#[collect(no_drop)]
pub struct NetConnection<'gc> {
    object: NetConnectionObject<'gc>,

    #[collect(require_static)]
    protocol: NetConnectionProtocol,
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
    #[allow(dead_code)]
    url: String,
}
