mod enum_bitfield;
mod struct_bitfield;

use proc_macro2::TokenStream;

pub(crate) fn bit_field_compatible(item: TokenStream) -> syn::Result<TokenStream> {
    let item = syn::parse2::<syn::DeriveInput>(item)?;
    match &item.data {
        syn::Data::Enum(_) => enum_bitfield::enum_bitfield(item),
        syn::Data::Struct(_) => struct_bitfield::struct_bitfield(item),
        syn::Data::Union(_) => {
            Err(syn::Error::new_spanned(&item, "unions cannot `#[derive(BitFieldCompatible)]`"))
        }
    }
}
