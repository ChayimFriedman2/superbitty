use superbitty::BitFieldCompatible;

#[derive(BitFieldCompatible)]
#[bit_field(offset = 0, size = 1, size = 2, offset = 3)]
struct InvalidOptions(u32);

fn main() {}
