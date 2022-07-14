use superbitty::BitFieldCompatible;

#[derive(BitFieldCompatible)]
#[bit_field(size = 0, foo = 123, bar = "456")]
struct InvalidOptions(u32);

fn main() {}
