//! AVM2 objects.

use crate::avm2::activation::Activation;
use crate::avm2::array::ArrayStorage;
use crate::avm2::bytearray::ByteArrayStorage;
use crate::avm2::class::Class;
use crate::avm2::domain::Domain;
use crate::avm2::error::{self, make_error_2012};
use crate::avm2::events::{DispatchList, Event};
use crate::avm2::function::FunctionArgs;
use crate::avm2::property::Property;
use crate::avm2::regexp::RegExp;
use crate::avm2::value::{Hint, Value};
use crate::avm2::vector::VectorStorage;
use crate::avm2::vtable::VTable;
use crate::avm2::Error;
use crate::avm2::Multiname;
use crate::avm2::Namespace;
use crate::bitmap::bitmap_data::BitmapData;
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
mod message_channel_object;
mod namespace_object;
mod net_connection_object;
mod netstream_object;
mod program_3d_object;
mod proxy_object;
mod qname_object;
mod regexp_object;
mod responder_object;
mod script_object;
mod security_domain_object;
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
mod worker_domain_object;
mod worker_object;
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
    file_reference_allocator, FileReference, FileReferenceObject, FileReferenceObjectHandle,
    FileReferenceObjectWeak,
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
pub use crate::avm2::object::message_channel_object::{
    MessageChannelObject, MessageChannelObjectWeak,
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
    get_dynamic_property, scriptobject_allocator, ScriptObject, ScriptObjectData,
    ScriptObjectHandle, ScriptObjectWeak, ScriptObjectWrapper,
};
pub use crate::avm2::object::security_domain_object::{
    SecurityDomainObject, SecurityDomainObjectWeak,
};
pub use crate::avm2::object::shader_data_object::{
    shader_data_allocator, ShaderDataObject, ShaderDataObjectWeak,
};
pub use crate::avm2::object::shared_object_object::{SharedObjectObject, SharedObjectObjectWeak};
pub use crate::avm2::object::socket_object::{socket_allocator, SocketObject, SocketObjectWeak};
pub use crate::avm2::object::sound_object::{
    sound_allocator, QueuedPlay, SoundLoadingState, SoundObject, SoundObjectHandle, SoundObjectWeak,
};
pub use crate::avm2::object::soundchannel_object::{
    sound_channel_allocator, SoundChannelObject, SoundChannelObjectWeak,
};
pub use crate::avm2::object::soundtransform_object::{
    sound_transform_allocator, SoundTransformObject, SoundTransformObjectWeak,
};
pub use crate::avm2::object::stage3d_object::{Stage3DObject, Stage3DObjectWeak};
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
pub use crate::avm2::object::worker_domain_object::{WorkerDomainObject, WorkerDomainObjectWeak};
pub use crate::avm2::object::worker_object::{WorkerObject, WorkerObjectWeak};
pub use crate::avm2::object::xml_list_object::{
    xml_list_allocator, E4XOrXml, XmlListObject, XmlListObjectWeak,
};
pub use crate::avm2::object::xml_object::{xml_allocator, XmlObject, XmlObjectWeak};
use crate::font::Font;

/// Represents an object that can be directly interacted with by the AVM2
/// runtime.
#[enum_trait_object(
    #[expect(clippy::enum_variant_names)]
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
        WorkerObject(WorkerObject<'gc>),
        WorkerDomainObject(WorkerDomainObject<'gc>),
        MessageChannelObject(MessageChannelObject<'gc>),
        SecurityDomainObject(SecurityDomainObject<'gc>),
    }
)]
pub trait TObject<'gc>: 'gc + Collect<'gc> + Debug + Into<Object<'gc>> + Clone + Copy {
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

    /// Get a dynamic property on this Object by name. This is like
    /// `get_property_local`, but it skips dynamic-dispatch TObject logic
    /// and always gets a dynamic property on the base ScriptObject. If the
    /// object is sealed, or the dynamic property does not exist on the
    /// ScriptObject, this returns `None`.
    #[no_dynamic]
    fn get_dynamic_property(self, local_name: AvmString<'gc>) -> Option<Value<'gc>> {
        use crate::avm2::object::script_object::maybe_int_property;

        // See the comment in script_object::get_dynamic_property
        let key = maybe_int_property(local_name);

        let base = self.base();
        let values = base.values();
        let value = values.as_hashmap().get(&key);
        value.map(|v| v.value)
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

    /// Set a dynamic property on this Object by name. This is like
    /// `set_property_local`, but it skips dynamic-dispatch TObject logic
    /// and always sets a dynamic property on the base ScriptObject.
    ///
    /// Note that calling this method on a non-dynamic (sealed) object will
    /// panic, as sealed objects cannot have dynamic properties set on them.
    ///
    /// Additionally, if the vtable of the object has the property `local_name`
    /// in it, this method will still declare a dynamic property on the object
    /// with the same name, so take care to only call this method on objects
    /// that are known to not have the property `local_name` in their vtable.
    #[no_dynamic]
    fn set_dynamic_property(
        self,
        local_name: AvmString<'gc>,
        value: Value<'gc>,
        mc: &Mutation<'gc>,
    ) {
        use crate::avm2::object::script_object::maybe_int_property;

        let base = self.base();
        assert!(!base.is_sealed());

        // See the comment in ScriptObjectWrapper::set_property_local
        let key = maybe_int_property(local_name);

        base.values_mut(mc).insert(key, value);
    }

    /// Purely an optimization for "array-like" access. This should return
    /// `None` when the lookup needs to be forwarded to the base.
    fn set_index_property(
        self,
        _activation: &mut Activation<'_, 'gc>,
        _index: usize,
        _value: Value<'gc>,
    ) -> Option<Result<(), Error<'gc>>> {
        None
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
        arguments: FunctionArgs<'_, 'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        // Note: normally this would just call into ScriptObjectData::call_property_local
        // but because calling into ScriptObjectData borrows it for entire duration,
        // we run a risk of a double borrow if the inner call borrows again.
        let self_val: Value<'gc> = Value::from(self.into());
        let result = self.base().get_property_local(multiname, activation)?;

        result.call(activation, self_val, arguments)
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

    /// Delete a dynamic property on this Object by name. This is like
    /// `delete_property_local`, but it skips dynamic-dispatch TObject logic
    /// and always tries to delete a dynamic property on the base ScriptObject.
    ///
    /// This method will panic when called on a non-dynamic (sealed) object, as
    /// sealed objects don't have dynamic properties to delete anyway.
    #[no_dynamic]
    fn delete_dynamic_property(self, name: AvmString<'gc>, mc: &Mutation<'gc>) {
        use crate::avm2::object::script_object::maybe_int_property;

        let base = self.base();
        assert!(!base.is_sealed());

        let key = maybe_int_property(name);
        base.values_mut(mc).remove(&key);
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
                Property::Virtual { get: Some(get), .. } => {
                    values.push((name, Value::from(*self).call_method(get, &[], activation)?))
                }
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

    #[inline(always)]
    #[no_dynamic]
    /// Get a raw pointer value for this object.
    fn as_ptr(&self) -> *const ObjectPtr {
        Gc::as_ptr(self.gc_base()).cast()
    }

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

    fn xml_descendants(
        &self,
        _activation: &mut Activation<'_, 'gc>,
        _multiname: &Multiname<'gc>,
    ) -> Option<XmlListObject<'gc>> {
        None
    }
}

pub enum ObjectPtr {}

macro_rules! impl_downcast_methods {
    ($(
        $vis:vis fn $fn_name:ident for $variant:ident;
    )*) => { $(
        #[doc = concat!("Downcast this object as a `", stringify!($variant), "`.")]
        #[inline(always)]
        $vis fn $fn_name(self) -> Option<$variant<'gc>> {
            if let Self::$variant(obj) = self {
                Some(obj)
            } else {
                None
            }
        }
    )* }
}

impl<'gc> Object<'gc> {
    pub fn ptr_eq<T: TObject<'gc>>(a: T, b: T) -> bool {
        std::ptr::eq(a.as_ptr(), b.as_ptr())
    }

    impl_downcast_methods! {
        pub fn as_script_object for ScriptObject;
        pub fn as_class_object for ClassObject;
        pub fn as_function_object for FunctionObject;
        pub fn as_namespace_object for NamespaceObject;
        pub fn as_qname_object for QNameObject;
        pub fn as_loader_info_object for LoaderInfoObject;
        pub fn as_array_object for ArrayObject;
        pub fn as_bytearray_object for ByteArrayObject;
        pub fn as_vector_object for VectorObject;
        pub fn as_stage_object for StageObject;
        pub fn as_domain_object for DomainObject;
        pub fn as_event_object for EventObject;
        pub fn as_dispatch_object for DispatchObject;
        pub fn as_font_object for FontObject;
        pub fn as_regexp_object for RegExpObject;
        pub fn as_sound_object for SoundObject;
        pub fn as_sound_channel for SoundChannelObject;
        pub fn as_bitmap_data_object for BitmapDataObject;
        pub fn as_shader_data for ShaderDataObject;
        pub fn as_date_object for DateObject;
        pub fn as_dictionary_object for DictionaryObject;
        pub fn as_text_format_object for TextFormatObject;
        pub fn as_error_object for ErrorObject;
        pub fn as_xml_object for XmlObject;
        pub fn as_xml_list_object for XmlListObject;
        pub fn as_context_3d for Context3DObject;
        pub fn as_index_buffer for IndexBuffer3DObject;
        pub fn as_vertex_buffer for VertexBuffer3DObject;
        pub fn as_program_3d for Program3DObject;
        pub fn as_stage_3d for Stage3DObject;
        pub fn as_texture for TextureObject;
        pub fn as_netstream_object for NetStreamObject;
        pub fn as_responder for ResponderObject;
        pub fn as_net_connection for NetConnectionObject;
        pub fn as_socket for SocketObject;
        pub fn as_local_connection_object for LocalConnectionObject;
        pub fn as_file_reference for FileReferenceObject;
        pub fn as_shared_object for SharedObjectObject;
        pub fn as_sound_transform for SoundTransformObject;
        pub fn as_style_sheet for StyleSheetObject;
    }

    /// Unwrap this object's `Namespace`, if the object is a boxed namespace.
    pub fn as_namespace(self) -> Option<Namespace<'gc>> {
        self.as_namespace_object().map(|o| o.namespace())
    }

    /// Unwrap this object as array storage.
    pub fn as_array_storage(&self) -> Option<Ref<'_, ArrayStorage<'gc>>> {
        self.as_array_object().map(|o| o.storage())
    }

    /// Unwrap this object as mutable array storage.
    pub fn as_array_storage_mut(
        &self,
        mc: &Mutation<'gc>,
    ) -> Option<RefMut<'_, ArrayStorage<'gc>>> {
        self.as_array_object().map(|o| o.storage_mut(mc))
    }

    /// Unwrap this object as byte array storage.
    pub fn as_bytearray(&self) -> Option<Ref<'_, ByteArrayStorage>> {
        self.as_bytearray_object().map(|o| o.storage())
    }

    /// Unwrap this object as mutable byte array storage.
    pub fn as_bytearray_mut(&self) -> Option<RefMut<'_, ByteArrayStorage>> {
        self.as_bytearray_object().map(|o| o.storage_mut())
    }

    /// Unwrap this object as vector storage.
    pub fn as_vector_storage(&self) -> Option<Ref<'_, VectorStorage<'gc>>> {
        self.as_vector_object().map(|o| o.storage())
    }

    /// Unwrap this object as mutable vector storage.
    pub fn as_vector_storage_mut(
        &self,
        mc: &Mutation<'gc>,
    ) -> Option<RefMut<'_, VectorStorage<'gc>>> {
        self.as_vector_object().map(|o| o.storage_mut(mc))
    }

    /// Get this object's `DisplayObject`, if it has one.
    pub fn as_display_object(self) -> Option<DisplayObject<'gc>> {
        self.as_stage_object().map(|o| o.display_object())
    }

    /// Unwrap this object as an application domain.
    pub fn as_application_domain(self) -> Option<Domain<'gc>> {
        self.as_domain_object().map(|o| o.domain())
    }

    /// Unwrap this object as an event.
    pub fn as_event(&self) -> Option<Ref<'_, Event<'gc>>> {
        self.as_event_object().map(|o| o.event())
    }

    /// Unwrap this object as a mutable event.
    pub fn as_event_mut(&self, mc: &Mutation<'gc>) -> Option<RefMut<'_, Event<'gc>>> {
        self.as_event_object().map(|o| o.event_mut(mc))
    }

    /// Unwrap this object as a mutable list of event handlers.
    pub fn as_dispatch_mut(&self, mc: &Mutation<'gc>) -> Option<RefMut<'_, DispatchList<'gc>>> {
        self.as_dispatch_object().map(|o| o.dispatch_mut(mc))
    }

    /// Unwrap this object as a font.
    pub fn as_font(&self) -> Option<Font<'gc>> {
        self.as_font_object().and_then(|o| o.font())
    }

    /// Unwrap this object as a regexp.
    pub fn as_regexp(&self) -> Option<Ref<'_, RegExp<'gc>>> {
        self.as_regexp_object().map(|o| o.regexp())
    }

    /// Unwrap this object as a mutable regexp.
    pub fn as_regexp_mut(&self, mc: &Mutation<'gc>) -> Option<RefMut<'_, RegExp<'gc>>> {
        self.as_regexp_object().map(|o| o.regexp_mut(mc))
    }

    /// Unwrap this object as a bitmap data.
    pub fn as_bitmap_data(&self) -> Option<BitmapData<'gc>> {
        self.as_bitmap_data_object().map(|o| o.get_bitmap_data())
    }

    /// Unwrap this object as a text format.
    pub fn as_text_format(&self) -> Option<Ref<'_, TextFormat>> {
        self.as_text_format_object().map(|o| o.text_format())
    }

    /// Unwrap this object as a mutable text format.
    pub fn as_text_format_mut(&self) -> Option<RefMut<'_, TextFormat>> {
        self.as_text_format_object().map(|o| o.text_format_mut())
    }

    pub fn as_netstream(self) -> Option<NetStream<'gc>> {
        self.as_netstream_object().map(|o| o.netstream())
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

macro_rules! define_weak_enum {
    (
        $(#[$attrs:meta])*
        $vis:ident enum $weak_enum:ident<'gc> for $strong_enum:ident<'gc> {
            $( $variant:ident($weak_ty:ident<'gc>) ),* $(,)?
        }
    ) => {
        $(#[$attrs])*
        $vis enum $weak_enum<'gc> {
            $( $variant($weak_ty<'gc>), )*
        }

        impl<'gc> $weak_enum<'gc> {
            $vis fn as_ptr(self) -> *const ObjectPtr {
                match self {
                    $( Self::$variant(o) => GcWeak::as_ptr(o.0).cast(), )*
                }
            }

            $vis fn upgrade(self, mc: &Mutation<'gc>) -> Option<$strong_enum<'gc>> {
                match self {
                    $( Self::$variant(o) => $strong_enum::$variant($variant(o.0.upgrade(mc)?)).into(), )*
                }
            }
        }

        impl<'gc> $strong_enum<'gc> {
            $vis fn downgrade(self) -> $weak_enum<'gc> {
                match self {
                    $( Self::$variant(o) => $weak_enum::$variant($weak_ty(Gc::downgrade(o.0))), )*
                }
            }
        }
    }
}

define_weak_enum! {
    #[expect(clippy::enum_variant_names)]
    #[derive(Clone, Collect, Debug, Copy)]
    #[collect(no_drop)]
    pub enum WeakObject<'gc> for Object<'gc> {
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
        WorkerObject(WorkerObjectWeak<'gc>),
        WorkerDomainObject(WorkerDomainObjectWeak<'gc>),
        MessageChannelObject(MessageChannelObjectWeak<'gc>),
        SecurityDomainObject(SecurityDomainObjectWeak<'gc>),
    }
}

/// Implements a custom allocator for classes that are not constructible.
/// (but their derived classes can be)
pub fn abstract_class_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let class_name = class.instance_class().name().local_name();
    Err(make_error_2012(activation, class_name))
}

/// Implements a custom call handler for classes that are constructed when
/// called.
pub fn construct_call_handler<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    this.as_object()
        .unwrap()
        .as_class_object()
        .unwrap()
        .construct(activation, args)
}
