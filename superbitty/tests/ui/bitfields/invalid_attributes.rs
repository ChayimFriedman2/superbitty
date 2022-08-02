use superbitty::bitfields;

bitfields! {
    #[foo]
    /// Hello
    #[bar = 123]
    #[doc = "this is a doc comment"]
    #[baz(abc)]
    #[doc = concat!("conc", "atnated")]
    struct Foo : u8 {}
}

fn main() {}
