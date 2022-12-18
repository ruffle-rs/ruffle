#[macro_export]
macro_rules! impl_custom_object {
    ($field:ident) => {
        $crate::impl_custom_object!($field {});
    };

    (@extra $field:ident bare_object($as_obj:ident -> $obj_type:ident :: $new:ident)) => {
        fn $as_obj(&self) -> Option<$obj_type<'gc>> {
            Some(*self)
        }

        fn create_bare_object(
            &self,
            activation: &mut $crate::avm1::Activation<'_, 'gc, '_>,
            this: $crate::avm1::Object<'gc>,
        ) -> Result<$crate::avm1::Object<'gc>, $crate::avm1::Error<'gc>> {
            Ok($obj_type::$new(activation.context.gc_context, this).into())
        }
    };

    ($field:ident {
        $(
            $extra_name:ident($($extra:tt)*);
        )*
    }) => {
        $(
            $crate::impl_custom_object!(@extra $field $extra_name($($extra)*));
        )*

        fn raw_script_object(&self) -> ScriptObject<'gc> {
            self.0.read().$field
        }

        fn as_ptr(&self) -> *const $crate::avm1::ObjectPtr {
            self.0.as_ptr() as *const $crate::avm1::ObjectPtr
        }
    };
}

#[macro_export]
macro_rules! add_field_accessors {
    ($([$set_ident: ident, $get_ident: ident, $($var: ident).+, $type_: ty],)*) => {
        add_field_accessors!(
            $([$($var).+, $type_, set => $set_ident, get => $get_ident],)*
        );
    };

    ($([$($var: ident).+, $type_: ty $(, set => $set_ident: ident)? $(, get => $get_ident: ident)?],)*) => {
        $(
            add_field_accessors!([single $($var).+, $type_ $(, set => $set_ident)? $(, get => $get_ident)?]);
        )*
    };


    // This intermediate stage is here because I couldn't figure out how to make the nested
    // repetitions of $var and the optional $set_ident and $get_ident all expand correctly.
    ([single $($var: ident).+, $type_: ty, set => $set_ident: ident]) => {
        add_field_accessors!([setter_only $set_ident, $($var).+, $type_],);
    };
    ([single $($var: ident).+, $type_: ty, get => $get_ident: ident]) => {
        add_field_accessors!([getter_only $get_ident, $($var).+, $type_],);
    };
    ([single $($var: ident).+, $type_: ty, set => $set_ident: ident, get => $get_ident: ident]) => {
        add_field_accessors!([getter_only $get_ident, $($var).+, $type_],);
        add_field_accessors!([setter_only $set_ident, $($var).+, $type_],);
    };
    ([single $($var: ident).+, $type_: ty]) => {
        // nothing
    };


    ($([getter_only $get_ident: ident, $($var: ident).+, $type_: ty],)*) => {
        $(
            pub fn $get_ident(&self) -> $type_ {
                self.0.read().$($var).+
            }
        )*
    };

    ($([setter_only $set_ident: ident, $($var: ident).+, $type_: ty],)*) => {
        $(
            pub fn $set_ident(&self, gc_context: MutationContext<'gc, '_>, v: $type_) {
                self.0.write(gc_context).$($var).+ = v;
            }
        )*
    };
}
