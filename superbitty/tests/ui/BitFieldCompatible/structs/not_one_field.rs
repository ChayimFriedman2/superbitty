use superbitty::BitFieldCompatible;

#[derive(BitFieldCompatible)]
#[bit_field(size = 0)]
struct Unit;

#[derive(BitFieldCompatible)]
#[bit_field(size = 0)]
struct ZeroTuple();

#[derive(BitFieldCompatible)]
#[bit_field(size = 0)]
struct ZeroRecord {}

#[derive(BitFieldCompatible)]
#[bit_field(size = 0)]
struct MoreThanOneTuple(u32, u32);

#[derive(BitFieldCompatible)]
#[bit_field(size = 0)]
struct MoreThanOneRecord {
    a: u32,
    b: u32,
}

fn main() {}
