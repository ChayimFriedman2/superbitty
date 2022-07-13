use superbitty::BitFieldCompatible;

#[derive(BitFieldCompatible)]
struct Foo;

#[derive(BitFieldCompatible)]
union Bar {}

fn main() {}
