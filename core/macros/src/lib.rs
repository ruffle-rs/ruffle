//! Proc macros used by Ruffle to generate various boilerplate.
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, parse_quote, FnArg, ImplItem, ImplItemMethod, ItemEnum, ItemTrait, Pat,
    TraitItem, Visibility,
};

mod native;

/// `enum_trait_object` will define an enum whose variants each implement a trait.
/// It can be used as faux-dynamic dispatch. This is used as an alternative to a
/// trait object, which doesn't get along with GC'd types.
///
/// This will auto-implement the trait for the enum, delegating all methods to the
/// underlying type. Additionally, `From` will be implemented for all of the variants,
/// so an underlying type can easily be converted into the enum.
///
/// TODO: This isn't completely robust for all cases, but should be good enough
/// for our usage.
///
/// Usage:
/// ```
/// use ruffle_macros::enum_trait_object;
///
/// #[enum_trait_object(
///     pub enum MyTraitEnum {
///         Object(Object)
///     }
/// )]
/// trait MyTrait {}
///
/// struct Object {}
/// impl MyTrait for Object {}
/// ```
#[proc_macro_attribute]
pub fn enum_trait_object(args: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input.
    let input_trait = parse_macro_input!(item as ItemTrait);
    let trait_name = input_trait.ident.clone();
    let trait_generics = input_trait.generics.clone();
    let enum_input = parse_macro_input!(args as ItemEnum);
    let enum_name = enum_input.ident.clone();

    // TODO: Revise whether the first two asserts are needed at all, and whether
    // the second condition should be `== 0` instead, based on the error message.
    assert!(
        trait_generics.lifetimes().count() <= 1,
        "Only one lifetime parameter is currently supported"
    );

    assert!(
        trait_generics.type_params().count() <= 1,
        "Generic type parameters are currently unsupported"
    );

    assert_eq!(
        trait_generics, enum_input.generics,
        "Trait and enum should have the same generic parameters"
    );

    // Implement each trait. This will match against each enum variant and delegate
    // to the underlying type.
    let trait_methods: Vec<_> = input_trait
        .items
        .iter()
        .map(|item| match item {
            TraitItem::Method(method) => {
                let method_name = method.sig.ident.clone();
                let params: Vec<_> = method
                    .sig
                    .inputs
                    .iter()
                    .filter_map(|arg| {
                        if let FnArg::Typed(arg) = arg {
                            if let Pat::Ident(i) = &*arg.pat {
                                let arg_name = i.ident.clone();
                                return Some(quote!(#arg_name,));
                            }
                        }
                        None
                    })
                    .collect();

                let match_arms: Vec<_> = enum_input
                    .variants
                    .iter()
                    .map(|variant| {
                        let variant_name = variant.ident.clone();
                        quote! {
                            #enum_name::#variant_name(o) => o.#method_name(#(#params)*),
                        }
                    })
                    .collect();
                let method_block = quote!({
                    match self {
                        #(#match_arms)*
                    }
                });

                ImplItem::Method(ImplItemMethod {
                    attrs: method.attrs.clone(),
                    vis: Visibility::Inherited,
                    defaultness: None,
                    sig: method.sig.clone(),
                    block: parse_quote!(#method_block),
                })
            }
            _ => panic!("Unsupported trait item: {item:?}"),
        })
        .collect();

    let (impl_generics, ty_generics, where_clause) = trait_generics.split_for_impl();

    // Implement `From` for each variant type.
    let from_impls: Vec<_> = enum_input
        .variants
        .iter()
        .map(|variant| {
            let variant_name = variant.ident.clone();
            let variant_type = variant
                .fields
                .iter()
                .next()
                .expect("Missing field for enum variant")
                .ty
                .clone();

            quote!(
                impl #impl_generics From<#variant_type> for #enum_name #ty_generics {
                    fn from(obj: #variant_type) -> #enum_name #trait_generics {
                        #enum_name::#variant_name(obj)
                    }
                }
            )
        })
        .collect();

    let out = quote!(
        #input_trait

        #enum_input

        impl #impl_generics #trait_name #ty_generics for #enum_name #ty_generics #where_clause {
            #(#trait_methods)*
        }

        #(#from_impls)*
    );

    out.into()
}

/// The `native` attribute allows you to implement an ActionScript
/// impl using native Rust/ruffle types, rather than taking in a `&[Value<'gc>]`
/// of arguments.
///
/// For example, consider the following native function in 'Event.as':
///
/// ```text
/// private native function init(type:String, bubbles:Boolean = false, cancelable:Boolean = false):void;
/// ```
///
/// Using the `#\[native\] macro, we can implement it like this:
///
/// ```rust,compile_fail
/// #[native]
///pub fn init<'gc>(
///    _activation: &mut Activation<'_, 'gc, '_>,
///    mut this: RefMut<'_, Event<'gc>>,
///    event_type: AvmString<'gc>,
///    bubbles: bool,
///    cancelable: bool
///) -> Result<Value<'gc>, Error<'gc>> {
///    this.set_event_type(event_type);
///    this.set_bubbles(bubbles);
///    this.set_cancelable(cancelable);
///    Ok(Value::Undefined)
///}
///```
///
/// We specify the desired types for `this`, along with each paramter.
/// The `#[native]` macro will generate a function with the normal `NativeMethodImpl`,
/// which will check that all of the arguments have the expected type, and call your function.
/// If the receiver (`this`) or any of the argument `Value`s do *not* match your declared types,
/// the generated function will return an error without calling your function.
///
/// To see all supported argument types, look at the `ExtractFromVm` impls in `core::avm2::extract`
///
/// Note: This macro **does not** perform *coercions* (e.g. `coerce_to_object`, `coerce_to_number`).
/// Instead, it checks that the receiver/argument is *already* of the correct type (e.g. `Value::Object` or `Value::Number`).
///
/// The actual coercion logic should have been already performed by Ruffle by the time the generated method is called:
/// * For native methods, this is done by `resolve_parameter` using the type signature defined in the loaded bytecode
/// * For methods defined entirely in Rust, you must define the method using `Method::from_builtin_and_params`.
///   Using a native method is much easier.
#[proc_macro_attribute]
pub fn native(args: TokenStream, item: TokenStream) -> TokenStream {
    native::native_impl(args, item)
}
