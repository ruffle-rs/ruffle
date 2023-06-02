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
            activation: &mut $crate::avm1::Activation<'_, 'gc>,
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
