use crate::avm2::activation::Activation;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::string::AvmString;
use ruffle_wstr::from_utf8;

pub fn _escape_xml<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let and = from_utf8("&");
    let lt = from_utf8("<");
    let gt = from_utf8(">");
    let quote = from_utf8("\"");
    let apos = from_utf8("'");

    let input = args.get_string(activation, 0)?;
    if input.contains(and.as_ref())
        || input.contains(lt.as_ref())
        || input.contains(gt.as_ref())
        || input.contains(quote.as_ref())
        || input.contains(apos.as_ref())
    {
        let result = input
            .replace(and.as_ref(), from_utf8("&amp;").as_ref())
            .replace(lt.as_ref(), from_utf8("&lt;").as_ref())
            .replace(gt.as_ref(), from_utf8("&gt;").as_ref())
            .replace(quote.as_ref(), from_utf8("&quot;").as_ref())
            .replace(apos.as_ref(), from_utf8("&apos;").as_ref());
        Ok(AvmString::new(activation.gc(), result).into())
    } else {
        // Save the allocation, just return input
        Ok(input.into())
    }
}
