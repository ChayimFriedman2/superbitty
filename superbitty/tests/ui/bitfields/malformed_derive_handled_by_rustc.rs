// We make sure here that we don't handle malformed `#[derive()]`,
// leaving it to rustc in order to generate better error messages.

use superbitty::bitfields;

bitfields! {
    #[derive]
    #[derive = ""]
    #[derive[Debug]]
    struct WithMalformedDerive : u8 {}
}

fn main() {}
