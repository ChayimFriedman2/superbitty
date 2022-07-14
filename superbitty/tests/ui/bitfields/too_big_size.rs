use superbitty::{bitfields, BitFieldCompatible};

#[derive(BitFieldCompatible, Clone, Copy)]
enum One {
    A,
    B,
}

#[derive(BitFieldCompatible, Clone, Copy)]
enum Two {
    A = 0b01,
    B = 0b10,
}

#[derive(BitFieldCompatible, Clone, Copy)]
enum Three {
    A = 0b001,
    B = 0b100,
}

bitfields! {
    struct TooBigSize : u8 {
        a: One,
        b: Two,
        c: Three,
        d: Two,
        e: One,
        // 1 + 2 + 3 + 2 + 1 = 9
    }
}

fn main() {}
