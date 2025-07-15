use ruffle_wstr::WString;

use crate::avm2::object::TObject;
use crate::avm2::{Activation, AvmString, Class, Multiname, Value};
use std::fmt::{Debug, Display};
use std::mem::size_of;

use super::function::display_function;
use super::method::Method;
use super::ClassObject;

/// An error generated while handling AVM2 logic
#[repr(transparent)]
pub struct Error<'gc>(Box<ErrorData<'gc>>);

enum ErrorData<'gc> {
    /// A thrown error. This can be produced by an explicit 'throw'
    /// opcode, or by a native implementation that throws an exception.
    /// This can be caught by any catch blocks created by ActionScript code
    AvmError(Value<'gc>),
    /// An internal VM error. This cannot be caught by ActionScript code -
    /// it will either be logged by Ruffle, or cause the player to
    /// stop executing.
    RustError(Box<dyn std::error::Error>),
}

impl<'gc> Error<'gc> {
    pub fn avm_error(error: Value<'gc>) -> Self {
        Error(Box::new(ErrorData::AvmError(error)))
    }

    pub fn rust_error(error: Box<dyn std::error::Error>) -> Self {
        Error(Box::new(ErrorData::RustError(error)))
    }

    pub fn as_avm_error(&self) -> Option<Value<'gc>> {
        match &*self.0 {
            ErrorData::AvmError(value) => Some(*value),
            _ => None,
        }
    }
}

impl Debug for Error<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let ErrorData::AvmError(error) = &*self.0 {
            if let Some(error) = error.as_object().and_then(|obj| obj.as_error_object()) {
                return write!(f, "{}", error.display_full());
            }
        }

        match &*self.0 {
            ErrorData::AvmError(error) => write!(f, "AvmError({error:?})"),
            ErrorData::RustError(error) => write!(f, "RustError({error:?})"),
        }
    }
}

// This type is used very frequently, so make sure it doesn't unexpectedly grow.
const _: () = assert!(size_of::<Result<Value<'_>, Error<'_>>>() <= 16);

#[inline(never)]
#[cold]
pub fn make_null_or_undefined_error<'gc>(
    activation: &mut Activation<'_, 'gc>,
    value: Value<'gc>,
    name: Option<&Multiname<'gc>>,
) -> Error<'gc> {
    let class = activation.avm2().classes().typeerror;
    if matches!(value, Value::Undefined) {
        make_error_1010(activation, name)
    } else {
        let mut msg = "Error #1009: Cannot access a property or method of a null object reference."
            .to_string();
        if let Some(name) = name {
            msg.push_str(&format!(
                " (accessing field: {})",
                name.to_qualified_name(activation.gc())
            ));
        }
        match error_constructor(activation, class, &msg, 1009) {
            Ok(err) => Error::avm_error(err),
            Err(err) => err,
        }
    }
}

pub enum ReferenceErrorCode {
    AssignToMethod = 1037,
    InvalidWrite = 1056,
    InvalidRead = 1069,
    WriteToReadOnly = 1074,
    ReadFromWriteOnly = 1077,
    InvalidNsRead = 1081,
    InvalidDelete = 1120,
}

#[inline(never)]
#[cold]
pub fn make_reference_error<'gc>(
    activation: &mut Activation<'_, 'gc>,
    code: ReferenceErrorCode,
    multiname: &Multiname<'gc>,
    object_class: Class<'gc>,
) -> Error<'gc> {
    let qualified_name = multiname.as_uri(activation.strings());
    let class_name = object_class
        .name()
        .to_qualified_name_err_message(activation.gc());

    let msg = match code {
        ReferenceErrorCode::AssignToMethod => format!(
            "Error #1037: Cannot assign to a method {qualified_name} on {class_name}.",
        ),
        ReferenceErrorCode::InvalidWrite => format!(
            "Error #1056: Cannot create property {qualified_name} on {class_name}.",
        ),
        ReferenceErrorCode::InvalidRead => format!(
            "Error #1069: Property {qualified_name} not found on {class_name} and there is no default value.",
        ),
        ReferenceErrorCode::WriteToReadOnly => format!(
            "Error #1074: Illegal write to read-only property {qualified_name} on {class_name}.",
        ),
        ReferenceErrorCode::ReadFromWriteOnly => format!(
            "Error #1077: Illegal read of write-only property {qualified_name} on {class_name}.",
        ),
        ReferenceErrorCode::InvalidNsRead => format!(
            "Error #1081: Property {qualified_name} not found on {class_name} and there is no default value.",
        ),
        ReferenceErrorCode::InvalidDelete => format!(
            "Error #1120: Cannot delete property {qualified_name} on {class_name}.",
        ),
    };

    let class = activation.avm2().classes().referenceerror;
    let error = error_constructor(activation, class, &msg, code as u32);
    match error {
        Ok(err) => Error::avm_error(err),
        Err(err) => err,
    }
}

#[inline(never)]
#[cold]
pub fn make_error_1002<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    let err = range_error(
        activation,
        "Error #1002: Number.toPrecision has a range of 1 to 21. Number.toFixed and Number.toExponential have a range of 0 to 20. Specified value is not within expected range.",
        1002,
    );
    match err {
        Ok(err) => Error::avm_error(err),
        Err(err) => err,
    }
}

#[inline(never)]
#[cold]
pub fn make_error_1003<'gc>(activation: &mut Activation<'_, 'gc>, radix: i32) -> Error<'gc> {
    let err = range_error(
        activation,
        &format!("Error #1003: The radix argument must be between 2 and 36; got {radix}."),
        1003,
    );
    match err {
        Ok(err) => Error::avm_error(err),
        Err(err) => err,
    }
}

#[inline(never)]
#[cold]
pub fn make_error_1006<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    let err = type_error(activation, "Error #1006: value is not a function.", 1006);
    match err {
        Ok(err) => Error::avm_error(err),
        Err(err) => err,
    }
}

#[inline(never)]
#[cold]
pub fn make_error_1010<'gc>(
    activation: &mut Activation<'_, 'gc>,
    name: Option<&Multiname<'gc>>,
) -> Error<'gc> {
    let mut msg = "Error #1010: A term is undefined and has no properties.".to_string();
    if let Some(name) = name {
        msg.push_str(&format!(
            " (accessing field: {})",
            name.to_qualified_name(activation.gc())
        ));
    }
    let error = type_error(activation, &msg, 1010);
    match error {
        Ok(e) => Error::avm_error(e),
        Err(e) => e,
    }
}

pub enum Error1014Type {
    ReferenceError,
    VerifyError,
}

#[inline(never)]
#[cold]
pub fn make_error_1014<'gc>(
    activation: &mut Activation<'_, 'gc>,
    kind: Error1014Type,
    class_name: AvmString<'gc>,
) -> Error<'gc> {
    let message = &format!("Error #1014: Class {class_name} could not be found.");
    let err = match kind {
        Error1014Type::ReferenceError => reference_error(activation, message, 1014),
        Error1014Type::VerifyError => verify_error(activation, message, 1014),
    };
    match err {
        Ok(err) => Error::avm_error(err),
        Err(err) => err,
    }
}

#[inline(never)]
#[cold]
pub fn make_error_1021<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    let err = verify_error(
        activation,
        "Error #1021: At least one branch target was not on a valid instruction in the method.",
        1021,
    );
    match err {
        Ok(err) => Error::avm_error(err),
        Err(err) => err,
    }
}

#[inline(never)]
#[cold]
pub fn make_error_1025<'gc>(activation: &mut Activation<'_, 'gc>, index: u32) -> Error<'gc> {
    let err = verify_error(
        activation,
        &format!("Error #1025: An invalid register {index} was accessed."),
        1025,
    );
    match err {
        Ok(err) => Error::avm_error(err),
        Err(err) => err,
    }
}

#[inline(never)]
#[cold]
pub fn make_error_1032<'gc>(activation: &mut Activation<'_, 'gc>, index: u32) -> Error<'gc> {
    let err = verify_error(
        activation,
        &format!("Error #1032: Cpool index {index} is out of range."),
        1032,
    );
    match err {
        Ok(err) => Error::avm_error(err),
        Err(err) => err,
    }
}

#[inline(never)]
#[cold]
pub fn make_error_1033<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    let err = verify_error(activation, "Error #1033: Cpool entry is wrong type.", 1033);
    match err {
        Ok(err) => Error::avm_error(err),
        Err(err) => err,
    }
}

pub fn make_error_1053<'gc>(
    activation: &mut Activation<'_, 'gc>,
    trait_name: AvmString<'gc>,
    class_name: AvmString<'gc>,
) -> Error<'gc> {
    let err = verify_error(
        activation,
        &format!("Error #1053: Illegal override of {trait_name} in {class_name}."),
        1053,
    );
    match err {
        Ok(err) => Error::avm_error(err),
        Err(err) => err,
    }
}

#[inline(never)]
#[cold]
pub fn make_error_1054<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    let err = verify_error(
        activation,
        "Error #1054: Illegal range or target offsets in exception handler.",
        1054,
    );
    match err {
        Ok(err) => Error::avm_error(err),
        Err(err) => err,
    }
}

#[inline(never)]
#[cold]
pub fn make_error_1065<'gc>(
    activation: &mut Activation<'_, 'gc>,
    name: &Multiname<'gc>,
) -> Error<'gc> {
    let qualified_name = name.as_uri(activation.strings());

    let err = reference_error(
        activation,
        &format!("Error #1065: Variable {qualified_name} is not defined."),
        1065,
    );
    match err {
        Ok(err) => Error::avm_error(err),
        Err(err) => err,
    }
}

#[inline(never)]
#[cold]
pub fn make_error_1080<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    let err = type_error(
        activation,
        "Error #1080: Illegal value for namespace.",
        1080,
    );
    match err {
        Ok(err) => Error::avm_error(err),
        Err(err) => err,
    }
}

#[inline(never)]
#[cold]
pub fn make_error_1085<'gc>(activation: &mut Activation<'_, 'gc>, tag: &str) -> Error<'gc> {
    let err = type_error(
        activation,
        &format!("Error #1085: The element type \"{tag}\" must be terminated by the matching end-tag \"</{tag}>\"."),
        1085,
    );
    match err {
        Ok(err) => Error::avm_error(err),
        Err(err) => err,
    }
}

#[inline(never)]
#[cold]
pub fn make_error_1086<'gc>(activation: &mut Activation<'_, 'gc>, method_name: &str) -> Error<'gc> {
    let err = type_error(
        activation,
        &format!("Error #1086: The {method_name} method only works on lists containing one item."),
        1086,
    );
    match err {
        Ok(err) => Error::avm_error(err),
        Err(err) => err,
    }
}

#[inline(never)]
#[cold]
pub fn make_error_1087<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    let err = type_error(
        activation,
        "Error #1087: Assignment to indexed XML is not allowed.",
        1087,
    );
    match err {
        Ok(err) => Error::avm_error(err),
        Err(err) => err,
    }
}

#[inline(never)]
#[cold]
pub fn make_error_1089<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    let err = type_error(
        activation,
        "Error #1089: Assignment to lists with more than one item is not supported.",
        1089,
    );
    match err {
        Ok(err) => Error::avm_error(err),
        Err(err) => err,
    }
}

#[inline(never)]
#[cold]
pub fn make_error_1098<'gc>(
    activation: &mut Activation<'_, 'gc>,
    prefix: &AvmString<'gc>,
) -> Error<'gc> {
    let err = type_error(
        activation,
        &format!("Error #1098: Illegal prefix {prefix} for no namespace."),
        1098,
    );
    match err {
        Ok(err) => Error::avm_error(err),
        Err(err) => err,
    }
}

#[inline(never)]
#[cold]
pub fn make_error_1107<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    let err = verify_error(
        activation,
        "Error #1107: The ABC data is corrupt, attempt to read out of bounds.",
        1107,
    );
    match err {
        Ok(err) => Error::avm_error(err),
        Err(err) => err,
    }
}

#[inline(never)]
#[cold]
pub fn make_error_1117<'gc>(
    activation: &mut Activation<'_, 'gc>,
    name: AvmString<'gc>,
) -> Error<'gc> {
    let err = type_error(
        activation,
        &format!("Error #1117: Invalid XML name: {}.", name.as_wstr()),
        1117,
    );

    match err {
        Ok(err) => Error::avm_error(err),
        Err(err) => err,
    }
}

#[inline(never)]
#[cold]
pub fn make_error_1118<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    let err = type_error(
        activation,
        "Error #1118: Illegal cyclical loop between nodes.",
        1118,
    );
    match err {
        Ok(err) => Error::avm_error(err),
        Err(err) => err,
    }
}

#[inline(never)]
#[cold]
pub fn make_error_1125<'gc>(
    activation: &mut Activation<'_, 'gc>,
    index: f64,
    range: usize,
) -> Error<'gc> {
    let err = range_error(
        activation,
        &format!("Error #1125: The index {index} is out of range {range}."),
        1125,
    );
    match err {
        Ok(err) => Error::avm_error(err),
        Err(err) => err,
    }
}

#[inline(never)]
#[cold]
pub fn make_error_1127<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    let err = type_error(
        activation,
        "Error #1127: Type application attempted on a non-parameterized type.",
        1127,
    );
    match err {
        Ok(err) => Error::avm_error(err),
        Err(err) => err,
    }
}

#[inline(never)]
#[cold]
pub fn make_error_1506<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    let err = range_error(
        activation,
        "Error #1506: The range specified is invalid.",
        1506,
    );
    match err {
        Ok(err) => Error::avm_error(err),
        Err(err) => err,
    }
}

#[inline(never)]
#[cold]
pub fn make_error_1508<'gc>(activation: &mut Activation<'_, 'gc>, param_name: &str) -> Error<'gc> {
    let err = argument_error(
        activation,
        &format!("Error #1508: The value specified for argument {param_name} is invalid."),
        1508,
    );
    match err {
        Ok(err) => Error::avm_error(err),
        Err(err) => err,
    }
}

pub enum Error2004Type {
    Error,
    ArgumentError,
    TypeError,
}

#[inline(never)]
#[cold]
pub fn make_error_2004<'gc>(
    activation: &mut Activation<'_, 'gc>,
    kind: Error2004Type,
) -> Error<'gc> {
    let message = "Error #2004: One of the parameters is invalid.";
    let err = match kind {
        Error2004Type::Error => error(activation, message, 2004),
        Error2004Type::ArgumentError => argument_error(activation, message, 2004),
        Error2004Type::TypeError => type_error(activation, message, 2004),
    };
    match err {
        Ok(err) => Error::avm_error(err),
        Err(err) => err,
    }
}

#[inline(never)]
#[cold]
pub fn make_error_2006<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    let err = range_error(
        activation,
        "Error #2006: The supplied index is out of bounds.",
        2006,
    );
    match err {
        Ok(err) => Error::avm_error(err),
        Err(err) => err,
    }
}

#[inline(never)]
#[cold]
pub fn make_error_2007<'gc>(activation: &mut Activation<'_, 'gc>, param_name: &str) -> Error<'gc> {
    let err = type_error(
        activation,
        &format!("Error #2007: Parameter {param_name} must be non-null."),
        2007,
    );
    match err {
        Ok(err) => Error::avm_error(err),
        Err(err) => err,
    }
}

#[inline(never)]
#[cold]
pub fn make_error_2008<'gc>(activation: &mut Activation<'_, 'gc>, param_name: &str) -> Error<'gc> {
    let err = argument_error(
        activation,
        &format!("Error #2008: Parameter {param_name} must be one of the accepted values."),
        2008,
    );
    match err {
        Ok(err) => Error::avm_error(err),
        Err(err) => err,
    }
}

#[inline(never)]
#[cold]
pub fn make_error_2012<'gc>(
    activation: &mut Activation<'_, 'gc>,
    class_name: impl Display,
) -> Error<'gc> {
    let err = argument_error(
        activation,
        &format!("Error #2012: {class_name} class cannot be instantiated."),
        2012,
    );
    match err {
        Ok(err) => Error::avm_error(err),
        Err(err) => err,
    }
}

#[inline(never)]
#[cold]
pub fn make_error_2025<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    let err = argument_error(
        activation,
        "Error #2025: The supplied DisplayObject must be a child of the caller.",
        2025,
    );
    match err {
        Ok(err) => Error::avm_error(err),
        Err(err) => err,
    }
}

#[inline(never)]
#[cold]
pub fn make_error_2027<'gc>(activation: &mut Activation<'_, 'gc>, value: i32) -> Error<'gc> {
    let err = range_error(
        activation,
        &format!("Error #2027: Parameter tabIndex must be a non-negative number; got {value}."),
        2027,
    );
    match err {
        Ok(err) => Error::avm_error(err),
        Err(err) => err,
    }
}

#[inline(never)]
#[cold]
pub fn make_error_2037<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    let err = error(
        activation,
        "Error #2037: Functions called in incorrect sequence, or earlier call was unsuccessful.",
        2037,
    );
    match err {
        Ok(err) => Error::avm_error(err),
        Err(err) => err,
    }
}

#[inline(never)]
#[cold]
pub fn make_error_2085<'gc>(activation: &mut Activation<'_, 'gc>, param_name: &str) -> Error<'gc> {
    let err = argument_error(
        activation,
        &format!("Error #2085: Parameter {param_name} must be non-empty string."),
        2007,
    );
    match err {
        Ok(err) => Error::avm_error(err),
        Err(err) => err,
    }
}

#[inline(never)]
#[cold]
pub fn make_error_2097<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    let err = argument_error(
        activation,
        "Error #2097: The FileFilter Array is not in the correct format.",
        2097,
    );
    match err {
        Ok(err) => Error::avm_error(err),
        Err(err) => err,
    }
}

#[inline(never)]
#[cold]
pub fn make_error_2126<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    let err = argument_error(
        activation,
        "Error #2126: NetConnection object must be connected.",
        2126,
    );
    match err {
        Ok(err) => Error::avm_error(err),
        Err(err) => err,
    }
}

#[inline(never)]
#[cold]
pub fn make_error_2136<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    let err = error(
        activation,
        "Error #2136: The SWF file contains invalid data.",
        2136,
    );
    match err {
        Ok(err) => Error::avm_error(err),
        Err(err) => err,
    }
}

#[inline(never)]
#[cold]
pub fn make_error_2162<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    let err = argument_error(
        activation,
        "Error #2162: The Shader output type is not compatible for this operation.",
        2162,
    );
    match err {
        Ok(err) => Error::avm_error(err),
        Err(err) => err,
    }
}

#[inline(never)]
#[cold]
pub fn range_error<'gc>(
    activation: &mut Activation<'_, 'gc>,
    message: &str,
    code: u32,
) -> Result<Value<'gc>, Error<'gc>> {
    let class = activation.avm2().classes().rangeerror;
    error_constructor(activation, class, message, code)
}

#[inline(never)]
#[cold]
pub fn eval_error<'gc>(
    activation: &mut Activation<'_, 'gc>,
    message: &str,
    code: u32,
) -> Result<Value<'gc>, Error<'gc>> {
    let class = activation.avm2().classes().evalerror;
    error_constructor(activation, class, message, code)
}

#[inline(never)]
#[cold]
pub fn argument_error<'gc>(
    activation: &mut Activation<'_, 'gc>,
    message: &str,
    code: u32,
) -> Result<Value<'gc>, Error<'gc>> {
    let class = activation.avm2().classes().argumenterror;
    error_constructor(activation, class, message, code)
}

#[inline(never)]
#[cold]
pub fn security_error<'gc>(
    activation: &mut Activation<'_, 'gc>,
    message: &str,
    code: u32,
) -> Result<Value<'gc>, Error<'gc>> {
    let class = activation.avm2().classes().securityerror;
    error_constructor(activation, class, message, code)
}

#[inline(never)]
#[cold]
pub fn type_error<'gc>(
    activation: &mut Activation<'_, 'gc>,
    message: &str,
    code: u32,
) -> Result<Value<'gc>, Error<'gc>> {
    let class = activation.avm2().classes().typeerror;
    error_constructor(activation, class, message, code)
}

#[inline(never)]
#[cold]
pub fn reference_error<'gc>(
    activation: &mut Activation<'_, 'gc>,
    message: &str,
    code: u32,
) -> Result<Value<'gc>, Error<'gc>> {
    let class = activation.avm2().classes().referenceerror;
    error_constructor(activation, class, message, code)
}

#[inline(never)]
#[cold]
pub fn verify_error<'gc>(
    activation: &mut Activation<'_, 'gc>,
    message: &str,
    code: u32,
) -> Result<Value<'gc>, Error<'gc>> {
    let class = activation.avm2().classes().verifyerror;
    error_constructor(activation, class, message, code)
}

#[inline(never)]
#[cold]
pub fn illegal_operation_error<'gc>(
    activation: &mut Activation<'_, 'gc>,
    message: &str,
    code: u32,
) -> Result<Value<'gc>, Error<'gc>> {
    let class = activation.avm2().classes().illegaloperationerror;
    error_constructor(activation, class, message, code)
}

#[inline(never)]
#[cold]
pub fn io_error<'gc>(
    activation: &mut Activation<'_, 'gc>,
    message: &str,
    code: u32,
) -> Result<Value<'gc>, Error<'gc>> {
    let class = activation.avm2().classes().ioerror;
    error_constructor(activation, class, message, code)
}

#[inline(never)]
#[cold]
pub fn eof_error<'gc>(
    activation: &mut Activation<'_, 'gc>,
    message: &str,
    code: u32,
) -> Result<Value<'gc>, Error<'gc>> {
    let class = activation.avm2().classes().eoferror;
    error_constructor(activation, class, message, code)
}

#[inline(never)]
#[cold]
pub fn uri_error<'gc>(
    activation: &mut Activation<'_, 'gc>,
    message: &str,
    code: u32,
) -> Result<Value<'gc>, Error<'gc>> {
    let class = activation.avm2().classes().urierror;
    error_constructor(activation, class, message, code)
}

#[inline(never)]
#[cold]
pub fn syntax_error<'gc>(
    activation: &mut Activation<'_, 'gc>,
    message: &str,
    code: u32,
) -> Result<Value<'gc>, Error<'gc>> {
    let class = activation.avm2().classes().syntaxerror;
    error_constructor(activation, class, message, code)
}

#[inline(never)]
#[cold]
pub fn error<'gc>(
    activation: &mut Activation<'_, 'gc>,
    message: &str,
    code: u32,
) -> Result<Value<'gc>, Error<'gc>> {
    let class = activation.avm2().classes().error;
    error_constructor(activation, class, message, code)
}

#[inline(never)]
#[cold]
pub fn make_mismatch_error<'gc>(
    activation: &mut Activation<'_, 'gc>,
    method: Method<'gc>,
    passed_arg_count: usize,
    bound_class: Option<Class<'gc>>,
) -> Result<Value<'gc>, Error<'gc>> {
    let expected_num_params = method
        .signature()
        .iter()
        .filter(|param| param.default_value.is_none())
        .count();

    let mut function_name = WString::new();

    display_function(&mut function_name, method, bound_class);

    return Err(Error::avm_error(argument_error(
        activation,
        &format!(
            "Error #1063: Argument count mismatch on {function_name}. Expected {expected_num_params}, got {passed_arg_count}.",
        ),
        1063,
    )?));
}

fn error_constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    class: ClassObject<'gc>,
    message: &str,
    code: u32,
) -> Result<Value<'gc>, Error<'gc>> {
    let message = AvmString::new_utf8(activation.gc(), message);

    class.construct(activation, &[message.into(), code.into()])
}

impl std::fmt::Display for Error<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

// Ideally, all of these impls would be unified under a single
// `impl<E: std::error::Error> From<E> for Error<'gc>`
// However, this would conflict with the 'str' and 'String'
// impls, which are still widely used.

impl<'gc, 'a> From<&'a str> for Error<'gc> {
    fn from(val: &'a str) -> Error<'gc> {
        Error::rust_error(val.into())
    }
}

impl<'gc> From<String> for Error<'gc> {
    fn from(val: String) -> Error<'gc> {
        Error::rust_error(val.into())
    }
}

impl<'gc> From<ruffle_render::error::Error> for Error<'gc> {
    fn from(val: ruffle_render::error::Error) -> Error<'gc> {
        Error::rust_error(val.into())
    }
}
