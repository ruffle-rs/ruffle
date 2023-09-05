use crate::avm2::object::Object;
use crate::context::UpdateContext;
use crate::events::PermissionStatus;
use gc_arena::{Collect, GcCell, Mutation};
pub trait GeolocationBackend {
    /// Gets an availability status of a geolocation sensor on the device
    fn is_geolocation_supported(&mut self) -> bool;

    /// Requests a permission to use a geolocation sensor on the frontend
    fn request_geolocation_permission(&self);

    /// Gets current permission status
    fn geolocation_permission_status(&self) -> PermissionStatus;

    /// Sets current permission status from the frontend to be used inside
    fn set_geolocation_permission_status(&mut self, status: String);

    /// Gets a requested update time interval in ms
    fn geolocation_update_interval(&self) -> f64;

    /// Sets a requested update time interval in ms
    fn set_geolocation_update_interval(&mut self, interval: f64);
}

// TODO: Currently this struct accepts only one instance of Geolocation class
#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct GeolocationInstances<'gc>(GcCell<'gc, Option<Object<'gc>>>);

impl<'gc> GeolocationInstances<'gc> {
    pub fn new(gc_context: &Mutation<'gc>) -> Self {
        Self(GcCell::new(gc_context, None))
    }

    pub fn get(&self) -> Option<Object<'gc>> {
        *self.0.read()
    }

    pub fn set(&self, new: Object<'gc>, context: &mut UpdateContext<'_, 'gc>) {
        let _ = std::mem::replace(&mut *self.0.write(context.gc_context), Some(new));
    }
}

#[derive(Default)]
pub struct NullGeolocationBackend {}

impl NullGeolocationBackend {
    pub fn new() -> Self {
        Self::default()
    }
}

impl GeolocationBackend for NullGeolocationBackend {
    fn is_geolocation_supported(&mut self) -> bool {
        false
    }

    fn request_geolocation_permission(&self) {}

    fn geolocation_permission_status(&self) -> PermissionStatus {
        PermissionStatus::Unknown
    }

    fn set_geolocation_permission_status(&mut self, _: String) {}

    fn geolocation_update_interval(&self) -> f64 {
        0.0
    }

    fn set_geolocation_update_interval(&mut self, _: f64) {}
}
