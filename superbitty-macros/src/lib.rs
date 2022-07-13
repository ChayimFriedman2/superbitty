mod bit_field_compatible;
mod bitfields;
mod utils;

use proc_macro::TokenStream;

#[proc_macro_derive(BitFieldCompatible)]
pub fn bit_field_compatible(item: TokenStream) -> TokenStream {
    bit_field_compatible::bit_field_compatible(item.into())
        .unwrap_or_else(|err| err.into_compile_error())
        .into()
}

#[proc_macro]
pub fn bitfields(item: TokenStream) -> TokenStream {
    bitfields::bitfields_impl(item.into()).unwrap_or_else(|err| err.into_compile_error()).into()
}
