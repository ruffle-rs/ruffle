//! Proc macros used by Ruffle to generate various boilerplate.
extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{
    parse_macro_input, parse_quote, DeriveInput, FnArg, ImplItem, ImplItemFn, ItemEnum, ItemTrait,
    LitStr, Meta, Pat, TraitItem, Visibility,
};

/// Define an enum whose variants each implement a trait.
///
/// It can be used as faux-dynamic dispatch. This is used as an alternative to a
/// trait object, which doesn't get along with GC'd types.
///
/// This will auto-implement the trait for the enum, delegating all methods to the
/// underlying type. Additionally, `From` will be implemented for all of the variants,
/// so an underlying type can easily be converted into the enum.
///
/// Methods can be individually marked with `#[no_dynamic]`, which will exempt them from
/// being dynamically dispatched, preventing implementors from overriding them.
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
    let mut input_trait = parse_macro_input!(item as ItemTrait);
    let trait_name = &input_trait.ident;
    let trait_generics = &input_trait.generics;
    let enum_input = parse_macro_input!(args as ItemEnum);
    let enum_name = &enum_input.ident;

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
        trait_generics, &enum_input.generics,
        "Trait and enum should have the same generic parameters"
    );

    /// An hacky way to prevent accidental method overriding.
    ///
    /// We modify the method signature to include a 'dummy' lifetime, which doesn't
    /// disrupt callers but forces implementors to mention a pub-in-private trait.
    ///
    /// This isn't fool-proof (an implementor in a submodule of the trait's module
    /// can still manually write the modified signature), and the error messages
    /// aren't great, but this is good enough for us.
    struct NoOverrideModule {
        mod_name: syn::Ident,
        lt: syn::Lifetime,
        contents: TokenStream2,
    }

    impl NoOverrideModule {
        fn make(trait_name: &syn::Ident) -> Self {
            let mod_name = syn::Ident::new(
                &format!("__{trait_name}_do_not_override"),
                Span::call_site(),
            );
            let lt = syn::Lifetime::new("'no_dyn", Span::call_site());
            let contents = quote! {
                #[automatically_derived]
                #[doc(hidden)]
                mod #mod_name {
                    pub trait NoDyn<#lt> {}
                    impl NoDyn<'_> for () {}
                }
            };
            Self {
                mod_name,
                lt,
                contents,
            }
        }

        fn adjust_method(&self, method: &mut syn::TraitItemFn) {
            let Self { mod_name, lt, .. } = self;
            let generics = &mut method.sig.generics;
            generics
                .params
                .insert(0, syn::LifetimeParam::new(lt.clone()).into());
            generics
                .make_where_clause()
                .predicates
                .push(parse_quote!((): #mod_name::NoDyn<#lt>));
        }
    }

    let mut no_override: Option<NoOverrideModule> = None;

    // Implement each trait. This will match against each enum variant and delegate
    // to the underlying type.
    let trait_methods: Vec<_> = input_trait
        .items
        .iter_mut()
        .map(|item| match item {
            TraitItem::Fn(ref mut method) => {
                let mut is_no_dynamic = false;

                method.attrs.retain(|attr| match &attr.meta {
                    Meta::Path(path) => {
                        if path.is_ident("no_dynamic") {
                            is_no_dynamic = true;

                            // Remove the #[no_dynamic] attribute from the
                            // list of method attributes.
                            false
                        } else {
                            true
                        }
                    }
                    _ => true,
                });

                let params: Vec<_> = method
                    .sig
                    .inputs
                    .iter()
                    .filter_map(|arg| {
                        if let FnArg::Typed(arg) = arg {
                            if let Pat::Ident(i) = &*arg.pat {
                                return Some(i.ident.clone());
                            }
                        }
                        None
                    })
                    .collect();

                let method_block = if is_no_dynamic {
                    no_override
                        .get_or_insert_with(|| NoOverrideModule::make(trait_name))
                        .adjust_method(method);

                    let method_name = &method.sig.ident;
                    let deref = if let Some(syn::Receiver {
                        colon_token: None,
                        reference,
                        ..
                    }) = method.sig.receiver()
                    {
                        reference.is_some().then(|| quote!(*))
                    } else {
                        panic!("#[no_dynamic] method `{method_name}` must take `self`, `&self`, or `&mut self`")
                    };

                    // Moves the provided default body to the enum's generated trait impl,
                    // and replace it by an impl that delegates to the enum.
                    method
                        .default
                        .replace(parse_quote!({
                            let mut o: #enum_name<'_> = (#deref self).into();
                            o.#method_name(#(#params),*)
                        }))
                        .expect("#[no_dynamic] method `{method_name}` must have a default body")
                } else {
                    let method_name = &method.sig.ident;
                    let match_arms: Vec<_> = enum_input
                        .variants
                        .iter()
                        .map(|variant| {
                            let variant_name = &variant.ident;
                            quote! {
                                #enum_name::#variant_name(o) => o.#method_name(#(#params),*),
                            }
                        })
                        .collect();

                    parse_quote!({
                        match self {
                            #(#match_arms)*
                        }
                    })
                };

                ImplItem::Fn(ImplItemFn {
                    attrs: method.attrs.clone(),
                    vis: Visibility::Inherited,
                    defaultness: None,
                    sig: method.sig.clone(),
                    block: method_block,
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
            let variant_name = &variant.ident;
            let variant_type = &variant
                .fields
                .iter()
                .next()
                .expect("Missing field for enum variant")
                .ty;

            quote!(
                impl #impl_generics From<#variant_type> for #enum_name #ty_generics {
                    fn from(obj: #variant_type) -> #enum_name #trait_generics {
                        #enum_name::#variant_name(obj)
                    }
                }
            )
        })
        .collect();

    let no_override = no_override.map(|s| s.contents).into_iter();
    let out = quote!(
        #(#no_override)*

        #input_trait

        #enum_input

        impl #impl_generics #trait_name #ty_generics for #enum_name #ty_generics #where_clause {
            #(#trait_methods)*
        }

        #(#from_impls)*
    );

    out.into()
}

#[proc_macro_derive(HasPrefixField)]
pub fn derive_has_prefix_field(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let mut is_repr_c = false;
    for attr in &input.attrs {
        if attr.path().is_ident("repr") {
            // Ignore parse errors.
            let _ = attr.parse_nested_meta(|meta| {
                is_repr_c = is_repr_c || meta.path.is_ident("C");
                Ok(())
            });
        }
    }

    let Some(first_field) = ({
        if let syn::Data::Struct(data) = &input.data {
            data.fields
                .iter()
                .next()
                .filter(|f| is_repr_c && f.ident.is_some())
        } else {
            None
        }
    }) else {
        panic!("`HasPrefixField` can only be derived for repr(C) structs with at least one named field");
    };

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let (ty, field_ty, field_name) = (
        &input.ident,
        &first_field.ty,
        first_field.ident.as_ref().unwrap(),
    );

    quote! {
        // SAFETY: `repr(C)` structs always have their first field at offset 0.
        // Technically, an attribute macro executing after this derive could rewrite the struct
        // definition (see <https://github.com/google/zerocopy/issues/388#issuecomment-1737817682>
        // for a worked-out example), so we add post-mono checks as a latch-ditch guard.
        #[automatically_derived]
        unsafe impl #impl_generics
                crate::utils::HasPrefixField<#field_ty>
                for #ty #ty_generics #where_clause {
            const ASSERT_PREFIX_FIELD: () = {
                ::core::assert!(::core::mem::offset_of!(Self, #field_name) == 0);
                // Check that the field exists and has the correct type.
                let _ = |check: &(Self,)| -> *const #field_ty {
                    let (Self { #field_name, .. },) = check;
                    #field_name as *const _
                };
            };
        }
    }
    .into()
}

/// Get the string passed to it as an interned `AvmAtom`, assumed to be present on
/// the current `StringContext`.
///
/// If no extra parameter is passed, an `activation: Activation<'_, 'gc>` variable will be
/// assumed to be in scope and will be used to retrieve the interned string. Otherwise, the
/// extra parameter should implement the `HasStringContext` trait.
///
/// ```rs
/// istr!("description");
/// // expands to:
/// activation.context.strings.common().str_description;
///
/// istr!(context, "description");
/// // expands to:
/// HasStringContext::strings_ref(context).str_description;
///
/// istr!("A");
/// // expands to:
/// activation.context.strings.common().ascii_chars[65 /* 'A' */];
/// ```
#[proc_macro]
pub fn atom(item: TokenStream) -> TokenStream {
    atom_internal(item, |atom| atom)
}

/// Like `atom!`, but returns an `AvmString` instead of an `AvmAtom`.
#[proc_macro]
pub fn istr(item: TokenStream) -> TokenStream {
    atom_internal(item, |atom| {
        quote!(
            crate::string::AvmString::from(#atom)
        )
    })
}

fn atom_internal(
    item: TokenStream,
    transform: impl FnOnce(TokenStream2) -> TokenStream2,
) -> TokenStream {
    struct Input {
        str: LitStr,
        context: Option<syn::Expr>,
    }

    impl Parse for Input {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            let mut context = None;
            if !input.peek(syn::LitStr) {
                context = Some(input.parse()?);
                input.parse::<syn::token::Comma>()?;
            }

            let str = input.parse()?;
            Ok(Self { context, str })
        }
    }

    let input = parse_macro_input!(item as Input);

    let string = input.str.value();
    let (string_ident, array_index) = if string.len() == 1 && string.is_ascii() {
        // Special case: a single ASCII char.
        let c = string.as_bytes()[0];
        (format_ident!("ascii_chars"), Some(c as usize))
    } else {
        (format_ident!("str_{string}"), None)
    };

    let mut atom = if let Some(context) = input.context {
        quote!(
            crate::string::HasStringContext::strings_ref(#context).common().#string_ident
        )
    } else {
        quote!(
            // Use raw field access instead of `HasStringContext` here:
            // - it's more permissive for the borrow checker;
            // - it works for both by-ref and by-value `Activation`s.
            activation.context.strings.common().#string_ident
        )
    };

    if let Some(i) = array_index {
        atom.extend(quote!([#i]));
    }

    transform(atom).into()
}
