use gc_arena::{Collect, Gc, MutationContext};
use std::ops::Deref;

#[derive(Debug, Clone, Collect)]
#[collect(no_drop)]
pub struct Avm1String<'gc>(Gc<'gc, String>);

impl<'gc> Avm1String<'gc> {
    pub fn new<S: Into<String>>(gc_context: MutationContext<'gc, '_>, string: S) -> Self {
        Self(Gc::allocate(gc_context, string.into()))
    }

    pub fn as_str(&self) -> &str {
        self
    }
}

impl Deref for Avm1String<'_> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.0.deref()
    }
}

impl AsRef<str> for Avm1String<'_> {
    #[inline]
    fn as_ref(&self) -> &str {
        self
    }
}

macro_rules! impl_eq {
    ($lhs:ty, $rhs: ty) => {
        #[allow(unused_lifetimes)]
        impl<'a, 'b> PartialEq<$rhs> for $lhs {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool {
                PartialEq::eq(&self[..], &other[..])
            }
        }

        #[allow(unused_lifetimes)]
        impl<'a, 'b> PartialEq<$lhs> for $rhs {
            #[inline]
            fn eq(&self, other: &$lhs) -> bool {
                PartialEq::eq(&self[..], &other[..])
            }
        }
    };
}

impl_eq! { Avm1String<'_>, str }
impl_eq! { Avm1String<'_>, &'a str }
impl_eq! { Avm1String<'_>, String }
