//! flash.net.FileReferenceList object

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::globals::array::ArrayBuilder;
use crate::avm1::globals::file_reference::parse_file_filters;
use crate::avm1::property_decl::{DeclContext, PropertyOrder, StaticDeclarations, SystemClass};
use crate::avm1::{NativeObject, Object, Value};
use crate::backend::ui::FileDialogSelection;

use ruffle_macros::istr;

const PROTO_DECLS: StaticDeclarations = declare_static_properties! {
    "browse" => method(browse; DONT_ENUM);
};

pub fn create_class<'gc>(
    context: &mut DeclContext<'_, 'gc>,
    super_proto: Object<'gc>,
) -> SystemClass<'gc> {
    let class = context.class(constructor, super_proto, PropertyOrder::PrototypeLast);
    context.define_properties_on(class.proto, PROTO_DECLS(context));
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
    let class_path = [istr!("flash"), istr!("net"), istr!("FileReference")];

    let array_builder = ArrayBuilder::new(activation);

    array_builder.with(selections.iter().filter_map(|selection| {
        // If instantiating the FileReference fails, exceptions are swallowed
        // and nothing is added to the array.
        if let Ok(Value::Object(obj)) = activation.instantiate_class_as_script(class_path, &[])
            && let NativeObject::FileReference(fr) = obj.native()
        {
            fr.init_from_file_selection(activation, selection.as_ref());
            return Some(obj.into());
        }
        None
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
