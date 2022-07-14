use superbitty::bitfields;

bitfields! {
    #[derive(Foo, serde::Serialize)]
    #[derive(serde::Deserialize, Debug)]
    #[derive(BitFieldCompatible)]
    struct UnknownDerive : u8 {}
}

fn main() {}
