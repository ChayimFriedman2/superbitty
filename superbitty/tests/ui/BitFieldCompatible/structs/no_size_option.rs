use superbitty::BitFieldCompatible;

#[derive(BitFieldCompatible)]
#[bit_field()]
struct InvalidOptions(u32);

fn main() {}
