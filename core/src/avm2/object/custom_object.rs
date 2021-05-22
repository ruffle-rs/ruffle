//! Custom object macro

#[macro_export]
macro_rules! impl_avm2_custom_object_properties {
    ($field:ident) => {
        fn get_property_local(
            self,
            receiver: Object<'gc>,
            name: &QName<'gc>,
            activation: &mut Activation<'_, 'gc, '_>,
        ) -> Result<Value<'gc>, Error> {
            let read = self.0.read();
            let rv = read.$field.get_property_local(receiver, name, activation)?;

            drop(read);

            rv.resolve(activation)
        }

        fn set_property_local(
            self,
            receiver: Object<'gc>,
            name: &QName<'gc>,
            value: Value<'gc>,
            activation: &mut Activation<'_, 'gc, '_>,
        ) -> Result<(), Error> {
            let mut write = self.0.write(activation.context.gc_context);
            let rv = write
                .$field
                .set_property_local(receiver, name, value, activation)?;

            drop(write);

            rv.resolve(activation)?;

            Ok(())
        }

        fn init_property_local(
            self,
            receiver: Object<'gc>,
            name: &QName<'gc>,
            value: Value<'gc>,
            activation: &mut Activation<'_, 'gc, '_>,
        ) -> Result<(), Error> {
            let mut write = self.0.write(activation.context.gc_context);
            let rv = write
                .$field
                .init_property_local(receiver, name, value, activation)?;

            drop(write);

            rv.resolve(activation)?;

            Ok(())
        }

        fn is_property_overwritable(
            self,
            gc_context: MutationContext<'gc, '_>,
            name: &QName<'gc>,
        ) -> bool {
            self.0
                .write(gc_context)
                .$field
                .is_property_overwritable(name)
        }

        fn delete_property(
            &self,
            gc_context: MutationContext<'gc, '_>,
            multiname: &QName<'gc>,
        ) -> bool {
            self.0.write(gc_context).$field.delete_property(multiname)
        }

        fn has_own_property(self, name: &QName<'gc>) -> Result<bool, Error> {
            self.0.read().$field.has_own_property(name)
        }

        fn resolve_any(self, local_name: AvmString<'gc>) -> Result<Option<Namespace<'gc>>, Error> {
            self.0.read().$field.resolve_any(local_name)
        }

        fn resolve_any_trait(
            self,
            local_name: AvmString<'gc>,
        ) -> Result<Option<Namespace<'gc>>, Error> {
            self.0.read().$field.resolve_any_trait(local_name)
        }
    };
}

#[macro_export]
macro_rules! impl_avm2_custom_object {
    ($field:ident) => {
        fn has_slot_local(self, id: u32) -> bool {
            self.0.read().$field.has_slot_local(id)
        }

        fn get_slot_local(self, id: u32) -> Result<Value<'gc>, Error> {
            self.0.read().$field.get_slot_local(id)
        }

        fn set_slot_local(
            self,
            id: u32,
            value: Value<'gc>,
            mc: MutationContext<'gc, '_>,
        ) -> Result<(), Error> {
            self.0.write(mc).$field.set_slot_local(id, value, mc)
        }

        fn init_slot_local(
            self,
            id: u32,
            value: Value<'gc>,
            mc: MutationContext<'gc, '_>,
        ) -> Result<(), Error> {
            self.0.write(mc).$field.init_slot_local(id, value, mc)
        }

        fn get_method(self, id: u32) -> Option<Object<'gc>> {
            self.0.read().$field.get_method(id)
        }

        fn get_trait(self, name: &QName<'gc>) -> Result<Vec<Trait<'gc>>, Error> {
            self.0.read().$field.get_trait(name)
        }

        fn get_trait_slot(self, id: u32) -> Result<Option<Trait<'gc>>, Error> {
            self.0.read().$field.get_trait_slot(id)
        }

        fn get_scope(self) -> Option<GcCell<'gc, Scope<'gc>>> {
            self.0.read().$field.get_scope()
        }

        fn has_trait(self, name: &QName<'gc>) -> Result<bool, Error> {
            self.0.read().$field.has_trait(name)
        }

        fn has_instantiated_property(self, name: &QName<'gc>) -> bool {
            self.0.read().$field.has_instantiated_property(name)
        }

        fn has_own_virtual_getter(self, name: &QName<'gc>) -> bool {
            self.0.read().$field.has_own_virtual_getter(name)
        }

        fn has_own_virtual_setter(self, name: &QName<'gc>) -> bool {
            self.0.read().$field.has_own_virtual_setter(name)
        }

        fn proto(&self) -> Option<Object<'gc>> {
            self.0.read().$field.proto()
        }

        fn set_proto(self, mc: MutationContext<'gc, '_>, proto: Object<'gc>) {
            self.0.write(mc).$field.set_proto(proto)
        }

        fn get_enumerant_name(&self, index: u32) -> Option<QName<'gc>> {
            self.0.read().$field.get_enumerant_name(index)
        }

        fn property_is_enumerable(&self, name: &QName<'gc>) -> bool {
            self.0.read().$field.property_is_enumerable(name)
        }

        fn set_local_property_is_enumerable(
            &self,
            mc: MutationContext<'gc, '_>,
            name: &QName<'gc>,
            is_enumerable: bool,
        ) -> Result<(), Error> {
            self.0
                .write(mc)
                .$field
                .set_local_property_is_enumerable(name, is_enumerable)
        }

        fn as_ptr(&self) -> *const ObjectPtr {
            self.0.as_ptr() as *const ObjectPtr
        }

        fn as_class(&self) -> Option<GcCell<'gc, Class<'gc>>> {
            self.0.read().base.as_class()
        }

        fn as_constr(&self) -> Option<Object<'gc>> {
            self.0.read().base.as_constr()
        }

        fn set_constr(self, mc: MutationContext<'gc, '_>, constr: Object<'gc>) {
            self.0.write(mc).base.set_constr(constr);
        }

        fn install_method(
            &mut self,
            mc: MutationContext<'gc, '_>,
            name: QName<'gc>,
            disp_id: u32,
            function: Object<'gc>,
        ) {
            self.0
                .write(mc)
                .$field
                .install_method(name, disp_id, function)
        }

        fn install_getter(
            &mut self,
            mc: MutationContext<'gc, '_>,
            name: QName<'gc>,
            disp_id: u32,
            function: Object<'gc>,
        ) -> Result<(), Error> {
            self.0
                .write(mc)
                .$field
                .install_getter(name, disp_id, function)
        }

        fn install_setter(
            &mut self,
            mc: MutationContext<'gc, '_>,
            name: QName<'gc>,
            disp_id: u32,
            function: Object<'gc>,
        ) -> Result<(), Error> {
            self.0
                .write(mc)
                .$field
                .install_setter(name, disp_id, function)
        }

        fn install_dynamic_property(
            &mut self,
            mc: MutationContext<'gc, '_>,
            name: QName<'gc>,
            value: Value<'gc>,
        ) -> Result<(), Error> {
            self.0
                .write(mc)
                .$field
                .install_dynamic_property(name, value)
        }

        fn install_slot(
            &mut self,
            mc: MutationContext<'gc, '_>,
            name: QName<'gc>,
            id: u32,
            value: Value<'gc>,
        ) {
            self.0.write(mc).$field.install_slot(name, id, value)
        }

        fn install_const(
            &mut self,
            mc: MutationContext<'gc, '_>,
            name: QName<'gc>,
            id: u32,
            value: Value<'gc>,
        ) {
            self.0.write(mc).$field.install_const(name, id, value)
        }

        fn interfaces(&self) -> Vec<Object<'gc>> {
            self.0.read().$field.interfaces()
        }

        fn set_interfaces(
            &self,
            gc_context: MutationContext<'gc, '_>,
            iface_list: Vec<Object<'gc>>,
        ) {
            self.0.write(gc_context).$field.set_interfaces(iface_list)
        }
    };
}
