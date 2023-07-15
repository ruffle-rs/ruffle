use std::cell::Cell;
use std::ops::Deref;

use gc_arena::Collect;
use ruffle_wstr::{ptr as wptr, wstr_impl_traits, WStr, WString};

/// Internal representation of `AvmAtom`s and (owned) `AvmString`.
///
/// Using this type directly is dangerous, as it can be used to violate
/// the interning invariants.
#[derive(Collect)]
#[collect(require_static)]
pub struct AvmStringRepr {
    ptr: *mut (),
    meta: wptr::WStrMetadata,
    // We abuse the 'is_wide' bit for interning.
    capacity: Cell<wptr::WStrMetadata>,
}

impl AvmStringRepr {
    pub fn from_raw(s: WString, interned: bool) -> Self {
        let (ptr, meta, cap) = s.into_raw_parts();
        let capacity = Cell::new(wptr::WStrMetadata::new32(cap, interned));
        Self {
            ptr,
            meta,
            capacity,
        }
    }

    #[inline]
    pub fn as_wstr(&self) -> &WStr {
        // SAFETY: we own a `WString`.
        unsafe { &*wptr::from_raw_parts(self.ptr, self.meta) }
    }

    pub fn is_interned(&self) -> bool {
        self.capacity.get().is_wide()
    }

    pub fn mark_interned(&self) {
        let cap = self.capacity.get();
        let new_cap = wptr::WStrMetadata::new32(cap.len32(), true);
        self.capacity.set(new_cap);
    }
}

impl Drop for AvmStringRepr {
    fn drop(&mut self) {
        // SAFETY: we drop the `WString` we logically own.
        unsafe {
            let cap = self.capacity.get().len32();
            let _ = WString::from_raw_parts(self.ptr, self.meta, cap);
        }
    }
}

impl Deref for AvmStringRepr {
    type Target = WStr;
    #[inline]
    fn deref(&self) -> &WStr {
        self.as_wstr()
    }
}

impl Default for AvmStringRepr {
    #[inline]
    fn default() -> Self {
        Self::from_raw(WString::new(), false)
    }
}

wstr_impl_traits!(impl for AvmStringRepr);
