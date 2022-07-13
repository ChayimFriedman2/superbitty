use superbitty::BitFieldCompatible;

#[derive(BitFieldCompatible)]
enum WithPayload {
    A(()),
    B,
    C { value: i32 },
}

fn main() {}
