//! flash.net.FileReferenceList object

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::globals::array::ArrayBuilder;
use crate::avm1::globals::as_broadcaster::BroadcasterFunctions;
use crate::avm1::globals::file_reference::{FileReferenceObject, parse_file_filters};
use crate::avm1::property_decl::{DeclContext, PropertyOrder, StaticDeclarations, SystemClass};
use crate::avm1::{NativeObject, Object, Value};
use crate::backend::ui::FileDialogSelection;

const PROTO_DECLS: StaticDeclarations = declare_static_properties! {
    "browse" => method(browse; DONT_ENUM);
};

pub fn create_class<'gc>(
    context: &mut DeclContext<'_, 'gc>,
    super_proto: Object<'gc>,
    broadcaster_fns: BroadcasterFunctions<'gc>,
    array_proto: Object<'gc>,
) -> SystemClass<'gc> {
    let class = context.class(constructor, super_proto, PropertyOrder::PrototypeLast);
    context.define_properties_on(class.proto, PROTO_DECLS(context));
    broadcaster_fns.initialize(context.strings, class.proto, array_proto);
    class
}

fn constructor<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Value::Undefined)
}

/// Build an AVM1 array of `FileReference` objects from a list of file dialog selections.
///
/// Used by `loader::select_multi_file_dialog_avm1` to populate `FileReferenceList.fileList`.
pub fn build_file_list<'gc>(
    activation: &mut Activation<'_, 'gc>,
    selections: &[Box<dyn FileDialogSelection>],
) -> Object<'gc> {
    let file_reference_proto = activation.prototypes().file_reference;

    let array_builder = ArrayBuilder::new(activation);

    array_builder.with(selections.iter().map(|selection| {
        let fr = FileReferenceObject::new(activation.gc());
        fr.init_from_file_selection(activation, selection.as_ref());

        Object::new_with_native(
            &activation.context.strings,
            Some(file_reference_proto),
            NativeObject::FileReference(fr),
        )
        .into()
    }))
}

fn browse<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let file_filters = match parse_file_filters(activation, args.first().copied())? {
        Some(filters) => filters,
        None => return Ok(false.into()),
    };

    let dialog = activation
        .context
        .ui
        .display_file_open_dialog_multiple(file_filters);

    let result = match dialog {
        Some(dialog) => {
            let process =
                crate::loader::select_multi_file_dialog_avm1(activation.context, this, dialog);

            activation.context.navigator.spawn_future(process);
            true
        }
        None => false,
    };

    Ok(result.into())
}
