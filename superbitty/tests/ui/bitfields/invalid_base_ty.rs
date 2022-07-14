use superbitty::bitfields;

bitfields! {
    struct NegativeBaseTy : isize {}
}

bitfields! {
    struct OtherBaseTy : Option<String> {}
}

fn main() {}
