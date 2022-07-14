use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

use crate::utils::SynErrors;

pub(crate) fn enum_bitfield(item: syn::DeriveInput) -> syn::Result<TokenStream> {
    let enum_ = match &item.data {
        syn::Data::Enum(enum_) => enum_,
        _ => unreachable!(),
    };

    let discriminants_mask = discriminants_mask(&enum_.variants)?;
    let shift = if discriminants_mask == 0 {
        0 // Using 128 will panic in debug mode.
    } else {
        discriminants_mask.trailing_zeros()
    };
    let bits_len = if discriminants_mask == 0 {
        0 // Both leading and trailing zero are 128, subtracting will overflow.
    } else {
        u128::BITS - discriminants_mask.trailing_zeros() - discriminants_mask.leading_zeros()
    };
    let from_raw = from_raw(&item.ident, enum_.variants.iter().map(|variant| &variant.ident));
    let type_name = &item.ident;
    let (impl_generics, type_generics, where_clause) = item.generics.split_for_impl();
    let result = quote! {
        // SAFETY: The trailing/leading zeros are ensured to be correct - that is, not relevant,
        // and stripping them is nothing. Thus, this calling `from_raw(MASK(into_raw(v)))` is the
        // same as calling converting the enum to int and back, and since it carries no payload
        // this is fine.
        unsafe impl #impl_generics ::superbitty::BitFieldCompatible
            for #type_name #type_generics
        #where_clause
        {
            const SHIFT: u32 = #shift;
            const BITS_LEN: u32 = #bits_len;

            #[inline]
            fn into_raw(self) -> u128 { self as u128 }
            #from_raw
        }
    };
    Ok(result)
}

pub(crate) fn from_raw<'a>(
    enum_name: &syn::Ident,
    variants: impl Iterator<Item = &'a syn::Ident> + Clone,
) -> TokenStream {
    let variant_consts = variants.clone().map(|name| {
        quote! {
            // const blocks, please.
            const #name: u128 = #enum_name::#name as u128;
        }
    });
    let variants_match_arms = variants.map(|name| {
        quote! {
            #name => Self::#name,
        }
    });
    quote! {
        #[inline]
        #[allow(non_upper_case_globals)]
        unsafe fn from_raw(__v: u128) -> Self {
            #(#variant_consts)*
            match __v {
                #(#variants_match_arms)*
                // SAFETY: We're guaranteed by precondition that `__v` is a valid variant.
                _ => unsafe { ::superbitty::__helpers::unreachable_unchecked() }
            }
        }
    }
}

pub(crate) fn discriminants_mask(
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
) -> syn::Result<u128> {
    // We cannot start with zero as it will always be less than any other value.
    // Thus, we only default to zero at the end.
    let mut discriminants_mask = 0;
    let mut prev_discriminant = None;
    let mut errors = SynErrors::default();
    for variant in variants {
        if !matches!(variant.fields, syn::Fields::Unit) {
            errors.push(syn::Error::new_spanned(
                &variant.fields,
                "cannot have payload with `BitFieldCompatible`",
            ));
        }

        let discriminant = match &variant.discriminant {
            Some((_, discriminant)) => match discriminant_value_or_err(discriminant) {
                Ok(discriminant) => discriminant,
                Err(err) => {
                    errors.push(err);
                    0
                }
            },
            None => match prev_discriminant {
                Some(prev_value) => prev_value + 1,
                None => 0,
            },
        };

        discriminants_mask |= discriminant;
        prev_discriminant = Some(discriminant);
    }
    errors.into_result()?;
    Ok(discriminants_mask)
}

pub(crate) fn discriminant_value_or_err(discriminant: &syn::Expr) -> syn::Result<u128> {
    let span = discriminant.span();
    let (discriminant, is_negative) = discriminant_value(discriminant)?;
    if is_negative {
        return Err(syn::Error::new(
            span,
            "negative discriminants are not supported with `BitFieldCompatible`",
        ));
    }
    let discriminant = discriminant.ok_or_else(|| {
        syn::Error::new(
            span,
            "complex expressions in the discriminant are not supported with `BitFieldCompatible` (only integers)",
        )
    })?;
    Ok(discriminant)
}

pub(crate) fn discriminant_value(discriminant: &syn::Expr) -> syn::Result<(Option<u128>, bool)> {
    fn numeric_value(expr: &syn::Expr) -> syn::Result<Option<(u128, bool)>> {
        if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Int(lit), .. }) = expr {
            let mut digits = lit.base10_digits();
            let mut is_negative = false;
            if digits.bytes().next() == Some(b'-') {
                is_negative = true;
                digits = &digits[1..];
            }
            let result = digits.parse().map_err(|err| syn::Error::new(lit.span(), err))?;
            Ok(Some((result, is_negative)))
        } else {
            Ok(None)
        }
    }

    let (discriminant, is_negative) = match numeric_value(discriminant)? {
        Some((discriminant, is_negative)) => (Some(discriminant), is_negative),
        _ => {
            let is_negative = match discriminant {
                syn::Expr::Unary(syn::ExprUnary {
                    op: syn::UnOp::Neg(_), expr: operand, ..
                }) => match numeric_value(operand)? {
                    Some((_, operand_is_negative)) => !operand_is_negative,
                    None => false,
                },
                _ => false,
            };
            (None, is_negative)
        }
    };
    Ok((discriminant, is_negative))
}
