use superbitty::{bitfields, BitFieldCompatible};

enum FieldA {}

#[derive(BitFieldCompatible, Clone, Copy)]
enum CompatibleField {
    Variant,
}

struct FieldB;

bitfields! {
    struct NonCompatibleFields : u8 {
        a: FieldA,
        compatible: CompatibleField,
        b: FieldB,
    }
}

fn main() {}
