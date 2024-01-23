use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::FunctionObject;
use crate::avm1::globals::as_broadcaster::BroadcasterFunctions;
use crate::avm1::globals::file_reference::FileReferenceObject;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{ArrayObject, Executable, NativeObject, Object, ScriptObject, TObject, Value};
use crate::backend::ui::FileFilter;
use crate::context::GcContext;
use gc_arena::GcCell;

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "browse" => method(browse; DONT_ENUM);
};

const OBJECT_DECLS: &[Declaration] = declare_properties! {};

pub fn browse<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let file_filters = match args.get(0) {
        Some(Value::Object(array)) => {
            // Array of filter objects.
            let length = array.length(activation)?;

            // Empty array is not allowed
            if length == 0 {
                return Ok(false.into());
            }

            let mut results = Vec::with_capacity(length as usize);

            for i in 0..length {
                if let Value::Object(element) = array.get_element(activation, i) {
                    let mac_type = if let Ok(val) = element.get("macType", activation) {
                        Some(val.coerce_to_string(activation)?.to_string())
                    } else {
                        None
                    };

                    let description = element
                        .get("description", activation)?
                        .coerce_to_string(activation)?
                        .to_string();

                    let extensions = element
                        .get("extension", activation)?
                        .coerce_to_string(activation)?
                        .to_string();

                    // Empty strings are not allowed for desc / extension
                    if description.is_empty() || extensions.is_empty() {
                        return Ok(false.into());
                    }

                    results.push(FileFilter {
                        description,
                        extensions,
                        mac_type,
                    });
                } else {
                    return Err(Error::ThrownValue("Unexpected filter value".into()));
                }
            }

            results
        }
        None => Vec::new(),
        _ => return Ok(Value::Undefined),
    };

    let dialog = activation
        .context
        .ui
        .display_file_open_dialog(file_filters, true);

    let result = match dialog {
        Some(dialog) => {
            let process = activation.context.load_manager.select_multiple_file_dialog(
                activation.context.player.clone(),
                this,
                dialog,
            );

            activation.context.navigator.spawn_future(process);
            true
        }
        None => false,
    };

    Ok(result.into())
}

pub(crate) fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    this.set_native(
        activation.context.gc_context,
        NativeObject::FileReference(FileReferenceObject(GcCell::new(
            activation.context.gc_context,
            Default::default(),
        ))),
    );
    let filelist = ArrayObject::empty(activation);
    this.set("fileList", filelist.into(), activation)?;
    Ok(this.into())
}

pub fn create_constructor<'gc>(
    context: &mut GcContext<'_, 'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
    array_proto: Object<'gc>,
    broadcaster_functions: BroadcasterFunctions<'gc>,
) -> Object<'gc> {
    let file_reference_list_proto = ScriptObject::new(context.gc_context, Some(proto));
    define_properties_on(PROTO_DECLS, context, file_reference_list_proto, fn_proto);
    broadcaster_functions.initialize(
        context.gc_context,
        file_reference_list_proto.into(),
        array_proto,
    );
    let constructor = FunctionObject::constructor(
        context.gc_context,
        Executable::Native(constructor),
        constructor_to_fn!(constructor),
        fn_proto,
        file_reference_list_proto.into(),
    );
    define_properties_on(
        OBJECT_DECLS,
        context,
        constructor.raw_script_object(),
        fn_proto,
    );
    constructor
}
