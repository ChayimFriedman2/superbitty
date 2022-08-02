mod derives;
mod parse;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;

use self::derives::derives;
use crate::utils::{is_unsigned_int_primitive, SynErrors};

pub(crate) fn bitfields_impl(item: TokenStream) -> syn::Result<TokenStream> {
    let mut item = syn::parse2::<parse::BitfieldsStruct>(item)?;

    disallow_invalid_attributes(&item.attrs)?;

    let (struct_attrs, derives) = derives(item.attrs.drain(..))?;

    verify_base_ty(&item.base_ty)?;

    let assert_bitfields_compatible = assert_bitfields_compatible(&item.fields);

    let derives = TokenStream::from_iter(derives.into_iter().map(|derive| derive(&item)));

    let base_ty = &item.base_ty;

    let (bitfields, after_last_bitfield_offset) = bitfields(item.fields);
    let assert_bitfields_size = quote! {
        const _: () = assert!(
            #after_last_bitfield_offset <= #base_ty::BITS,
            "bitfield size is too big - choose another base type",
        );
    };

    let new_method = new_method(
        &bitfields,
        constructor_name(&item.constructor),
        constructor_vis(&item.constructor, &item.vis),
        &item.base_ty,
    );
    let bitfields_accessors =
        bitfields.iter().map(|bitfield| bitfield_accessors(bitfield, &item.base_ty));

    let struct_vis = &item.vis;
    let struct_kw = &item.struct_token;
    let struct_name = &item.ident;
    let generics = &item.generics;
    let (impl_generics, type_generics, where_clause) = item.generics.split_for_impl();
    let result = quote! {
        #(#struct_attrs)*
        #struct_vis #struct_kw #struct_name #generics(
            // Invariant: Always holds valid instances of the bit fields.
            ::superbitty::Raw<#base_ty>,
        )
            #where_clause;

        #assert_bitfields_compatible
        #assert_bitfields_size

        #derives

        impl #impl_generics #struct_name #type_generics
        #where_clause
        {
            #new_method

            #(#bitfields_accessors)*
        }
    };
    Ok(result)
}

/// Proc macros can cause unsoundness - they can replace the inner representation with some evil
/// type that implements the bitwise operators incorrectly, causing us to create invalid instances
/// of types such as enums. Because of that we disallow them here.
fn disallow_invalid_attributes(attrs: &[syn::Attribute]) -> syn::Result<()> {
    let mut errors = SynErrors::default();
    for attr in attrs {
        if !attr.path.is_ident("doc") && !attr.path.is_ident("derive") {
            errors.push(syn::Error::new_spanned(
                attr,
                "only doc comments and `#[derive(…)]` are allowed with `bitfields!`",
            ));
        }
    }
    errors.into_result()
}

struct Bitfield {
    parse: parse::BitfieldsStructField,
    bit_offset: TokenStream,
    type_shift: TokenStream,
    bits_mask: TokenStream,
    is_last: bool,
}

fn bitfields(fields: parse::BitfieldsStructFields) -> (Vec<Bitfield>, TokenStream) {
    let mut prev_offset = quote!(0);
    let mut result = Vec::with_capacity(fields.fields.len());
    let fields_count = fields.fields.len();
    for (index, field) in fields.fields.into_iter().enumerate() {
        let bit_offset = quote! { ( #prev_offset ) };
        let field_ty = &field.ty;
        let bits_mask = quote! {
            <#field_ty as ::superbitty::BitFieldCompatible>::BITS_MASK
        };
        prev_offset.extend(std::iter::once(quote! {
            + <#field_ty as ::superbitty::BitFieldCompatible>::BITS_LEN
        }));
        let type_shift = quote! { <#field_ty as ::superbitty::BitFieldCompatible>::SHIFT };
        let is_last = index == fields_count - 1;
        result.push(Bitfield { parse: field, bit_offset, type_shift, bits_mask, is_last })
    }
    (result, quote! { ( #prev_offset) })
}

fn constructor_name(constructor: &parse::Constructor) -> Ident {
    match constructor {
        Some((_, constructor_name)) => constructor_name.clone(),
        None => syn::Ident::new("new", Span::call_site()),
    }
}
fn constructor_vis<'a>(
    constructor: &'a parse::Constructor,
    item_vis: &'a syn::Visibility,
) -> &'a syn::Visibility {
    match constructor {
        Some((constructor_vis, _)) => constructor_vis,
        None => item_vis,
    }
}

fn new_method(
    bitfields: &[Bitfield],
    method_name: syn::Ident,
    method_vis: &syn::Visibility,
    base_ty: &syn::Type,
) -> TokenStream {
    let args = bitfields.iter().map(
        |Bitfield { parse: parse::BitfieldsStructField { ident: name, ty, .. }, .. }| quote!(#name : #ty),
    );
    let fields_calculation = bitfields.iter().map(
        |Bitfield {
             parse: parse::BitfieldsStructField { ident: name, ty, .. },
             bit_offset,
             type_shift,
             ..
         }| {
            quote! {
                ((<#ty as ::superbitty::BitFieldCompatible>::into_raw(#name) >> #type_shift)
                    << #bit_offset)
            }
        },
    );
    quote! {
        #method_vis fn #method_name( #( #args, )* ) -> Self {
            Self(
                // SAFETY: We're combining valid values from `into_raw()` that by
                // `BitFieldCompatible`'s preconditions guaranteed to return valid
                // discriminants.
                unsafe { ::superbitty::Raw::new(( 0 #( | #fields_calculation )* ) as #base_ty) },
            )
        }
    }
}

fn bitfield_accessors(
    Bitfield {
        parse: parse::BitfieldsStructField { attrs, vis, ident: field_name, ty },
        bit_offset,
        type_shift,
        bits_mask,
        is_last,
    }: &Bitfield,
    base_ty: &syn::Type,
) -> TokenStream {
    let setter_name = format_ident!("set_{field_name}");
    let mut getter_stripped_field =
        quote! { ((::superbitty::Raw::raw(self.0) as u128) >> #bit_offset) };
    if !is_last {
        getter_stripped_field = quote! { (#getter_stripped_field & #bits_mask) }
    }
    quote_spanned! {field_name.span()=>
        #(#attrs)* // We put the attributes on the getter mainly for documentation comments.
        #[inline]
        #vis fn #field_name(&self) -> #ty {
            // SAFETY: Since `self.0` always holds valid instances, and all bitfields are
            // `Copy`, we can convert the bitfield to its enum soundly.
            unsafe {
                <#ty as ::superbitty::BitFieldCompatible>::from_raw(
                    #getter_stripped_field << #type_shift,
                )
            }
        }

        #[inline]
        #vis fn #setter_name(&mut self, value: #ty) {
            let raw_without_field = (::superbitty::Raw::raw(self.0) as u128) & !(#bits_mask << #bit_offset);
            let field_in_place =
                (<#ty as ::superbitty::BitFieldCompatible>::into_raw(value) >> #type_shift)
                    << #bit_offset;
            // SAFETY: We only trim irrelevant bits that by `BitFieldCompatible`'s precondition
            // should be safe.
            *unsafe { ::superbitty::Raw::get_mut(&mut self.0) } =
                (raw_without_field | field_in_place) as #base_ty;
        }
    }
}

fn verify_base_ty(base_ty: &syn::Type) -> syn::Result<()> {
    // We can leave that out because `Raw` will validate that but this gives better error message.
    if is_unsigned_int_primitive(base_ty) {
        return Ok(());
    }

    return Err(syn::Error::new_spanned(
        base_ty,
        "unsupported base type for `bitfields!`: only primitive unsigned types are supported",
    ));
}

fn assert_bitfields_compatible(fields: &parse::BitfieldsStructFields) -> TokenStream {
    let field_asserts = fields.fields.iter().map(|parse::BitfieldsStructField { ty, .. }| {
        quote_spanned! {ty.span()=>
            ::superbitty::__helpers::assert_bitfield_compatible::<#ty>();
        }
    });
    quote! {
        const _: () = {
            #(#field_asserts)*
        };
    }
}
