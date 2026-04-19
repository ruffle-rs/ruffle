#![allow(dead_code)]

use gc_arena::Collect;

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
    ($($name:ident),* $(,)?) => {
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
        )*
    };
}

define_marker!(Erased);

define_variants!(
    ScriptObject,
    FunctionObject,
    NamespaceObject,
    ArrayObject,
    StageObject,
    DomainObject,
    EventObject,
    DispatchObject,
    XmlObject,
    XmlListObject,
    RegExpObject,
    ByteArrayObject,
    LoaderInfoObject,
    ClassObject,
    VectorObject,
    SoundObject,
    SoundChannelObject,
    BitmapDataObject,
    DateObject,
    DictionaryObject,
    QNameObject,
    TextFormatObject,
    ProxyObject,
    ErrorObject,
    Stage3DObject,
    Context3DObject,
    IndexBuffer3DObject,
    VertexBuffer3DObject,
    TextureObject,
    Program3DObject,
    NetStreamObject,
    NetConnectionObject,
    ResponderObject,
    ShaderDataObject,
    SocketObject,
    FileReferenceObject,
    FontObject,
    LocalConnectionObject,
    SharedObjectObject,
    SoundTransformObject,
    StyleSheetObject,
    WorkerObject,
    WorkerDomainObject,
    MessageChannelObject,
    SecurityDomainObject,
    ContentElementObject,
    ElementFormatObject,
    FontDescriptionObject,
    TextBlockObject,
);
