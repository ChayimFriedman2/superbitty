mod enum_bitfield;

use proc_macro2::TokenStream;

pub(crate) fn bit_field_compatible(item: TokenStream) -> syn::Result<TokenStream> {
    let item = syn::parse2::<syn::DeriveInput>(item)?;
    match &item.data {
        syn::Data::Enum(_) => enum_bitfield::enum_bitfield(item),
        _ => Err(syn::Error::new_spanned(&item, "only enums can `#[derive(BitFieldCompatible)]`")),
    }
}
