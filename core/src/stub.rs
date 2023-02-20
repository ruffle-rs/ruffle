#[cfg(feature = "known_stubs")]
mod external {
    include!(concat!(env!("OUT_DIR"), "/actionscript_stubs.rs"));
}

#[cfg(feature = "known_stubs")]
pub fn get_known_stubs() -> fnv::FnvHashSet<&'static Stub> {
    let mut result = fnv::FnvHashSet::default();
    for stub in ruffle_render::stub::KNOWN_STUBS.iter() {
        result.insert(stub);
    }
    for stub in external::AS_DEFINED_STUBS {
        result.insert(stub);
    }
    result
}

pub use ruffle_render::stub::{Stub, StubCollection};
