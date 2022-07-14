use proc_macro2::{Span, TokenStream};
use syn::parse::Parser;

use crate::bitfields::parse;
use crate::utils::SynErrors;

type DeriveFn = Box<dyn Fn(&parse::BitfieldsStruct) -> TokenStream>;

mod derive_fns {
    use proc_macro2::{Span, TokenStream};
    use quote::quote_spanned;

    use super::parse;

    macro_rules! impl_trait {
        {$span:ident =>
            impl $trait:ident for #$item:ident
            $impl:tt
        } => {{
            let type_name = &$item.ident;
            let (type_generics, impl_generics, where_clause) = $item.generics.split_for_impl();
            quote_spanned! {$span=>
                impl #impl_generics ::superbitty::__helpers::$trait
                    for #type_name #type_generics
                #where_clause
                $impl
            }
        }};
    }

    pub(super) fn debug(span: Span, item: &parse::BitfieldsStruct) -> TokenStream {
        let type_name_as_string = item.ident.to_string();
        let fields =
            item.fields.fields.iter().map(|parse::BitfieldsStructField { ident: name, .. }| {
                let name_as_str = name.to_string();
                quote_spanned! {span=>
                    s.field(#name_as_str, &self.#name());
                }
            });
        impl_trait! {span=>
            impl Debug for #item {
                fn fmt(
                    &self,
                    f: &mut ::superbitty::__helpers::Formatter<'_>,
                ) -> ::superbitty::__helpers::FmtResult
                {
                    let mut s = f.debug_struct(#type_name_as_string);
                    #( #fields )*
                    s.finish()
                }
            }
        }
    }

    pub(super) fn clone(span: Span, item: &parse::BitfieldsStruct) -> TokenStream {
        impl_trait! {span=>
            impl Clone for #item {
                #[inline]
                fn clone(&self) -> Self {
                    // SAFETY: We're copying an existing field, so if it's valid
                    // so are we.
                    // We actually do a `Copy` and not `Clone`, but `Copy` is a
                    // supertrait of `BitFieldCompatible` so this is fine.
                    Self { __unsafe_raw: self.__unsafe_raw }
                }
            }
        }
    }

    pub(super) fn copy(span: Span, item: &parse::BitfieldsStruct) -> TokenStream {
        impl_trait! {span=>
            impl Copy for #item {}
        }
    }

    pub(super) fn default(span: Span, item: &parse::BitfieldsStruct) -> TokenStream {
        let field_types = item.fields.fields.iter().map(|field| &field.ty);
        let constructor_name = super::super::constructor_name(&item.constructor);
        impl_trait! {span=>
            impl Default for #item {
                #[inline]
                fn default() -> Self {
                    Self::#constructor_name(
                        #(
                            <#field_types as ::superbitty::__helpers::Default>::default(),
                        )*
                    )
                }
            }
        }
    }

    pub(super) fn partial_eq(span: Span, item: &parse::BitfieldsStruct) -> TokenStream {
        let field_names = item.fields.fields.iter().map(|field| &field.ident);
        impl_trait! {span=>
            impl PartialEq for #item {
                #[inline]
                fn eq(&self, _other: &Self) -> bool {
                    true #(
                        && ::superbitty::__helpers::PartialEq::eq(
                            &self.#field_names(), &_other.#field_names(),
                        )
                    )*
                }
            }
        }
    }

    pub(super) fn eq(span: Span, item: &parse::BitfieldsStruct) -> TokenStream {
        impl_trait! {span=>
            impl Eq for #item {}
        }
    }

    pub(super) fn partial_ord(span: Span, item: &parse::BitfieldsStruct) -> TokenStream {
        let field_names = item.fields.fields.iter().map(|field| &field.ident);
        impl_trait! {span=>
            impl PartialOrd for #item {
                #[inline]
                fn partial_cmp(&self, _other: &Self) -> ::superbitty::__helpers::PartialCmpResult {
                    #(
                        match ::superbitty::__helpers::PartialOrd::partial_cmp(
                            &self.#field_names(), &_other.#field_names(),
                        ) {
                            ::superbitty::__helpers::SOME_EQ => {}
                            cmp => return cmp,
                        }
                    )*
                    ::superbitty::__helpers::SOME_EQ
                }
            }
        }
    }

    pub(super) fn ord(span: Span, item: &parse::BitfieldsStruct) -> TokenStream {
        let field_names = item.fields.fields.iter().map(|field| &field.ident);
        impl_trait! {span=>
            impl Ord for #item {
                #[inline]
                fn cmp(&self, _other: &Self) -> ::superbitty::__helpers::Ordering {
                    ::superbitty::__helpers::Ordering::Equal #(
                        .then_with(|| {
                            ::superbitty::__helpers::Ord::cmp(
                                &self.#field_names(), &_other.#field_names(),
                            )
                        })
                    )*
                }
            }
        }
    }

    pub(super) fn hash(span: Span, item: &parse::BitfieldsStruct) -> TokenStream {
        let field_names = item.fields.fields.iter().map(|field| &field.ident);
        impl_trait! {span=>
            impl Hash for #item {
                #[inline]
                fn hash<H>(&self, _state: &mut H)
                where
                    H: ::superbitty::__helpers::Hasher,
                {
                    #(
                        ::superbitty::__helpers::Hash::hash(
                            &self.#field_names(),
                            _state,
                        );
                    )*
                }
            }
        }
    }
}

fn derive_fn(trait_: &str, span: Span) -> Option<DeriveFn> {
    let result = match trait_ {
        "Debug" => derive_fns::debug,
        "Clone" => derive_fns::clone,
        "Copy" => derive_fns::copy,
        "Default" => derive_fns::default,
        "PartialEq" => derive_fns::partial_eq,
        "Eq" => derive_fns::eq,
        "PartialOrd" => derive_fns::partial_ord,
        "Ord" => derive_fns::ord,
        "Hash" => derive_fns::hash,
        _ => return None,
    };
    Some(Box::new(move |item| result(span, item)))
}

type DeriveList = syn::punctuated::Punctuated<syn::Path, syn::Token![,]>;
/// We cannot use [`parse_args_with()`] for that because it also accepts braces
/// and brackets, while we want only parentheses.
///
/// [`parse_args_with()`]: syn::Attribute::parse_args_with
fn parse_derive_list(input: syn::parse::ParseStream) -> syn::Result<DeriveList> {
    let content;
    syn::parenthesized!(content in input);
    DeriveList::parse_terminated(&content)
}

pub(super) fn derives(
    struct_attrs: impl IntoIterator<Item = syn::Attribute>,
) -> syn::Result<(Vec<syn::Attribute>, Vec<DeriveFn>)> {
    let mut derives = Vec::new();
    let mut errors = SynErrors::default();
    let mut rest_attrs = Vec::new();
    for attr in struct_attrs {
        if attr.path.is_ident("derive") {
            if let Ok(derive) = parse_derive_list.parse2(attr.tokens.clone()) {
                derives.extend(derive.into_iter().filter_map(|derive| {
                    if let Some(derive_fn) =
                        derive.get_ident().and_then(|ident| derive_fn(&ident.to_string(), ident.span()))
                    {
                        Some(derive_fn)
                    } else {
                        errors.push(syn::Error::new_spanned(
                            &derive,
                            "unknown derive in `bitfields!`. Refer to the documentation for the list of supported derives",
                        ));
                        None
                    }
                }));
                continue;
            }
            // Let the compiler deal with malformed derive; it will emit better error messages.
        }
        rest_attrs.push(attr);
    }
    errors.into_result()?;
    Ok((rest_attrs, derives))
}
