use superbitty::BitFieldCompatible;

const fn foo() -> isize {
    0
}

#[derive(BitFieldCompatible)]
enum WithPayload {
    A = foo(),
    B,
}

fn main() {}
