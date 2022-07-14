use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};

use crate::utils::{is_unsigned_int_primitive, SynErrors};

pub(crate) fn struct_bitfield(item: syn::DeriveInput) -> syn::Result<TokenStream> {
    let struct_ = match &item.data {
        syn::Data::Struct(struct_) => struct_,
        _ => unreachable!(),
    };

    let (size, offset) = bitfield_properties(&item.attrs)?;
    let raw_field = raw_field(&struct_.fields)?;

    let type_name = &item.ident;
    let (impl_generics, type_generics, where_clause) = item.generics.split_for_impl();
    let result = quote! {
        // SAFETY: Nothing unsafe is done in `from_raw()` or `into_raw()`, calling
        // whoever is always fine (but might cut information if the value is used without
        // care).
        unsafe impl #impl_generics ::superbitty::BitFieldCompatible
            for #type_name #type_generics
        #where_clause
        {
            const SHIFT: u32 = #offset;
            const BITS_LEN: u32 = #size;

            #[inline]
            fn into_raw(self) -> u128 { self.#raw_field as u128 }
            #[inline]
            unsafe fn from_raw(v: u128) -> Self {
                Self { #raw_field: v as _ }
            }
        }
    };
    Ok(result)
}

fn raw_field(fields: &syn::Fields) -> syn::Result<TokenStream> {
    let zero_fields = || {
        syn::Error::new(
            Span::call_site(),
            "one field is required for `#[derive(BitFieldCompatible)]`",
        )
    };
    let too_many_fields = || {
        syn::Error::new(
            Span::call_site(),
            "only one field is allowed for `#[derive(BitFieldCompatible)]`",
        )
    };

    let (field_name, field_ty) = match fields {
        syn::Fields::Named(fields) => {
            if fields.named.len() == 0 {
                return Err(zero_fields());
            }
            if fields.named.len() > 1 {
                return Err(too_many_fields());
            }
            let field_name = fields.named[0].ident.to_token_stream();
            (field_name, &fields.named[0].ty)
        }
        syn::Fields::Unnamed(fields) => {
            if fields.unnamed.len() == 0 {
                return Err(zero_fields());
            }
            if fields.unnamed.len() > 1 {
                return Err(too_many_fields());
            }
            (quote!(0), &fields.unnamed[0].ty)
        }
        syn::Fields::Unit => return Err(zero_fields()),
    };

    if !is_unsigned_int_primitive(field_ty) {
        return Err(syn::Error::new_spanned(
            field_ty,
            "only primitive unsigned integers are supported as raw type \
                with `#[derive(BitFieldCompatible)]`",
        ));
    }

    Ok(field_name)
}

fn bitfield_properties(attrs: &[syn::Attribute]) -> syn::Result<(u32, u32)> {
    let attr = attrs.iter().find(|attr| attr.path.is_ident("bit_field")).ok_or_else(|| {
        syn::Error::new(
            Span::call_site(),
            "there must be an attribute `#[bit_field(size = …, [offset = …])]` \
                    to denote the bitfield range",
        )
    })?;

    let options = attr.parse_args_with(
        syn::punctuated::Punctuated::<syn::MetaNameValue, syn::Token![,]>::parse_terminated,
    )?;

    let mut size = None;
    let mut offset = None;
    let mut errors = SynErrors::default();
    for name_value in options {
        let value = match &name_value.lit {
            syn::Lit::Int(value) => {
                // FIXME: Give a better error message in case of negative number.
                value.base10_parse()?
            }
            _ => {
                errors.push(syn::Error::new_spanned(
                    &name_value,
                    "malformed option, expected `name = <int_value>`",
                ));
                continue;
            }
        };
        let is_duplicate = match name_value.path.get_ident().map(Ident::to_string).as_deref() {
            Some("size") => std::mem::replace(&mut size, Some(value)).is_some(),
            Some("offset") => std::mem::replace(&mut offset, Some(value)).is_some(),
            _ => {
                errors.push(syn::Error::new_spanned(
                    &name_value,
                    "unknown option, expected `size` or `offset`",
                ));
                continue;
            }
        };
        if is_duplicate {
            errors.push(syn::Error::new_spanned(&name_value, "duplicate option"));
            continue;
        }
    }

    let size = match size {
        Some(size) => size,
        None => {
            errors.push(syn::Error::new_spanned(&attr, "the `size` option is mandatory"));
            0
        }
    };

    errors.into_result()?;

    Ok((size, offset.unwrap_or(0)))
}
