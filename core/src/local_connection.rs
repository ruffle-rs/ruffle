use crate::avm1::Object as Avm1Object;
use crate::avm2::object::LocalConnectionObject;
use crate::string::AvmString;
use gc_arena::Collect;
use slotmap::{new_key_type, SlotMap};

new_key_type! {
    pub struct LocalConnectionHandle;
}

#[derive(Collect)]
#[collect(no_drop)]
pub enum LocalConnectionKind<'gc> {
    Avm2(LocalConnectionObject<'gc>),
    Avm1(Avm1Object<'gc>),
}

impl<'gc> From<LocalConnectionObject<'gc>> for LocalConnectionKind<'gc> {
    fn from(obj: LocalConnectionObject<'gc>) -> Self {
        Self::Avm2(obj)
    }
}

#[derive(Collect)]
#[collect(no_drop)]
pub struct LocalConnection<'gc> {
    object: LocalConnectionKind<'gc>,

    connection_name: AvmString<'gc>,
}

impl<'gc> LocalConnection<'gc> {
    pub fn new(
        object: impl Into<LocalConnectionKind<'gc>>,
        connection_name: AvmString<'gc>,
    ) -> Self {
        Self {
            object: object.into(),
            connection_name,
        }
    }
}

/// Manages the collection of local connections.
pub struct LocalConnections<'gc> {
    connections: SlotMap<LocalConnectionHandle, LocalConnection<'gc>>,
}

unsafe impl<'gc> Collect for LocalConnections<'gc> {
    fn trace(&self, cc: &gc_arena::Collection) {
        for (_, connection) in self.connections.iter() {
            connection.trace(cc)
        }
    }
}

impl<'gc> LocalConnections<'gc> {
    pub fn empty() -> Self {
        Self {
            connections: SlotMap::with_key(),
        }
    }

    pub fn insert(&mut self, connection: LocalConnection<'gc>) -> LocalConnectionHandle {
        self.connections.insert(connection)
    }

    pub fn remove(&mut self, handle: LocalConnectionHandle) {
        self.connections.remove(handle);
    }

    pub fn all_by_name(&self, requested_name: AvmString<'gc>) -> Vec<&LocalConnection<'gc>> {
        let mut conns = Vec::new();
        for (_, connection) in self.connections.iter() {
            if connection.connection_name == requested_name {
                conns.push(connection);
            }
        }

        conns
    }
}
