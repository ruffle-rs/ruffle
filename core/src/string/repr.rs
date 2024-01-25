use std::cell::Cell;
use std::ops::Deref;

use gc_arena::Collect;
use ruffle_wstr::{ptr as wptr, wstr_impl_traits, WStr, WString};

use crate::string::avm_string::AvmString;

/// Internal representation of `AvmAtom`s and (owned) `AvmString`.
///
/// Using this type directly is dangerous, as it can be used to violate
/// the interning invariants.
#[derive(Collect)]
#[collect(unsafe_drop)]
pub struct AvmStringRepr<'gc> {
    #[collect(require_static)]
    ptr: *mut (),

    #[collect(require_static)]
    meta: wptr::WStrMetadata,

    // We abuse the 'is_wide' bit for interning.
    #[collect(require_static)]
    capacity: Cell<wptr::WStrMetadata>,

    // If Some, the string is dependent.
    owner: Option<AvmString<'gc>>,
}

impl<'gc> AvmStringRepr<'gc> {
    pub fn from_raw(s: WString, interned: bool) -> Self {
        let (ptr, meta, cap) = s.into_raw_parts();
        let capacity = Cell::new(wptr::WStrMetadata::new32(cap, interned));
        Self {
            ptr,
            meta,
            capacity,
            owner: None,
        }
    }

    pub fn new_dependent(s: AvmString<'gc>, start: usize, end: usize) -> Self {
        let wstr = &s[start..end];
        let wstr_ptr = wstr as *const WStr;

        let meta = unsafe { wptr::WStrMetadata::of(wstr_ptr) };
        // Dependent strings are never interned
        let capacity = Cell::new(wptr::WStrMetadata::new32(meta.len32(), false));
        let ptr = wstr_ptr as *mut WStr as *mut ();

        Self {
            owner: Some(s),
            ptr,
            meta,
            capacity,
        }
    }

    #[inline]
    pub fn is_dependent(&self) -> bool {
        self.owner.is_some()
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
        if self.is_dependent() {
            panic!("bug: we interned a dependent string");
        }
        let cap = self.capacity.get();
        let new_cap = wptr::WStrMetadata::new32(cap.len32(), true);
        self.capacity.set(new_cap);
    }
}

impl<'gc> Drop for AvmStringRepr<'gc> {
    fn drop(&mut self) {
        if self.owner.is_none() {
            // SAFETY: we drop the `WString` we logically own.
            unsafe {
                let cap = self.capacity.get().len32();
                let _ = WString::from_raw_parts(self.ptr, self.meta, cap);
            }
        }
    }
}

impl<'gc> Deref for AvmStringRepr<'gc> {
    type Target = WStr;
    #[inline]
    fn deref(&self) -> &WStr {
        self.as_wstr()
    }
}

impl<'gc> Default for AvmStringRepr<'gc> {
    #[inline]
    fn default() -> Self {
        Self::from_raw(WString::new(), false)
    }
}

wstr_impl_traits!(impl['gc] for AvmStringRepr<'gc>);
