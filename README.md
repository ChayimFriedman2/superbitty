# Superbitty

A bitfields crate.

```rust
use superbitty::{bitfields, BitFieldCompatible};

#[derive(BitFieldCompatible, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Enum {
    A,
    B,
    C,
    D,
}

#[derive(BitFieldCompatible, Clone, Copy)]
#[bit_field(size = 6)]
pub struct Rest(pub u8);

bitfields! {
    pub struct Bitfields : u8 {
        pub e: Enum,
        pub r: Rest,
    }
}

fn main() {
    let mut instance = Bitfields::new(Enum::B, Rest(0b010));
    assert_eq!(instance.e(), Enum::B);
    instance.set_r(Rest(0b101));
    assert_eq!(instance.r().0, 0b101);
}
```
