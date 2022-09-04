use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{parse_macro_input, FnArg, ItemFn, Pat};

// A `#[native]` method is of the form `fn(&mut Activation, ReceiverType, Arg0Type, Arg1Type, ...)
// When looking for arguments, we skip over the first 2 parameters.
const NUM_NON_ARG_PARAMS: usize = 2;

pub fn native_impl(_args: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let vis = &input.vis;
    let method_name = &input.sig.ident;

    // Extracts `(name, Type)` from a parameter definition of the form `name: Type`
    let get_param_ident_ty = |arg| {
        let arg: &FnArg = arg;
        match arg {
            FnArg::Typed(pat) => {
                if let Pat::Ident(ident) = &*pat.pat {
                    (&ident.ident, &pat.ty)
                } else {
                    panic!("Only normal (ident pattern) parameters are supported, found {pat:?}",)
                }
            }
            FnArg::Receiver(_) => {
                panic!("#[native] attribute can only be used on freestanding functions!")
            }
        }
    };

    let num_params = input.sig.inputs.len() - NUM_NON_ARG_PARAMS;

    // Generates names for use in `let` bindings.
    let arg_names =
        (0..num_params).map(|index| Ident::new(&format!("arg{index}"), Span::call_site()));

    // Generates `let argN = <arg_extraction_code>;` for each argument.
    // The separate 'let' bindings allow us to use `activation` in `<arg_extraction_code>`
    // without causing a borrowcheck error - otherwise, we could just put the extraction code
    // in the call expression.on (e.g. `user_fn)
    let arg_extractions = input.sig.inputs.iter().skip(NUM_NON_ARG_PARAMS).enumerate().zip(arg_names.clone()).map(|((index, arg), arg_name)| {
        let (param_ident, param_ty) = get_param_ident_ty(arg);

        let param_name = param_ident.to_string();
        // Only use location information from `param_ident`. This ensures that
        // tokens emitted using `param_span` will be able to access local variables
        // defined by this macro, even if this macro is used from within another macro.
        let param_ty_span = param_ty.span().resolved_at(Span::call_site());
        let param_ty_name = param_ty.to_token_stream().to_string();

        // Subtle: We build up this error message in two parts.
        // For a native method `fn my_method(activation, this, my_argument: bool)`, we would produce the following string:
        // "my_method: Argument extraction failed for parameter `my_argument`: argument {:?} is not a `bool`".
        // Note that this string contains a `{:?}` format specifier - this will be substituted at *runtime*, when we have
        // the actual parameter value.
        let error_message_fmt = format!("{method_name}: Argument extraction failed for parameter `{param_name}`: argument {{:?}} is not a `{param_ty_name}`");

        // Use `quote_spanned` to make compile-time error messages point at the argument in the function definition.
        quote_spanned! { param_ty_span=>
            let #arg_name = if let Some(arg) = crate::avm2::extract::ExtractFromVm::extract_from(&args[#index], activation) {
                arg
            } else {
                // As described above, `error_message_fmt` contains a `{:?}` for the actual argument value.
                // Substitute it in with `format!`
                return Err(format!(#error_message_fmt, args[#index]).into())
            };
        }
    });

    let receiver_ty = get_param_ident_ty(
        input
            .sig
            .inputs
            .iter()
            .nth(1)
            .expect("Missing 'this' parameter"),
    )
    .1;

    let reciever_ty_span = receiver_ty.span();

    let receiver_ty_name = receiver_ty.to_token_stream().to_string();

    // These format strings are handled in the same way as `error_message_fmt` above.
    let error_message_fmt = format!(
        "{method_name}: Receiver extraction failed: receiver {{:?}} is not a `{receiver_ty_name}`"
    );
    let mismatched_arg_error_fmt =
        format!("{method_name}: Expected {num_params} arguments, found {{:?}}");

    let receiver_extraction = quote_spanned! { reciever_ty_span=>
        // Try to extract the requested reciever type.
        let this = this.map(Value::Object);
        let this = if let Some(this) = crate::avm2::extract::ReceiverHelper::extract_from(&this, activation) {
            this
        } else {
            return Err(format!(#error_message_fmt, this).into());
        };
    };

    let output = quote! {
        // Generate a method with the proper `NativeMethodImpl` signature
        #vis fn #method_name<'gc>(
            activation: &mut crate::avm2::Activation<'_, 'gc>,
            this: Option<crate::avm2::Object<'gc>>,
            args: &[crate::avm2::Value<'gc>]
        ) -> Result<crate::avm2::Value<'gc>, crate::avm2::Error<'gc>> {
            // Paste the entire function definition provided by the user.
            // It will only be accessible here, so other code will only see
            // our generated function.
            #input;

            // Generate an error if called with the wrong number of parameters
            if args.len() != #num_params {
                return Err(format!(#mismatched_arg_error_fmt, args.len()).into());
            }

            // Emit `let this = <receiver_extraction_code>;` for the receiver
            #receiver_extraction

            // Emit `let argN = <arg_extraction_code>;` for each argument.
            #(#arg_extractions)*

            // Finally, call the user's method with the extracted receiver and arguments.
            #method_name (
                activation, this, #({ #arg_names } ),*
            )
        }
    };
    output.into()
}
