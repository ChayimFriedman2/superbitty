use superbitty::BitFieldCompatible;

#[derive(BitFieldCompatible)]
enum WithNegativeDiscriminant {
    A = -1,
}

fn main() {}
