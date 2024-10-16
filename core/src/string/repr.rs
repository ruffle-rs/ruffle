use std::cell::Cell;
use std::ops::Deref;

use gc_arena::{Collect, Gc};
use ruffle_wstr::{panic_on_invalid_length, ptr as wptr, wstr_impl_traits, WStr, WString};

/// Internal representation of `AvmAtom`s and (owned) `AvmString`s.
///
/// Using this type directly is dangerous, as it can be used to violate
/// the interning invariants.
#[derive(Collect)]
#[collect(unsafe_drop)]
pub struct AvmStringRepr<'gc> {
    #[collect(require_static)]
    ptr: *mut (),

    // Length and is_wide bit.
    #[collect(require_static)]
    meta: wptr::WStrMetadata,

    // We abuse WStrMetadata to store capacity and is_interned bit.
    // If a string is Static or Dependent, the capacity should always be 0.
    capacity: Cell<wptr::WStrMetadata>,

    // If a string is Static or Dependent, this should always be 0.
    // If a string is Owned, this indicates used chars, including dependents.
    // Example: assume a string a="abc" has 10 bytes of capacity (chars_used=3).
    // Then, with a+"d", we produce a dependent string and owner's chars_used becomes 4.
    // len <= chars_used <= capacity.
    chars_used: Cell<u32>,

    // If Some, the string is Dependent. The owner is assumed to be non-dynamic.
    owner: Option<Gc<'gc, Self>>,
}

impl<'gc> AvmStringRepr<'gc> {
    pub fn from_raw(s: WString, interned: bool) -> Self {
        let (ptr, meta, cap) = s.into_raw_parts();
        let capacity = Cell::new(wptr::WStrMetadata::new32(cap, interned));
        Self {
            ptr,
            meta,
            capacity,
            chars_used: Cell::new(meta.len32()),
            owner: None,
        }
    }

    pub fn from_raw_static(s: &'static WStr, interned: bool) -> Self {
        // SAFETY: 'wstr' is a static WStr and doesn't require an owner to stay valid
        unsafe { Self::new_dependent_raw(None, s, interned) }
    }

    pub fn new_dependent(s: Gc<'gc, Self>, start: usize, end: usize) -> Self {
        let wstr = &s.as_ref()[start..end];
        let owner = Some(s.owner().unwrap_or(s));
        // Dependent strings are never interned
        let interned = false;
        // SAFETY: 'wstr' is a WStr pointing into 'owner'
        unsafe { Self::new_dependent_raw(owner, wstr, interned) }
    }

    unsafe fn new_dependent_raw(
        owner: Option<Gc<'gc, Self>>,
        wstr: &'gc WStr,
        interned: bool,
    ) -> Self {
        Self {
            owner,
            ptr: wstr as *const WStr as *mut (),
            meta: wptr::WStrMetadata::of(wstr),
            chars_used: Cell::new(0),
            capacity: Cell::new(wptr::WStrMetadata::new32(0, interned)),
        }
    }

    pub fn try_append_inline(left: Gc<'gc, Self>, right: &WStr) -> Option<Self> {
        // note: we could also in-place append a byte string to a wide string
        // But it was skipped for now.
        if left.is_wide() != right.is_wide() {
            return None;
        }

        let left_origin = left.owner().unwrap_or(left);
        let char_size = if left.is_wide() { 2 } else { 1 };
        /*
            assumptions:
            - left.len <= left.chars_used <= left.capacity
            - left_ptr is inside left_origin_ptr .. left_origin_ptr + left.chars_used

            note: it's possible that left == left_origin.
        */
        unsafe {
            let left_origin_ptr = left_origin.ptr as *const u8;
            let left_ptr = left.ptr as *const u8;

            /*
            Assume a="abc", b=a+"d", c=a.substr(1), we're running d=c+"e"

            a          ->  abc
            b          ->  abcd
            c          ->   bc        v left_capacity_end
            a's memory ->  abcd_______
                                ^ first_requested
                                ^ first_available

            We can only append in-place if first_requested and first_available match
            And we have enough spare capacity.
            */

            let first_available =
                left_origin_ptr.add(char_size * left_origin.chars_used.get() as usize);
            let first_requested = left_ptr.add(char_size * left.len());

            let mut chars_available = 0;
            if first_available == first_requested {
                let left_capacity_end =
                    left_origin_ptr.add(char_size * left_origin.capacity.get().len());
                chars_available =
                    ((left_capacity_end as usize) - (first_available as usize)) / char_size;
            }
            if chars_available >= right.len() {
                let first_available = first_available as *mut u8;
                let right_ptr = right as *const WStr as *const () as *const u8;
                std::ptr::copy_nonoverlapping(right_ptr, first_available, char_size * right.len());

                let new_chars_used: usize = left_origin.chars_used.get() as usize + right.len();
                if new_chars_used >= u32::MAX as usize {
                    // This isn't really about the string length,
                    // but it's close enough?
                    panic_on_invalid_length(new_chars_used);
                }
                left_origin.chars_used.set(new_chars_used as u32);

                let new_len = left.len() + right.len();
                if new_len > WStr::MAX_LEN {
                    panic_on_invalid_length(new_len);
                }

                let new_wstr = wptr::from_raw_parts(
                    left_ptr as *const (),
                    wptr::WStrMetadata::new(new_len, left.is_wide()),
                );
                // Dependent strings are never interned.
                let ret = Self::new_dependent_raw(Some(left_origin), &*new_wstr, false);
                return Some(ret);
            }
        }

        None
    }

    #[inline]
    pub fn is_dependent(&self) -> bool {
        self.owner.is_some()
    }

    #[inline]
    pub fn owner(&self) -> Option<Gc<'gc, Self>> {
        self.owner
    }

    #[inline]
    pub fn as_wstr(&self) -> &WStr {
        // SAFETY: we own a `WString`.
        unsafe { &*wptr::from_raw_parts(self.ptr, self.meta) }
    }

    pub fn is_interned(&self) -> bool {
        self.capacity.get().is_wide()
    }

    pub(crate) fn mark_interned(&self) {
        if self.is_dependent() {
            panic!("bug: we interned a dependent string");
        }
        let cap = self.capacity.get();
        let new_cap = wptr::WStrMetadata::new32(cap.len32(), true);
        self.capacity.set(new_cap);
    }
}

impl Drop for AvmStringRepr<'_> {
    fn drop(&mut self) {
        let cap = self.capacity.get().len32();
        if cap > 0 {
            // SAFETY: we drop the `WString` we logically own.
            debug_assert!(self.owner.is_none());
            let _ = unsafe { WString::from_raw_parts(self.ptr, self.meta, cap) };
        } else {
            // Nothing to do, this is a Static or a Dependant string.
            // It could also have been an empty owned WString, but
            // these don't need to be dropped either.
        }
    }
}

impl Deref for AvmStringRepr<'_> {
    type Target = WStr;
    #[inline]
    fn deref(&self) -> &WStr {
        self.as_wstr()
    }
}

impl Default for AvmStringRepr<'_> {
    #[inline]
    fn default() -> Self {
        Self::from_raw(WString::new(), false)
    }
}

wstr_impl_traits!(impl['gc] for AvmStringRepr<'gc>);
