//! Proc macros used by Ruffle to generate various boilerplate.
extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{
    DeriveInput, FnArg, ImplItem, ImplItemFn, ItemEnum, ItemTrait, LitStr, Meta, Pat, TraitItem,
    Visibility, parse_macro_input, parse_quote,
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
            TraitItem::Fn(method) => {
                let mut is_no_dynamic = false;

                method.attrs.retain(|attr| match &attr.meta {
                    Meta::Path(path) if path.is_ident("no_dynamic") => {
                        is_no_dynamic = true;

                        // Remove the #[no_dynamic] attribute from the
                        // list of method attributes.
                        false
                    }
                    _ => true,
                });

                let params: Vec<_> = method
                    .sig
                    .inputs
                    .iter()
                    .filter_map(|arg| {
                        if let FnArg::Typed(arg) = arg && let Pat::Ident(i) = &*arg.pat {
                            return Some(i.ident.clone());
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

/// Sibling of [`enum_trait_object`] that emits a pointer-sized tagged handle
/// instead of an enum.
///
/// Takes the same input shape: an `enum`-shaped variant list inside the
/// attribute, plus the annotated trait. Generates:
///
/// * A `#[repr(transparent)]` struct with the enum's name, wrapping
///   `Gc<'gc, crate::avm2::object::ScriptObjectData<'gc, crate::avm2::object::kind::Erased>>`.
/// * An `impl Trait for Struct` whose method bodies match on
///   `self.0.kind()` and dispatch via `crate::avm2::object::cast::downcast_unchecked`.
/// * `From<Variant> for Struct` impls backed by
///   `crate::avm2::object::cast::upcast`.
///
/// `#[no_dynamic]` is handled identically to `enum_trait_object`: the original
/// default body is moved into the struct's impl, the trait's default body
/// becomes a delegating `(*self).into().method(...)` call, and the NoDyn
/// pub-in-private trait prevents variants from overriding.
///
/// The inner Gc type, kind accessor, and cast functions are hardcoded to the
/// paths above — this macro is specifically for the AVM2 `Object` type.
#[proc_macro_attribute]
pub fn tagged_trait_object(args: TokenStream, item: TokenStream) -> TokenStream {
    let mut input_trait = parse_macro_input!(item as ItemTrait);
    let trait_name = &input_trait.ident;
    let trait_generics = &input_trait.generics;
    let enum_input = parse_macro_input!(args as ItemEnum);
    let struct_name = &enum_input.ident;

    assert!(
        trait_generics.lifetimes().count() == 1,
        "tagged_trait_object requires exactly one lifetime parameter"
    );
    assert!(
        trait_generics.type_params().count() == 0,
        "Generic type parameters are currently unsupported"
    );
    assert_eq!(
        trait_generics, &enum_input.generics,
        "Trait and struct should have the same generic parameters"
    );

    let lt = &trait_generics.lifetimes().next().unwrap().lifetime;

    // Same NoDyn pub-in-private trick as enum_trait_object for #[no_dynamic]
    // methods: variants inherit a delegating default body and cannot override.
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

    let trait_methods: Vec<_> = input_trait
        .items
        .iter_mut()
        .map(|item| match item {
            TraitItem::Fn(method) => {
                let mut is_no_dynamic = false;
                method.attrs.retain(|attr| match &attr.meta {
                    Meta::Path(path) if path.is_ident("no_dynamic") => {
                        is_no_dynamic = true;
                        false
                    }
                    _ => true,
                });

                let params: Vec<_> = method
                    .sig
                    .inputs
                    .iter()
                    .filter_map(|arg| {
                        if let FnArg::Typed(arg) = arg && let Pat::Ident(i) = &*arg.pat {
                            return Some(i.ident.clone());
                        }
                        None
                    })
                    .collect();

                let self_token: syn::Token![self] = method
                    .sig
                    .receiver()
                    .map(|r| r.self_token)
                    .unwrap_or_else(|| syn::Token![self](Span::call_site()));

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

                    method
                        .default
                        .replace(parse_quote!({
                            let mut o: #struct_name<'_> = (#deref #self_token).into();
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
                            let variant_type = &variant
                                .fields
                                .iter()
                                .next()
                                .expect("Missing field for enum variant")
                                .ty;
                            quote! {
                                crate::avm2::object::kind::ObjectKind::#variant_name => {
                                    // SAFETY: the kind tag matches T::Kind::ID for this variant,
                                    // so the allocation is originally of this variant's Data type.
                                    let o: #variant_type = unsafe {
                                        crate::avm2::object::cast::downcast_unchecked(#self_token.0)
                                    };
                                    o.#method_name(#(#params),*)
                                }
                            }
                        })
                        .collect();

                    parse_quote!({
                        match #self_token.0.kind() {
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

    let from_impls: Vec<_> = enum_input
        .variants
        .iter()
        .map(|variant| {
            let variant_type = &variant
                .fields
                .iter()
                .next()
                .expect("Missing field for enum variant")
                .ty;
            quote!(
                impl #impl_generics From<#variant_type> for #struct_name #ty_generics {
                    fn from(obj: #variant_type) -> #struct_name #trait_generics {
                        #struct_name(crate::avm2::object::cast::upcast(obj))
                    }
                }
            )
        })
        .collect();

    // Copy the enum's attrs onto the struct, dropping enum-specific lints and
    // filtering `Debug` out of `#[derive(...)]` (auto-deriving Debug on the
    // struct's `Gc<ScriptObjectData<Erased>>` field doesn't work because
    // `ScriptObjectData` doesn't itself implement Debug; we emit a manual impl
    // further down).
    let mut derived_debug = false;
    let struct_attrs: Vec<_> = enum_input
        .attrs
        .iter()
        .filter_map(|attr| match &attr.meta {
            Meta::List(list) if list.path.is_ident("expect") || list.path.is_ident("allow") => {
                if list.tokens.to_string().contains("enum_variant_names") {
                    None
                } else {
                    Some(attr.clone())
                }
            }
            Meta::List(list) if list.path.is_ident("derive") => {
                let tokens = list.tokens.to_string();
                if tokens.split(',').any(|t| t.trim() == "Debug") {
                    derived_debug = true;
                    // Strip just `Debug` from the derive list. If removing it leaves
                    // nothing, drop the whole attribute.
                    let kept: Vec<_> = tokens
                        .split(',')
                        .map(|t| t.trim())
                        .filter(|t| *t != "Debug")
                        .collect();
                    if kept.is_empty() {
                        None
                    } else {
                        let list_tokens: TokenStream2 = kept
                            .join(", ")
                            .parse()
                            .expect("Failed to re-parse derive list");
                        let path = &list.path;
                        Some(syn::parse_quote! { #[#path(#list_tokens)] })
                    }
                } else {
                    Some(attr.clone())
                }
            }
            _ => Some(attr.clone()),
        })
        .collect();
    let struct_vis = &enum_input.vis;
    let struct_generics = &enum_input.generics;

    let debug_impl = if derived_debug {
        quote! {
            impl #impl_generics ::core::fmt::Debug for #struct_name #ty_generics #where_clause {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    f.debug_tuple(stringify!(#struct_name))
                        .field(&self.0.kind())
                        .finish()
                }
            }
        }
    } else {
        quote!()
    };

    let struct_def = quote! {
        #(#struct_attrs)*
        #[repr(transparent)]
        #struct_vis struct #struct_name #struct_generics(
            pub ::gc_arena::Gc<
                #lt,
                crate::avm2::object::ScriptObjectData<
                    #lt,
                    crate::avm2::object::kind::Erased,
                >,
            >,
        );
    };

    let no_override = no_override.map(|s| s.contents).into_iter();
    let out = quote!(
        #(#no_override)*

        #input_trait

        #struct_def

        impl #impl_generics #trait_name #ty_generics for #struct_name #ty_generics #where_clause {
            #(#trait_methods)*
        }

        #debug_impl

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
        panic!(
            "`HasPrefixField` can only be derived for repr(C) structs with at least one named field"
        );
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
                ruffle_common::utils::HasPrefixField<#field_ty>
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

/// A helper used to define common strings.
///
/// Generates field names and string values and passes them along to
/// define_common_strings_impl to implement the structure.
#[proc_macro]
pub fn define_common_strings(input: TokenStream) -> TokenStream {
    struct Input {
        ascii_ident: syn::Ident,
        strings: Vec<syn::LitStr>,
    }

    impl Parse for Input {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            let ascii_ident: syn::Ident = input.parse()?;
            input.parse::<syn::Token![,]>()?;

            let mut strings = Vec::new();
            while !input.is_empty() {
                strings.push(input.parse()?);
                if !input.is_empty() {
                    input.parse::<syn::Token![,]>()?;
                }
            }
            Ok(Self {
                ascii_ident,
                strings,
            })
        }
    }

    let input = parse_macro_input!(input as Input);
    let idents: Vec<_> = input
        .strings
        .iter()
        .map(|s| common_str_ident(&s.value()))
        .collect();

    let strings: Vec<_> = input
        .strings
        .iter()
        .map(|s| {
            let value = s.value();
            assert!(value.is_ascii(), "Non-ASCII common strings unsupported");
            syn::LitByteStr::new(value.as_bytes(), s.span())
        })
        .collect();

    let ascii_ident = &input.ascii_ident;

    quote! {
        define_common_strings_impl! {
            #ascii_ident,
            #( #idents: #strings, )*
        }
    }
    .into()
}

fn common_str_ident(s: &str) -> syn::Ident {
    let mut ident = String::from("str_");
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            ident.push(c);
        } else {
            ident.push_str(&format!("_{:02x}", c as u32));
        }
    }
    format_ident!("{ident}")
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
        (common_str_ident(&string), None)
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
