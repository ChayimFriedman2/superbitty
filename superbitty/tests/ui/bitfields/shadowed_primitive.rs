use superbitty::bitfields;

#[allow(non_camel_case_types)]
struct u32;

bitfields! {
    struct Foo : u32 {}
}

fn main() {}
