//! AVM2 objects.

use crate::avm2::activation::Activation;
use crate::avm2::array::ArrayStorage;
use crate::avm2::bytearray::ByteArrayStorage;
use crate::avm2::class::Class;
use crate::avm2::domain::Domain;
use crate::avm2::error;
use crate::avm2::events::{DispatchList, Event};
use crate::avm2::property::Property;
use crate::avm2::regexp::RegExp;
use crate::avm2::value::{Hint, Value};
use crate::avm2::vector::VectorStorage;
use crate::avm2::vtable::VTable;
use crate::avm2::Error;
use crate::avm2::Multiname;
use crate::avm2::Namespace;
use crate::bitmap::bitmap_data::BitmapDataWrapper;
use crate::display_object::DisplayObject;
use crate::html::TextFormat;
use crate::streams::NetStream;
use crate::string::AvmString;
use gc_arena::{Collect, Gc, GcWeak, Mutation};
use ruffle_macros::enum_trait_object;
use std::cell::{Ref, RefMut};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

mod array_object;
mod bitmapdata_object;
mod bytearray_object;
mod class_object;
mod context3d_object;
mod date_object;
mod dictionary_object;
mod dispatch_object;
mod domain_object;
mod error_object;
mod event_object;
mod file_reference_object;
mod font_object;
mod function_object;
mod index_buffer_3d_object;
mod loaderinfo_object;
mod local_connection_object;
mod namespace_object;
mod net_connection_object;
mod netstream_object;
mod program_3d_object;
mod proxy_object;
mod qname_object;
mod regexp_object;
mod responder_object;
mod script_object;
mod shader_data_object;
mod shared_object_object;
mod socket_object;
mod sound_object;
mod soundchannel_object;
mod soundtransform_object;
mod stage3d_object;
mod stage_object;
mod stylesheet_object;
mod textformat_object;
mod texture_object;
mod vector_object;
mod vertex_buffer_3d_object;
mod xml_list_object;
mod xml_object;

pub use crate::avm2::object::array_object::{array_allocator, ArrayObject, ArrayObjectWeak};
pub use crate::avm2::object::bitmapdata_object::{
    bitmap_data_allocator, BitmapDataObject, BitmapDataObjectWeak,
};
pub use crate::avm2::object::bytearray_object::{
    byte_array_allocator, ByteArrayObject, ByteArrayObjectWeak,
};
pub use crate::avm2::object::class_object::{ClassObject, ClassObjectWeak};
pub use crate::avm2::object::context3d_object::{Context3DObject, Context3DObjectWeak};
pub use crate::avm2::object::date_object::{date_allocator, DateObject, DateObjectWeak};
pub use crate::avm2::object::dictionary_object::{
    dictionary_allocator, DictionaryObject, DictionaryObjectWeak,
};
pub use crate::avm2::object::dispatch_object::{DispatchObject, DispatchObjectWeak};
pub use crate::avm2::object::domain_object::{
    application_domain_allocator, DomainObject, DomainObjectWeak,
};
pub use crate::avm2::object::error_object::{error_allocator, ErrorObject, ErrorObjectWeak};
pub use crate::avm2::object::event_object::{event_allocator, EventObject, EventObjectWeak};
pub use crate::avm2::object::file_reference_object::{
    file_reference_allocator, FileReference, FileReferenceObject, FileReferenceObjectWeak,
};
pub use crate::avm2::object::font_object::{font_allocator, FontObject, FontObjectWeak};
pub use crate::avm2::object::function_object::{FunctionObject, FunctionObjectWeak};
pub use crate::avm2::object::index_buffer_3d_object::{
    IndexBuffer3DObject, IndexBuffer3DObjectWeak,
};
pub use crate::avm2::object::loaderinfo_object::{
    LoaderInfoObject, LoaderInfoObjectWeak, LoaderStream,
};
pub use crate::avm2::object::local_connection_object::{
    local_connection_allocator, LocalConnectionObject, LocalConnectionObjectWeak,
};
pub use crate::avm2::object::namespace_object::{NamespaceObject, NamespaceObjectWeak};
pub use crate::avm2::object::net_connection_object::{
    net_connection_allocator, NetConnectionObject, NetConnectionObjectWeak,
};
pub use crate::avm2::object::netstream_object::{
    netstream_allocator, NetStreamObject, NetStreamObjectWeak,
};
pub use crate::avm2::object::program_3d_object::{Program3DObject, Program3DObjectWeak};
pub use crate::avm2::object::proxy_object::{proxy_allocator, ProxyObject, ProxyObjectWeak};
pub use crate::avm2::object::qname_object::{QNameObject, QNameObjectWeak};
pub use crate::avm2::object::regexp_object::{reg_exp_allocator, RegExpObject, RegExpObjectWeak};
pub use crate::avm2::object::responder_object::{
    responder_allocator, ResponderObject, ResponderObjectWeak,
};
pub use crate::avm2::object::script_object::{
    maybe_int_property, scriptobject_allocator, ScriptObject, ScriptObjectData, ScriptObjectWeak,
    ScriptObjectWrapper,
};
pub use crate::avm2::object::shader_data_object::{
    shader_data_allocator, ShaderDataObject, ShaderDataObjectWeak,
};
pub use crate::avm2::object::shared_object_object::{
    shared_object_allocator, SharedObjectObject, SharedObjectObjectWeak,
};
pub use crate::avm2::object::socket_object::{socket_allocator, SocketObject, SocketObjectWeak};
pub use crate::avm2::object::sound_object::{
    sound_allocator, QueuedPlay, SoundLoadingState, SoundObject, SoundObjectWeak,
};
pub use crate::avm2::object::soundchannel_object::{
    sound_channel_allocator, SoundChannelObject, SoundChannelObjectWeak,
};
pub use crate::avm2::object::soundtransform_object::{
    sound_transform_allocator, SoundTransformObject, SoundTransformObjectWeak,
};
pub use crate::avm2::object::stage3d_object::{
    stage_3d_allocator, Stage3DObject, Stage3DObjectWeak,
};
pub use crate::avm2::object::stage_object::{StageObject, StageObjectWeak};
pub use crate::avm2::object::stylesheet_object::{
    style_sheet_allocator, StyleSheetObject, StyleSheetObjectWeak,
};
pub use crate::avm2::object::textformat_object::{
    textformat_allocator, TextFormatObject, TextFormatObjectWeak,
};
pub use crate::avm2::object::texture_object::{TextureObject, TextureObjectWeak};
pub use crate::avm2::object::vector_object::{vector_allocator, VectorObject, VectorObjectWeak};
pub use crate::avm2::object::vertex_buffer_3d_object::{
    VertexBuffer3DObject, VertexBuffer3DObjectWeak,
};
pub use crate::avm2::object::xml_list_object::{
    xml_list_allocator, E4XOrXml, XmlListObject, XmlListObjectWeak,
};
pub use crate::avm2::object::xml_object::{xml_allocator, XmlObject, XmlObjectWeak};
use crate::font::Font;

/// Represents an object that can be directly interacted with by the AVM2
/// runtime.
#[enum_trait_object(
    #[allow(clippy::enum_variant_names)]
    #[derive(Clone, Collect, Debug, Copy)]
    #[collect(no_drop)]
    pub enum Object<'gc> {
        ScriptObject(ScriptObject<'gc>),
        FunctionObject(FunctionObject<'gc>),
        NamespaceObject(NamespaceObject<'gc>),
        ArrayObject(ArrayObject<'gc>),
        StageObject(StageObject<'gc>),
        DomainObject(DomainObject<'gc>),
        EventObject(EventObject<'gc>),
        DispatchObject(DispatchObject<'gc>),
        XmlObject(XmlObject<'gc>),
        XmlListObject(XmlListObject<'gc>),
        RegExpObject(RegExpObject<'gc>),
        ByteArrayObject(ByteArrayObject<'gc>),
        LoaderInfoObject(LoaderInfoObject<'gc>),
        ClassObject(ClassObject<'gc>),
        VectorObject(VectorObject<'gc>),
        SoundObject(SoundObject<'gc>),
        SoundChannelObject(SoundChannelObject<'gc>),
        BitmapDataObject(BitmapDataObject<'gc>),
        DateObject(DateObject<'gc>),
        DictionaryObject(DictionaryObject<'gc>),
        QNameObject(QNameObject<'gc>),
        TextFormatObject(TextFormatObject<'gc>),
        ProxyObject(ProxyObject<'gc>),
        ErrorObject(ErrorObject<'gc>),
        Stage3DObject(Stage3DObject<'gc>),
        Context3DObject(Context3DObject<'gc>),
        IndexBuffer3DObject(IndexBuffer3DObject<'gc>),
        VertexBuffer3DObject(VertexBuffer3DObject<'gc>),
        TextureObject(TextureObject<'gc>),
        Program3DObject(Program3DObject<'gc>),
        NetStreamObject(NetStreamObject<'gc>),
        NetConnectionObject(NetConnectionObject<'gc>),
        ResponderObject(ResponderObject<'gc>),
        ShaderDataObject(ShaderDataObject<'gc>),
        SocketObject(SocketObject<'gc>),
        FileReferenceObject(FileReferenceObject<'gc>),
        FontObject(FontObject<'gc>),
        LocalConnectionObject(LocalConnectionObject<'gc>),
        SharedObjectObject(SharedObjectObject<'gc>),
        SoundTransformObject(SoundTransformObject<'gc>),
        StyleSheetObject(StyleSheetObject<'gc>),
    }
)]
pub trait TObject<'gc>: 'gc + Collect + Debug + Into<Object<'gc>> + Clone + Copy {
    /// Get the base of this object.
    /// Any trait method implementations that were not overridden will forward the call to this instead.
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>>;

    #[inline(always)]
    #[no_dynamic]
    fn base(&self) -> ScriptObjectWrapper<'gc> {
        let gc_base = self.gc_base();
        ScriptObjectWrapper(gc_base)
    }

    /// Retrieve a local property of the object. The Multiname should always be public.
    ///
    /// This skips class field lookups and looks at:
    /// - object-specific storage (like arrays)
    /// - Object dynamic properties
    /// - prototype chain.
    fn get_property_local(
        self,
        name: &Multiname<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        self.base().get_property_local(name, activation)
    }

    /// Same as get_property_local, but constructs a public Multiname for you.
    #[no_dynamic]
    fn get_string_property_local(
        self,
        name: AvmString<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        let name = Multiname::new(activation.avm2().namespaces.public_vm_internal(), name);
        self.get_property_local(&name, activation)
    }

    /// Purely an optimization for "array-like" access. This should return
    /// `None` when the lookup needs to be forwarded to the base or throw.
    fn get_index_property(self, _index: usize) -> Option<Value<'gc>> {
        None
    }

    /// Set a local property of the object. The Multiname should always be public.
    ///
    /// This skips class field lookups and looks at:
    /// - object-specific storage (like arrays)
    /// - Object dynamic properties
    fn set_property_local(
        self,
        name: &Multiname<'gc>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<(), Error<'gc>> {
        let base = self.base();
        base.set_property_local(name, value, activation)
    }

    /// Same as set_property_local, but constructs a public Multiname for you.
    /// TODO: this feels upside down, as in: we shouldn't need multinames/namespaces
    /// by the time we reach dynamic properties.
    /// But for now, this function is a smaller change to the core than a full refactor.
    #[no_dynamic]
    fn set_string_property_local(
        self,
        name: impl Into<AvmString<'gc>>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<(), Error<'gc>> {
        let name = Multiname::new(activation.avm2().namespaces.public_vm_internal(), name);
        self.set_property_local(&name, value, activation)
    }

    /// Init a local property of the object. The Multiname should always be public.
    ///
    /// This skips class field lookups and looks at:
    /// - object-specific storage (like arrays)
    /// - Object dynamic properties
    ///
    /// This should be effectively equivalent to set_property_local,
    /// as "init" is a concept specific to class const fields.
    fn init_property_local(
        self,
        name: &Multiname<'gc>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<(), Error<'gc>> {
        let base = self.base();
        base.init_property_local(name, value, activation)
    }

    /// Call a local property of the object. The Multiname should always be public.
    ///
    /// This skips class field lookups and looks at:
    /// - object-specific storage (like arrays)
    /// - Object dynamic properties
    /// - prototype chain
    fn call_property_local(
        self,
        multiname: &Multiname<'gc>,
        arguments: &[Value<'gc>],
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        // Note: normally this would just call into ScriptObjectData::call_property_local
        // but because calling into ScriptObjectData borrows it for entire duration,
        // we run a risk of a double borrow if the inner call borrows again.
        let self_val: Value<'gc> = Value::from(self.into());
        let result = self.base().get_property_local(multiname, activation)?;

        result.call(activation, self_val, arguments)
    }

    /// Retrieve a slot by its index.
    #[no_dynamic]
    #[inline(always)]
    fn get_slot(self, id: u32) -> Value<'gc> {
        let base = self.base();

        base.get_slot(id)
    }

    /// Set a slot by its index.
    #[no_dynamic]
    fn set_slot(
        self,
        id: u32,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<(), Error<'gc>> {
        let value = self.vtable().coerce_trait_value(id, value, activation)?;
        let base = self.base();

        base.set_slot(id, value, activation.gc());

        Ok(())
    }

    #[no_dynamic]
    fn set_slot_no_coerce(self, id: u32, value: Value<'gc>, mc: &Mutation<'gc>) {
        let base = self.base();

        base.set_slot(id, value, mc);
    }

    /// Implements the `in` opcode and AS3 operator.
    ///
    /// By default, this just calls `has_property`, but may be overridden by
    /// other object types to change the behavior of the `in` operator only.
    fn has_property_via_in(
        self,
        _activation: &mut Activation<'_, 'gc>,
        name: &Multiname<'gc>,
    ) -> Result<bool, Error<'gc>> {
        Ok(self.has_property(name))
    }

    /// Indicates whether or not a property exists on an object.
    #[no_dynamic]
    fn has_property(self, name: &Multiname<'gc>) -> bool {
        if self.has_own_property(name) {
            true
        } else if let Some(proto) = self.proto() {
            proto.has_property(name)
        } else {
            false
        }
    }

    /// Indicates whether or not a property or trait exists on an object and is
    /// not part of the prototype chain.
    fn has_own_property(self, name: &Multiname<'gc>) -> bool {
        self.base().has_own_property(name)
    }

    /// Same as has_own_property, but constructs a public Multiname for you.
    fn has_own_property_string(
        self,
        name: AvmString<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<bool, Error<'gc>> {
        Ok(self.has_own_property(&Multiname::new(
            activation.avm2().find_public_namespace(),
            name,
        )))
    }

    /// Returns true if an object has one or more traits of a given name.
    #[no_dynamic]
    fn has_trait(self, name: &Multiname<'gc>) -> bool {
        self.vtable().has_trait(name)
    }

    /// Delete a property by QName, after multiname resolution and all other
    /// considerations have been taken.
    ///
    /// This required method is only intended to be called by other TObject
    /// methods.
    fn delete_property_local(
        self,
        activation: &mut Activation<'_, 'gc>,
        name: &Multiname<'gc>,
    ) -> Result<bool, Error<'gc>> {
        let base = self.base();

        Ok(base.delete_property_local(activation.gc(), name))
    }

    /// Same as delete_property_local, but constructs a public Multiname for you.
    #[no_dynamic]
    fn delete_string_property_local(
        self,
        name: impl Into<AvmString<'gc>>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<bool, Error<'gc>> {
        let name = Multiname::new(activation.avm2().namespaces.public_vm_internal(), name);
        self.delete_property_local(activation, &name)
    }

    /// Retrieve the `__proto__` of a given object.
    ///
    /// The proto is another object used to resolve methods across a class of
    /// multiple objects. It should also be accessible as `__proto__` from
    /// `get`.
    #[no_dynamic]
    fn proto(&self) -> Option<Object<'gc>> {
        let base = self.base();

        base.proto()
    }

    /// Change the `__proto__` on this object.
    ///
    /// This method primarily exists so that the global scope that player
    /// globals loads into can be created before its superclasses are. It
    /// should be used sparingly, if at all.
    #[no_dynamic]
    fn set_proto(self, mc: &Mutation<'gc>, proto: Object<'gc>) {
        let base = self.base();

        base.set_proto(mc, proto)
    }

    /// Get the next enumerant index in enumerant space.
    ///
    /// Every object maintains a list of enumerants - key/value pairs indexed
    /// by arbitrary integers. This function allows clients to iterate through
    /// those indexing integers. Indexing starts with zero, and then the caller
    /// repeatedly hands prior results back into this function until it returns
    /// None.
    ///
    /// Repeated calls to this function with prior return values must
    /// eventually return `None`. Furthermore, returning `0`, while valid, is
    /// treated by AVM2 code as signalling `None`.
    fn get_next_enumerant(
        self,
        last_index: u32,
        _activation: &mut Activation<'_, 'gc>,
    ) -> Result<u32, Error<'gc>> {
        let base = self.base();

        Ok(base.get_next_enumerant(last_index))
    }

    /// Retrieve a given enumerable name by index.
    ///
    /// Enumerants are listed by index, starting from zero and iterated via
    /// `get_next_enumerant`. Only enumerants returned by that function are
    /// valid here. A value of `None` indicates that no enumerant with that
    /// index exists.
    fn get_enumerant_name(
        self,
        index: u32,
        _activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        let base = self.base();

        Ok(base.get_enumerant_name(index).unwrap_or(Value::Null))
    }

    /// Retrieve a given enumerable value by index.
    ///
    /// This default implementation of value retrieval assumes that the names
    /// of enumerants are also valid local names in the public namespace.
    fn get_enumerant_value(
        self,
        index: u32,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        let name = self
            .get_enumerant_name(index, activation)?
            .coerce_to_string(activation)?;

        // todo: this probably doesn't need non-public accesses
        Value::from(self.into()).get_public_property(name, activation)
    }

    /// Determine if a property is currently enumerable.
    ///
    /// Properties that do not exist are also not enumerable.
    fn property_is_enumerable(&self, name: AvmString<'gc>) -> bool {
        let base = self.base();

        base.property_is_enumerable(name)
    }

    /// Mark a dynamic property on this object as enumerable.
    fn set_local_property_is_enumerable(
        &self,
        mc: &Mutation<'gc>,
        name: AvmString<'gc>,
        is_enumerable: bool,
    ) {
        let base = self.base();

        base.set_local_property_is_enumerable(mc, name, is_enumerable)
    }

    /// Install a bound method on an object.
    #[no_dynamic]
    fn install_bound_method(
        &self,
        mc: &Mutation<'gc>,
        disp_id: u32,
        function: FunctionObject<'gc>,
    ) {
        let base = self.base();

        base.install_bound_method(mc, disp_id, function)
    }

    /// Construct a parameterization of this particular type and return it.
    ///
    /// This is called specifically to parameterize generic types, of which
    /// only one exists: `Vector`. When `Vector` is applied with a given
    /// parameter, a new type is returned which can be used to construct
    /// `Vector`s of that type.
    ///
    /// If the object is not a parameterized type, this yields an error. In
    /// practice, this means only `Vector` can use this method. Parameters must
    /// be class objects or `null`, which indicates any type.
    ///
    /// When a given type is parameterized with the same parameters multiple
    /// times, each application must return the same object. This is because
    /// each application has a separate prototype that accepts dynamic
    /// parameters.
    fn apply(
        &self,
        activation: &mut Activation<'_, 'gc>,
        _params: &[Value<'gc>],
    ) -> Result<ClassObject<'gc>, Error<'gc>> {
        Err(error::make_error_1127(activation))
    }

    /// Determine the type of primitive coercion this object would prefer, in
    /// the case that there is no obvious reason to prefer one type over the
    /// other.
    ///
    /// All native ECMAScript objects prefer numerical coercions, except `Date`,
    /// which wants string coercions.
    fn default_hint(&self) -> Hint {
        Hint::Number
    }

    /// Implement the result of calling `Object.prototype.toString` on this
    /// object class.
    ///
    /// `toString` is a method used to request an object be coerced to a string
    /// value. The default implementation is stored here. User-specified string
    /// coercions happen by defining `toString` in a downstream class or
    /// prototype; this is then picked up by the VM runtime when doing
    /// coercions.
    fn to_string(&self, mc: &Mutation<'gc>) -> AvmString<'gc> {
        let class_name = self.instance_class().name().local_name();

        AvmString::new_utf8(mc, format!("[object {class_name}]"))
    }

    /// Returns all public properties from this object's vtable, together with their values.
    /// This includes normal fields, const fields, and getter methods
    /// This is used for JSON serialization.
    // FIXME - the order doesn't currently match Flash Player
    #[no_dynamic]
    fn public_vtable_properties(
        &self,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Vec<(AvmString<'gc>, Value<'gc>)>, Error<'gc>> {
        let vtable = self.vtable();

        let mut values = Vec::new();
        for (name, prop) in vtable.public_properties() {
            match prop {
                Property::Slot { slot_id } | Property::ConstSlot { slot_id } => {
                    values.push((name, self.base().get_slot(slot_id)));
                }
                Property::Virtual { get: Some(get), .. } => values.push((
                    name,
                    Value::from((*self).into()).call_method(get, &[], activation)?,
                )),
                _ => {}
            }
        }

        Ok(values)
    }

    /// Determine if this object is an instance of a given type.
    ///
    /// This uses the ES4 definition of instance, which walks the class object
    /// chain and accounts for interfaces. For the ES3 definition of instance,
    /// use `is_instance_of`, which uses the prototype chain.
    ///
    /// The given object should be the class object for the given type we are
    /// checking against this object.
    #[no_dynamic]
    fn is_of_type(&self, test_class: Class<'gc>) -> bool {
        self.instance_class().has_class_in_chain(test_class)
    }

    /// Get a raw pointer value for this object.
    fn as_ptr(&self) -> *const ObjectPtr;

    /// Get this object's vtable, if it has one.
    /// Every object with class should have a vtable
    #[no_dynamic]
    fn vtable(&self) -> VTable<'gc> {
        let base = self.base();
        base.vtable()
    }

    #[no_dynamic]
    fn get_bound_method(&self, id: u32) -> Option<FunctionObject<'gc>> {
        let base = self.base();
        base.get_bound_method(id)
    }

    /// Get this object's class's `Class`, if it has one.
    #[no_dynamic]
    fn instance_class(&self) -> Class<'gc> {
        let base = self.base();
        base.instance_class()
    }

    /// Get this object's class's name, formatted for debug output.
    #[no_dynamic]
    fn instance_of_class_name(&self, mc: &Mutation<'gc>) -> AvmString<'gc> {
        self.instance_class().name().to_qualified_name(mc)
    }

    // Sets a different vtable for object, without changing instance_of.
    #[no_dynamic]
    fn set_vtable(&self, mc: &Mutation<'gc>, vtable: VTable<'gc>) {
        let base = self.base();
        base.set_vtable(mc, vtable);
    }

    /// Try to corece this object into a `ClassObject`.
    fn as_class_object(&self) -> Option<ClassObject<'gc>> {
        None
    }

    fn as_function_object(&self) -> Option<FunctionObject<'gc>> {
        None
    }

    /// Unwrap this object's `Namespace`, if the object is a boxed namespace.
    fn as_namespace(&self) -> Option<Namespace<'gc>> {
        None
    }

    fn as_namespace_object(&self) -> Option<NamespaceObject<'gc>> {
        None
    }

    /// Unwrap this object as a `QNameObject`
    fn as_qname_object(self) -> Option<QNameObject<'gc>> {
        None
    }

    fn as_loader_info_object(&self) -> Option<&LoaderInfoObject<'gc>> {
        None
    }

    fn as_array_object(&self) -> Option<ArrayObject<'gc>> {
        None
    }

    /// Unwrap this object as array storage.
    fn as_array_storage(&self) -> Option<Ref<ArrayStorage<'gc>>> {
        None
    }

    /// Unwrap this object as bytearray.
    fn as_bytearray(&self) -> Option<Ref<ByteArrayStorage>> {
        None
    }

    fn as_bytearray_mut(&self) -> Option<RefMut<ByteArrayStorage>> {
        None
    }

    fn as_bytearray_object(&self) -> Option<ByteArrayObject<'gc>> {
        None
    }

    /// Unwrap this object as mutable array storage.
    fn as_array_storage_mut(&self, _mc: &Mutation<'gc>) -> Option<RefMut<ArrayStorage<'gc>>> {
        None
    }

    /// Unwrap this object as vector storage.
    fn as_vector_storage(&self) -> Option<Ref<VectorStorage<'gc>>> {
        None
    }

    /// Unwrap this object as mutable vector storage.
    fn as_vector_storage_mut(&self, _mc: &Mutation<'gc>) -> Option<RefMut<VectorStorage<'gc>>> {
        None
    }

    /// Get this object's `DisplayObject`, if it has one.
    fn as_display_object(&self) -> Option<DisplayObject<'gc>> {
        None
    }

    fn init_application_domain(&self, _mc: &Mutation<'gc>, _domain: Domain<'gc>) {
        panic!("Tried to init an application domain on a non-ApplicationDomain object!")
    }

    /// Unwrap this object as an ApplicationDomain.
    fn as_application_domain(&self) -> Option<Domain<'gc>> {
        None
    }

    /// Unwrap this object as an event.
    fn as_event_object(self) -> Option<EventObject<'gc>> {
        None
    }

    /// Unwrap this object as an event.
    fn as_event(&self) -> Option<Ref<Event<'gc>>> {
        None
    }

    /// Unwrap this object as a mutable event.
    fn as_event_mut(&self, _mc: &Mutation<'gc>) -> Option<RefMut<Event<'gc>>> {
        None
    }

    /// Unwrap this object as a list of event handlers.
    fn as_dispatch(&self) -> Option<Ref<DispatchList<'gc>>> {
        None
    }

    /// Unwrap this object as a mutable list of event handlers.
    fn as_dispatch_mut(&self, _mc: &Mutation<'gc>) -> Option<RefMut<DispatchList<'gc>>> {
        None
    }

    /// Unwrap this object as a regexp.
    fn as_regexp_object(&self) -> Option<RegExpObject<'gc>> {
        None
    }

    /// Unwrap this object as a regexp.
    fn as_regexp(&self) -> Option<Ref<RegExp<'gc>>> {
        None
    }

    /// Unwrap this object as a font.
    fn as_font(&self) -> Option<Font<'gc>> {
        None
    }

    /// Unwrap this object as a mutable regexp.
    fn as_regexp_mut(&self, _mc: &Mutation<'gc>) -> Option<RefMut<RegExp<'gc>>> {
        None
    }

    /// Unwrap this object's sound handle.
    fn as_sound_object(self) -> Option<SoundObject<'gc>> {
        None
    }

    /// Unwrap this object's sound instance handle.
    fn as_sound_channel(self) -> Option<SoundChannelObject<'gc>> {
        None
    }

    fn as_bitmap_data(&self) -> Option<BitmapDataWrapper<'gc>> {
        None
    }

    fn as_shader_data(&self) -> Option<ShaderDataObject<'gc>> {
        None
    }

    /// Initialize the bitmap data in this object, if it's capable of
    /// supporting said data.
    ///
    /// This should only be called to initialize the association between an AVM
    /// object and it's associated bitmap data. This association should not be
    /// reinitialized later.
    fn init_bitmap_data(&self, _mc: &Mutation<'gc>, _new_bitmap: BitmapDataWrapper<'gc>) {}

    /// Get this objects `DateObject`, if it has one.
    fn as_date_object(&self) -> Option<DateObject<'gc>> {
        None
    }

    /// Get this object as a `DictionaryObject`, if it is one.
    fn as_dictionary_object(self) -> Option<DictionaryObject<'gc>> {
        None
    }

    /// Unwrap this object as a text format.
    fn as_text_format(&self) -> Option<Ref<TextFormat>> {
        None
    }

    /// Unwrap this object as a mutable text format.
    fn as_text_format_mut(&self) -> Option<RefMut<TextFormat>> {
        None
    }

    /// Unwrap this object as an Error.
    fn as_error_object(&self) -> Option<ErrorObject<'gc>> {
        None
    }

    fn as_xml_object(&self) -> Option<XmlObject<'gc>> {
        None
    }

    fn as_xml_list_object(&self) -> Option<XmlListObject<'gc>> {
        None
    }

    fn xml_descendants(
        &self,
        _activation: &mut Activation<'_, 'gc>,
        _multiname: &Multiname<'gc>,
    ) -> Option<XmlListObject<'gc>> {
        None
    }

    fn as_context_3d(&self) -> Option<Context3DObject<'gc>> {
        None
    }

    fn as_index_buffer(&self) -> Option<IndexBuffer3DObject<'gc>> {
        None
    }

    fn as_vertex_buffer(&self) -> Option<VertexBuffer3DObject<'gc>> {
        None
    }

    fn as_program_3d(&self) -> Option<Program3DObject<'gc>> {
        None
    }

    fn as_stage_3d(&self) -> Option<Stage3DObject<'gc>> {
        None
    }

    fn as_texture(&self) -> Option<TextureObject<'gc>> {
        None
    }

    fn as_netstream(self) -> Option<NetStream<'gc>> {
        None
    }

    fn as_responder(self) -> Option<ResponderObject<'gc>> {
        None
    }

    fn as_net_connection(self) -> Option<NetConnectionObject<'gc>> {
        None
    }

    fn as_socket(&self) -> Option<SocketObject<'gc>> {
        None
    }

    fn as_local_connection_object(&self) -> Option<LocalConnectionObject<'gc>> {
        None
    }

    fn as_file_reference(&self) -> Option<FileReferenceObject<'gc>> {
        None
    }

    fn as_shared_object(&self) -> Option<SharedObjectObject<'gc>> {
        None
    }

    fn as_sound_transform(&self) -> Option<SoundTransformObject<'gc>> {
        None
    }

    fn as_style_sheet(&self) -> Option<StyleSheetObject<'gc>> {
        None
    }
}

pub enum ObjectPtr {}

impl<'gc> Object<'gc> {
    pub fn ptr_eq<T: TObject<'gc>>(a: T, b: T) -> bool {
        a.as_ptr() == b.as_ptr()
    }

    #[rustfmt::skip]
    pub fn downgrade(&self) -> WeakObject<'gc> {
        match self {
            Self::ScriptObject(o) => WeakObject::ScriptObject(ScriptObjectWeak(Gc::downgrade(o.0))),
            Self::FunctionObject(o) => WeakObject::FunctionObject(FunctionObjectWeak(Gc::downgrade(o.0))),
            Self::NamespaceObject(o) => WeakObject::NamespaceObject(NamespaceObjectWeak(Gc::downgrade(o.0))),
            Self::ArrayObject(o) => WeakObject::ArrayObject(ArrayObjectWeak(Gc::downgrade(o.0))),
            Self::StageObject(o) => WeakObject::StageObject(StageObjectWeak(Gc::downgrade(o.0))),
            Self::DomainObject(o) => WeakObject::DomainObject(DomainObjectWeak(Gc::downgrade(o.0))),
            Self::EventObject(o) => WeakObject::EventObject(EventObjectWeak(Gc::downgrade(o.0))),
            Self::DispatchObject(o) => WeakObject::DispatchObject(DispatchObjectWeak(Gc::downgrade(o.0))),
            Self::XmlObject(o) => WeakObject::XmlObject(XmlObjectWeak(Gc::downgrade(o.0))),
            Self::XmlListObject(o) => WeakObject::XmlListObject(XmlListObjectWeak(Gc::downgrade(o.0))),
            Self::RegExpObject(o) => WeakObject::RegExpObject(RegExpObjectWeak(Gc::downgrade(o.0))),
            Self::ByteArrayObject(o) => WeakObject::ByteArrayObject(ByteArrayObjectWeak(Gc::downgrade(o.0))),
            Self::LoaderInfoObject(o) => WeakObject::LoaderInfoObject(LoaderInfoObjectWeak(Gc::downgrade(o.0))),
            Self::ClassObject(o) => WeakObject::ClassObject(ClassObjectWeak(Gc::downgrade(o.0))),
            Self::VectorObject(o) => WeakObject::VectorObject(VectorObjectWeak(Gc::downgrade(o.0))),
            Self::SoundObject(o) => WeakObject::SoundObject(SoundObjectWeak(Gc::downgrade(o.0))),
            Self::SoundChannelObject(o) => WeakObject::SoundChannelObject(SoundChannelObjectWeak(Gc::downgrade(o.0))),
            Self::BitmapDataObject(o) => WeakObject::BitmapDataObject(BitmapDataObjectWeak(Gc::downgrade(o.0))),
            Self::DateObject(o) => WeakObject::DateObject(DateObjectWeak(Gc::downgrade(o.0))),
            Self::DictionaryObject(o) => WeakObject::DictionaryObject(DictionaryObjectWeak(Gc::downgrade(o.0))),
            Self::QNameObject(o) => WeakObject::QNameObject(QNameObjectWeak(Gc::downgrade(o.0))),
            Self::TextFormatObject(o) => WeakObject::TextFormatObject(TextFormatObjectWeak(Gc::downgrade(o.0))),
            Self::ProxyObject(o) => WeakObject::ProxyObject(ProxyObjectWeak(Gc::downgrade(o.0))),
            Self::ErrorObject(o) => WeakObject::ErrorObject(ErrorObjectWeak(Gc::downgrade(o.0))),
            Self::Stage3DObject(o) => WeakObject::Stage3DObject(Stage3DObjectWeak(Gc::downgrade(o.0))),
            Self::Context3DObject(o) => WeakObject::Context3DObject(Context3DObjectWeak(Gc::downgrade(o.0))),
            Self::IndexBuffer3DObject(o) => WeakObject::IndexBuffer3DObject(IndexBuffer3DObjectWeak(Gc::downgrade(o.0))),
            Self::VertexBuffer3DObject(o) => WeakObject::VertexBuffer3DObject(VertexBuffer3DObjectWeak(Gc::downgrade(o.0))),
            Self::TextureObject(o) => WeakObject::TextureObject(TextureObjectWeak(Gc::downgrade(o.0))),
            Self::Program3DObject(o) => WeakObject::Program3DObject(Program3DObjectWeak(Gc::downgrade(o.0))),
            Self::NetStreamObject(o) => WeakObject::NetStreamObject(NetStreamObjectWeak(Gc::downgrade(o.0))),
            Self::NetConnectionObject(o) => WeakObject::NetConnectionObject(NetConnectionObjectWeak(Gc::downgrade(o.0))),
            Self::ResponderObject(o) => WeakObject::ResponderObject(ResponderObjectWeak(Gc::downgrade(o.0))),
            Self::ShaderDataObject(o) => WeakObject::ShaderDataObject(ShaderDataObjectWeak(Gc::downgrade(o.0))),
            Self::SocketObject(o) => WeakObject::SocketObject(SocketObjectWeak(Gc::downgrade(o.0))),
            Self::FileReferenceObject(o) => WeakObject::FileReferenceObject(FileReferenceObjectWeak(Gc::downgrade(o.0))),
            Self::FontObject(o) => WeakObject::FontObject(FontObjectWeak(Gc::downgrade(o.0))),
            Self::LocalConnectionObject(o) => WeakObject::LocalConnectionObject(LocalConnectionObjectWeak(Gc::downgrade(o.0))),
            Self::SharedObjectObject(o) => WeakObject::SharedObjectObject(SharedObjectObjectWeak(Gc::downgrade(o.0))),
            Self::SoundTransformObject(o) => WeakObject::SoundTransformObject(SoundTransformObjectWeak(Gc::downgrade(o.0))),
            Self::StyleSheetObject(o) => WeakObject::StyleSheetObject(StyleSheetObjectWeak(Gc::downgrade(o.0))),
        }
    }
}

impl PartialEq for Object<'_> {
    fn eq(&self, other: &Self) -> bool {
        Object::ptr_eq(*self, *other)
    }
}

impl Eq for Object<'_> {}

impl Hash for Object<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ptr().hash(state);
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Clone, Collect, Debug, Copy)]
#[collect(no_drop)]
pub enum WeakObject<'gc> {
    ScriptObject(ScriptObjectWeak<'gc>),
    FunctionObject(FunctionObjectWeak<'gc>),
    NamespaceObject(NamespaceObjectWeak<'gc>),
    ArrayObject(ArrayObjectWeak<'gc>),
    StageObject(StageObjectWeak<'gc>),
    DomainObject(DomainObjectWeak<'gc>),
    EventObject(EventObjectWeak<'gc>),
    DispatchObject(DispatchObjectWeak<'gc>),
    XmlObject(XmlObjectWeak<'gc>),
    XmlListObject(XmlListObjectWeak<'gc>),
    RegExpObject(RegExpObjectWeak<'gc>),
    ByteArrayObject(ByteArrayObjectWeak<'gc>),
    LoaderInfoObject(LoaderInfoObjectWeak<'gc>),
    ClassObject(ClassObjectWeak<'gc>),
    VectorObject(VectorObjectWeak<'gc>),
    SoundObject(SoundObjectWeak<'gc>),
    SoundChannelObject(SoundChannelObjectWeak<'gc>),
    BitmapDataObject(BitmapDataObjectWeak<'gc>),
    DateObject(DateObjectWeak<'gc>),
    DictionaryObject(DictionaryObjectWeak<'gc>),
    QNameObject(QNameObjectWeak<'gc>),
    TextFormatObject(TextFormatObjectWeak<'gc>),
    ProxyObject(ProxyObjectWeak<'gc>),
    ErrorObject(ErrorObjectWeak<'gc>),
    Stage3DObject(Stage3DObjectWeak<'gc>),
    Context3DObject(Context3DObjectWeak<'gc>),
    IndexBuffer3DObject(IndexBuffer3DObjectWeak<'gc>),
    VertexBuffer3DObject(VertexBuffer3DObjectWeak<'gc>),
    TextureObject(TextureObjectWeak<'gc>),
    Program3DObject(Program3DObjectWeak<'gc>),
    NetStreamObject(NetStreamObjectWeak<'gc>),
    NetConnectionObject(NetConnectionObjectWeak<'gc>),
    ResponderObject(ResponderObjectWeak<'gc>),
    ShaderDataObject(ShaderDataObjectWeak<'gc>),
    SocketObject(SocketObjectWeak<'gc>),
    FileReferenceObject(FileReferenceObjectWeak<'gc>),
    FontObject(FontObjectWeak<'gc>),
    LocalConnectionObject(LocalConnectionObjectWeak<'gc>),
    SharedObjectObject(SharedObjectObjectWeak<'gc>),
    SoundTransformObject(SoundTransformObjectWeak<'gc>),
    StyleSheetObject(StyleSheetObjectWeak<'gc>),
}

impl<'gc> WeakObject<'gc> {
    pub fn as_ptr(self) -> *const ObjectPtr {
        match self {
            Self::ScriptObject(o) => GcWeak::as_ptr(o.0) as *const ObjectPtr,
            Self::FunctionObject(o) => GcWeak::as_ptr(o.0) as *const ObjectPtr,
            Self::NamespaceObject(o) => GcWeak::as_ptr(o.0) as *const ObjectPtr,
            Self::ArrayObject(o) => GcWeak::as_ptr(o.0) as *const ObjectPtr,
            Self::StageObject(o) => GcWeak::as_ptr(o.0) as *const ObjectPtr,
            Self::DomainObject(o) => GcWeak::as_ptr(o.0) as *const ObjectPtr,
            Self::EventObject(o) => GcWeak::as_ptr(o.0) as *const ObjectPtr,
            Self::DispatchObject(o) => GcWeak::as_ptr(o.0) as *const ObjectPtr,
            Self::XmlObject(o) => GcWeak::as_ptr(o.0) as *const ObjectPtr,
            Self::XmlListObject(o) => GcWeak::as_ptr(o.0) as *const ObjectPtr,
            Self::RegExpObject(o) => GcWeak::as_ptr(o.0) as *const ObjectPtr,
            Self::ByteArrayObject(o) => GcWeak::as_ptr(o.0) as *const ObjectPtr,
            Self::LoaderInfoObject(o) => GcWeak::as_ptr(o.0) as *const ObjectPtr,
            Self::ClassObject(o) => GcWeak::as_ptr(o.0) as *const ObjectPtr,
            Self::VectorObject(o) => GcWeak::as_ptr(o.0) as *const ObjectPtr,
            Self::SoundObject(o) => GcWeak::as_ptr(o.0) as *const ObjectPtr,
            Self::SoundChannelObject(o) => GcWeak::as_ptr(o.0) as *const ObjectPtr,
            Self::BitmapDataObject(o) => GcWeak::as_ptr(o.0) as *const ObjectPtr,
            Self::DateObject(o) => GcWeak::as_ptr(o.0) as *const ObjectPtr,
            Self::DictionaryObject(o) => GcWeak::as_ptr(o.0) as *const ObjectPtr,
            Self::QNameObject(o) => GcWeak::as_ptr(o.0) as *const ObjectPtr,
            Self::TextFormatObject(o) => GcWeak::as_ptr(o.0) as *const ObjectPtr,
            Self::ProxyObject(o) => GcWeak::as_ptr(o.0) as *const ObjectPtr,
            Self::ErrorObject(o) => GcWeak::as_ptr(o.0) as *const ObjectPtr,
            Self::Stage3DObject(o) => GcWeak::as_ptr(o.0) as *const ObjectPtr,
            Self::Context3DObject(o) => GcWeak::as_ptr(o.0) as *const ObjectPtr,
            Self::IndexBuffer3DObject(o) => GcWeak::as_ptr(o.0) as *const ObjectPtr,
            Self::VertexBuffer3DObject(o) => GcWeak::as_ptr(o.0) as *const ObjectPtr,
            Self::TextureObject(o) => GcWeak::as_ptr(o.0) as *const ObjectPtr,
            Self::Program3DObject(o) => GcWeak::as_ptr(o.0) as *const ObjectPtr,
            Self::NetStreamObject(o) => GcWeak::as_ptr(o.0) as *const ObjectPtr,
            Self::NetConnectionObject(o) => GcWeak::as_ptr(o.0) as *const ObjectPtr,
            Self::ResponderObject(o) => GcWeak::as_ptr(o.0) as *const ObjectPtr,
            Self::ShaderDataObject(o) => GcWeak::as_ptr(o.0) as *const ObjectPtr,
            Self::SocketObject(o) => GcWeak::as_ptr(o.0) as *const ObjectPtr,
            Self::FileReferenceObject(o) => GcWeak::as_ptr(o.0) as *const ObjectPtr,
            Self::FontObject(o) => GcWeak::as_ptr(o.0) as *const ObjectPtr,
            Self::LocalConnectionObject(o) => GcWeak::as_ptr(o.0) as *const ObjectPtr,
            Self::SharedObjectObject(o) => GcWeak::as_ptr(o.0) as *const ObjectPtr,
            Self::SoundTransformObject(o) => GcWeak::as_ptr(o.0) as *const ObjectPtr,
            Self::StyleSheetObject(o) => GcWeak::as_ptr(o.0) as *const ObjectPtr,
        }
    }

    pub fn upgrade(self, mc: &Mutation<'gc>) -> Option<Object<'gc>> {
        Some(match self {
            Self::ScriptObject(o) => ScriptObject(o.0.upgrade(mc)?).into(),
            Self::FunctionObject(o) => FunctionObject(o.0.upgrade(mc)?).into(),
            Self::NamespaceObject(o) => NamespaceObject(o.0.upgrade(mc)?).into(),
            Self::ArrayObject(o) => ArrayObject(o.0.upgrade(mc)?).into(),
            Self::StageObject(o) => StageObject(o.0.upgrade(mc)?).into(),
            Self::DomainObject(o) => DomainObject(o.0.upgrade(mc)?).into(),
            Self::EventObject(o) => EventObject(o.0.upgrade(mc)?).into(),
            Self::DispatchObject(o) => DispatchObject(o.0.upgrade(mc)?).into(),
            Self::XmlObject(o) => XmlObject(o.0.upgrade(mc)?).into(),
            Self::XmlListObject(o) => XmlListObject(o.0.upgrade(mc)?).into(),
            Self::RegExpObject(o) => RegExpObject(o.0.upgrade(mc)?).into(),
            Self::ByteArrayObject(o) => ByteArrayObject(o.0.upgrade(mc)?).into(),
            Self::LoaderInfoObject(o) => LoaderInfoObject(o.0.upgrade(mc)?).into(),
            Self::ClassObject(o) => ClassObject(o.0.upgrade(mc)?).into(),
            Self::VectorObject(o) => VectorObject(o.0.upgrade(mc)?).into(),
            Self::SoundObject(o) => SoundObject(o.0.upgrade(mc)?).into(),
            Self::SoundChannelObject(o) => SoundChannelObject(o.0.upgrade(mc)?).into(),
            Self::BitmapDataObject(o) => BitmapDataObject(o.0.upgrade(mc)?).into(),
            Self::DateObject(o) => DateObject(o.0.upgrade(mc)?).into(),
            Self::DictionaryObject(o) => DictionaryObject(o.0.upgrade(mc)?).into(),
            Self::QNameObject(o) => QNameObject(o.0.upgrade(mc)?).into(),
            Self::TextFormatObject(o) => TextFormatObject(o.0.upgrade(mc)?).into(),
            Self::ProxyObject(o) => ProxyObject(o.0.upgrade(mc)?).into(),
            Self::ErrorObject(o) => ErrorObject(o.0.upgrade(mc)?).into(),
            Self::Stage3DObject(o) => Stage3DObject(o.0.upgrade(mc)?).into(),
            Self::Context3DObject(o) => Context3DObject(o.0.upgrade(mc)?).into(),
            Self::IndexBuffer3DObject(o) => IndexBuffer3DObject(o.0.upgrade(mc)?).into(),
            Self::VertexBuffer3DObject(o) => VertexBuffer3DObject(o.0.upgrade(mc)?).into(),
            Self::TextureObject(o) => TextureObject(o.0.upgrade(mc)?).into(),
            Self::Program3DObject(o) => Program3DObject(o.0.upgrade(mc)?).into(),
            Self::NetStreamObject(o) => NetStreamObject(o.0.upgrade(mc)?).into(),
            Self::NetConnectionObject(o) => NetConnectionObject(o.0.upgrade(mc)?).into(),
            Self::ResponderObject(o) => ResponderObject(o.0.upgrade(mc)?).into(),
            Self::ShaderDataObject(o) => ShaderDataObject(o.0.upgrade(mc)?).into(),
            Self::SocketObject(o) => SocketObject(o.0.upgrade(mc)?).into(),
            Self::FileReferenceObject(o) => FileReferenceObject(o.0.upgrade(mc)?).into(),
            Self::FontObject(o) => FontObject(o.0.upgrade(mc)?).into(),
            Self::LocalConnectionObject(o) => LocalConnectionObject(o.0.upgrade(mc)?).into(),
            Self::SharedObjectObject(o) => SharedObjectObject(o.0.upgrade(mc)?).into(),
            Self::SoundTransformObject(o) => SoundTransformObject(o.0.upgrade(mc)?).into(),
            Self::StyleSheetObject(o) => StyleSheetObject(o.0.upgrade(mc)?).into(),
        })
    }
}

/// Implements a custom allocator for classes that are not constructible.
/// (but their derived classes can be)
pub fn abstract_class_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let class_name = class.instance_class().name().local_name();

    return Err(Error::AvmError(error::argument_error(
        activation,
        &format!("Error #2012: {class_name} class cannot be instantiated."),
        2012,
    )?));
}
