use superbitty::BitFieldCompatible;

#[derive(BitFieldCompatible)]
#[bit_field(size = 0)]
struct InvalidRawTypeTuple(());

#[derive(BitFieldCompatible)]
#[bit_field(size = 0)]
struct InvalidRawTypeRecord {
    raw: i32,
}

fn main() {}
