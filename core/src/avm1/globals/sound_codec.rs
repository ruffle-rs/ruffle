use crate::avm1::Object;
use crate::avm1::property_decl::{DeclContext, StaticDeclarations};

const OBJECT_DECLS: StaticDeclarations = declare_static_properties! {
    "SPEEX" => value("Speex");
    "NELLYMOSER" => value("Nellymoser");
};

pub fn create<'gc>(context: &mut DeclContext<'_, 'gc>) -> Object<'gc> {
    let sound_codec = Object::new(context.strings, Some(context.object_proto));
    context.define_properties_on(sound_codec, OBJECT_DECLS(context));
    sound_codec
}
