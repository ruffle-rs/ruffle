#[macro_export]
macro_rules! impl_custom_object_without_set {
    ($field:ident) => {
        fn get_local(
            &self,
            name: &str,
            activation: &mut crate::avm1::Activation<'_, 'gc, '_>,
            this: crate::avm1::Object<'gc>,
        ) -> Result<crate::avm1::Value<'gc>, crate::avm1::Error<'gc>> {
            self.0.read().$field.get_local(name, activation, this)
        }

        fn call(
            &self,
            name: &str,
            activation: &mut crate::avm1::Activation<'_, 'gc, '_>,
            this: crate::avm1::Object<'gc>,
            base_proto: Option<crate::avm1::Object<'gc>>,
            args: &[crate::avm1::Value<'gc>],
        ) -> Result<crate::avm1::Value<'gc>, crate::avm1::Error<'gc>> {
            self.0
                .read()
                .$field
                .call(name, activation, this, base_proto, args)
        }

        fn call_setter(
            &self,
            name: &str,
            value: crate::avm1::Value<'gc>,
            activation: &mut crate::avm1::Activation<'_, 'gc, '_>,
        ) -> Option<crate::avm1::object::Object<'gc>> {
            self.0.read().$field.call_setter(name, value, activation)
        }

        fn delete(
            &self,
            activation: &mut crate::avm1::Activation<'_, 'gc, '_>,
            name: &str,
        ) -> bool {
            self.0.read().$field.delete(activation, name)
        }

        fn proto(&self) -> Option<crate::avm1::Object<'gc>> {
            self.0.read().$field.proto()
        }

        fn set_proto(
            &self,
            gc_context: gc_arena::MutationContext<'gc, '_>,
            prototype: Option<crate::avm1::Object<'gc>>,
        ) {
            self.0.read().$field.set_proto(gc_context, prototype);
        }

        fn define_value(
            &self,
            gc_context: gc_arena::MutationContext<'gc, '_>,
            name: &str,
            value: crate::avm1::Value<'gc>,
            attributes: enumset::EnumSet<crate::avm1::property::Attribute>,
        ) {
            self.0
                .read()
                .$field
                .define_value(gc_context, name, value, attributes)
        }

        fn set_attributes(
            &mut self,
            gc_context: gc_arena::MutationContext<'gc, '_>,
            name: Option<&str>,
            set_attributes: enumset::EnumSet<crate::avm1::property::Attribute>,
            clear_attributes: enumset::EnumSet<crate::avm1::property::Attribute>,
        ) {
            self.0.write(gc_context).$field.set_attributes(
                gc_context,
                name,
                set_attributes,
                clear_attributes,
            )
        }

        fn add_property(
            &self,
            gc_context: gc_arena::MutationContext<'gc, '_>,
            name: &str,
            get: crate::avm1::object::Object<'gc>,
            set: Option<crate::avm1::object::Object<'gc>>,
            attributes: enumset::EnumSet<crate::avm1::property::Attribute>,
        ) {
            self.0
                .read()
                .$field
                .add_property(gc_context, name, get, set, attributes)
        }

        fn add_property_with_case(
            &self,
            activation: &mut crate::avm1::Activation<'_, 'gc, '_>,
            gc_context: gc_arena::MutationContext<'gc, '_>,
            name: &str,
            get: crate::avm1::object::Object<'gc>,
            set: Option<crate::avm1::object::Object<'gc>>,
            attributes: enumset::EnumSet<crate::avm1::property::Attribute>,
        ) {
            self.0
                .read()
                .$field
                .add_property_with_case(activation, gc_context, name, get, set, attributes)
        }

        fn has_property(
            &self,
            activation: &mut crate::avm1::Activation<'_, 'gc, '_>,
            name: &str,
        ) -> bool {
            self.0.read().$field.has_property(activation, name)
        }

        fn has_own_property(
            &self,
            activation: &mut crate::avm1::Activation<'_, 'gc, '_>,
            name: &str,
        ) -> bool {
            self.0.read().$field.has_own_property(activation, name)
        }

        fn has_own_virtual(
            &self,
            activation: &mut crate::avm1::Activation<'_, 'gc, '_>,
            name: &str,
        ) -> bool {
            self.0.read().$field.has_own_virtual(activation, name)
        }

        fn is_property_enumerable(
            &self,
            activation: &mut crate::avm1::Activation<'_, 'gc, '_>,
            name: &str,
        ) -> bool {
            self.0
                .read()
                .$field
                .is_property_enumerable(activation, name)
        }

        fn get_keys(&self, activation: &mut crate::avm1::Activation<'_, 'gc, '_>) -> Vec<String> {
            self.0.read().$field.get_keys(activation)
        }

        fn as_string(&self) -> std::borrow::Cow<str> {
            std::borrow::Cow::Owned(self.0.read().$field.as_string().into_owned())
        }

        fn type_of(&self) -> &'static str {
            self.0.read().$field.type_of()
        }

        fn interfaces(&self) -> Vec<crate::avm1::Object<'gc>> {
            self.0.read().$field.interfaces()
        }

        fn set_interfaces(
            &mut self,
            gc_context: gc_arena::MutationContext<'gc, '_>,
            iface_list: Vec<crate::avm1::Object<'gc>>,
        ) {
            self.0
                .write(gc_context)
                .$field
                .set_interfaces(gc_context, iface_list)
        }

        fn as_script_object(&self) -> Option<ScriptObject<'gc>> {
            Some(self.0.read().$field)
        }

        fn as_ptr(&self) -> *const crate::avm1::ObjectPtr {
            self.0.as_ptr() as *const crate::avm1::ObjectPtr
        }

        fn length(&self) -> usize {
            self.0.read().$field.length()
        }

        fn array(&self) -> Vec<crate::avm1::Value<'gc>> {
            self.0.read().$field.array()
        }

        fn set_length(&self, gc_context: gc_arena::MutationContext<'gc, '_>, length: usize) {
            self.0.read().$field.set_length(gc_context, length)
        }

        fn array_element(&self, index: usize) -> crate::avm1::Value<'gc> {
            self.0.read().$field.array_element(index)
        }

        fn set_array_element(
            &self,
            index: usize,
            value: crate::avm1::Value<'gc>,
            gc_context: gc_arena::MutationContext<'gc, '_>,
        ) -> usize {
            self.0
                .read()
                .$field
                .set_array_element(index, value, gc_context)
        }

        fn delete_array_element(
            &self,
            index: usize,
            gc_context: gc_arena::MutationContext<'gc, '_>,
        ) {
            self.0.read().$field.delete_array_element(index, gc_context)
        }

        fn set_watcher(
            &self,
            activation: &mut crate::avm1::Activation<'_, 'gc, '_>,
            gc_context: gc_arena::MutationContext<'gc, '_>,
            name: std::borrow::Cow<str>,
            callback: crate::avm1::object::Object<'gc>,
            user_data: crate::avm1::Value<'gc>,
        ) {
            self.0
                .read()
                .$field
                .set_watcher(activation, gc_context, name, callback, user_data);
        }

        fn remove_watcher(
            &self,
            activation: &mut crate::avm1::Activation<'_, 'gc, '_>,
            gc_context: gc_arena::MutationContext<'gc, '_>,
            name: std::borrow::Cow<str>,
        ) -> bool {
            self.0
                .read()
                .$field
                .remove_watcher(activation, gc_context, name)
        }
    };
}

#[macro_export]
macro_rules! impl_custom_object {
    ($field:ident) => {
        crate::impl_custom_object_without_set!($field);

        fn set(
            &self,
            name: &str,
            value: crate::avm1::Value<'gc>,
            activation: &mut crate::avm1::Activation<'_, 'gc, '_>,
        ) -> Result<(), crate::avm1::Error<'gc>> {
            self.0.read().$field.set(name, value, activation)
        }
    };
}
