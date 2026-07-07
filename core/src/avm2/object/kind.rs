#![allow(dead_code)]

use gc_arena::Collect;

use super::{
    ScriptObjectData, array_object::ArrayObjectData, bitmapdata_object::BitmapDataObjectData,
    bytearray_object::ByteArrayObjectData, class_object::ClassObjectData,
    content_element_object::ContentElementObjectData, context3d_object::Context3DObjectData,
    date_object::DateObjectData, dictionary_object::DictionaryObjectData,
    dispatch_object::DispatchObjectData, domain_object::DomainObjectData,
    element_format_object::ElementFormatObjectData, error_object::ErrorObjectData,
    event_object::EventObjectData, file_reference_object::FileReferenceObjectData,
    font_description_object::FontDescriptionObjectData, font_object::FontObjectData,
    function_object::FunctionObjectData, index_buffer_3d_object::IndexBuffer3DObjectData,
    loaderinfo_object::LoaderInfoObjectData, local_connection_object::LocalConnectionObjectData,
    message_channel_object::MessageChannelObjectData, namespace_object::NamespaceObjectData,
    net_connection_object::NetConnectionObjectData, netstream_object::NetStreamObjectData,
    program_3d_object::Program3DObjectData, proxy_object::ProxyObjectData,
    qname_object::QNameObjectData, regexp_object::RegExpObjectData,
    responder_object::ResponderObjectData, security_domain_object::SecurityDomainObjectData,
    shader_data_object::ShaderDataObjectData, shared_object_object::SharedObjectObjectData,
    socket_object::SocketObjectData, sound_object::SoundObjectData,
    soundchannel_object::SoundChannelObjectData, soundtransform_object::SoundTransformObjectData,
    stage_object::StageObjectData, stage3d_object::Stage3DObjectData,
    stylesheet_object::StyleSheetObjectData, text_block_object::TextBlockObjectData,
    textformat_object::TextFormatObjectData, texture_object::TextureObjectData,
    vector_object::VectorObjectData, vertex_buffer_3d_object::VertexBuffer3DObjectData,
    worker_domain_object::WorkerDomainObjectData, worker_object::WorkerObjectData,
    xml_list_object::XmlListObjectData, xml_object::XmlObjectData,
};

mod sealed {
    pub trait Sealed {}
}

pub trait Kind: sealed::Sealed + 'static {
    const ID: ObjectKind;
}

macro_rules! define_marker {
    ($name:ident) => {
        #[derive(Clone, Collect)]
        #[collect(require_static)]
        pub enum $name {}

        impl sealed::Sealed for $name {}
    };
}

macro_rules! define_variants {
    ($($name:ident => $data:ty),* $(,)?) => {
        #[expect(clippy::enum_variant_names)]
        #[derive(Clone, Copy, Debug, Eq, PartialEq, Collect)]
        #[collect(require_static)]
        pub enum ObjectKind {
            $($name,)*
        }

        $(
            define_marker!($name);

            impl Kind for $name {
                const ID: ObjectKind = ObjectKind::$name;
            }

            unsafe impl<'gc> super::ObjectVariant<'gc> for super::$name<'gc> {
                type Kind = $name;
                type Data = $data;

                fn data_gc(self) -> ::gc_arena::Gc<'gc, Self::Data> {
                    self.0
                }

                unsafe fn from_data_gc_unchecked(
                    gc: ::gc_arena::Gc<'gc, Self::Data>,
                ) -> Self {
                    super::$name(gc)
                }
            }
        )*
    };
}

define_marker!(Erased);

define_variants!(
    ScriptObject => ScriptObjectData<'gc, ScriptObject>,
    FunctionObject => FunctionObjectData<'gc>,
    NamespaceObject => NamespaceObjectData<'gc>,
    ArrayObject => ArrayObjectData<'gc>,
    StageObject => StageObjectData<'gc>,
    DomainObject => DomainObjectData<'gc>,
    EventObject => EventObjectData<'gc>,
    DispatchObject => DispatchObjectData<'gc>,
    XmlObject => XmlObjectData<'gc>,
    XmlListObject => XmlListObjectData<'gc>,
    RegExpObject => RegExpObjectData<'gc>,
    ByteArrayObject => ByteArrayObjectData<'gc>,
    LoaderInfoObject => LoaderInfoObjectData<'gc>,
    ClassObject => ClassObjectData<'gc>,
    VectorObject => VectorObjectData<'gc>,
    SoundObject => SoundObjectData<'gc>,
    SoundChannelObject => SoundChannelObjectData<'gc>,
    BitmapDataObject => BitmapDataObjectData<'gc>,
    DateObject => DateObjectData<'gc>,
    DictionaryObject => DictionaryObjectData<'gc>,
    QNameObject => QNameObjectData<'gc>,
    TextFormatObject => TextFormatObjectData<'gc>,
    ProxyObject => ProxyObjectData<'gc>,
    ErrorObject => ErrorObjectData<'gc>,
    Stage3DObject => Stage3DObjectData<'gc>,
    Context3DObject => Context3DObjectData<'gc>,
    IndexBuffer3DObject => IndexBuffer3DObjectData<'gc>,
    VertexBuffer3DObject => VertexBuffer3DObjectData<'gc>,
    TextureObject => TextureObjectData<'gc>,
    Program3DObject => Program3DObjectData<'gc>,
    NetStreamObject => NetStreamObjectData<'gc>,
    NetConnectionObject => NetConnectionObjectData<'gc>,
    ResponderObject => ResponderObjectData<'gc>,
    ShaderDataObject => ShaderDataObjectData<'gc>,
    SocketObject => SocketObjectData<'gc>,
    FileReferenceObject => FileReferenceObjectData<'gc>,
    FontObject => FontObjectData<'gc>,
    LocalConnectionObject => LocalConnectionObjectData<'gc>,
    SharedObjectObject => SharedObjectObjectData<'gc>,
    SoundTransformObject => SoundTransformObjectData<'gc>,
    StyleSheetObject => StyleSheetObjectData<'gc>,
    WorkerObject => WorkerObjectData<'gc>,
    WorkerDomainObject => WorkerDomainObjectData<'gc>,
    MessageChannelObject => MessageChannelObjectData<'gc>,
    SecurityDomainObject => SecurityDomainObjectData<'gc>,
    ContentElementObject => ContentElementObjectData<'gc>,
    ElementFormatObject => ElementFormatObjectData<'gc>,
    FontDescriptionObject => FontDescriptionObjectData<'gc>,
    TextBlockObject => TextBlockObjectData<'gc>,
);
