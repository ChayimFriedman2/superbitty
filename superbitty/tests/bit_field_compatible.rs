use superbitty::BitFieldCompatible;

fn assert_values<T: BitFieldCompatible>(shift: u32, bits_len: u32, bits_mask: u128) {
    assert_eq!(T::SHIFT, shift);
    assert_eq!(T::BITS_LEN, bits_len);
    assert_eq!(T::BITS_MASK, bits_mask);
}

#[derive(BitFieldCompatible, Clone, Copy)]
enum OneZeroVariantEnum {
    Variant,
}

#[derive(BitFieldCompatible, Clone, Copy)]
enum OneNonZeroVariantEnum {
    Variant = 0b10,
}

#[derive(BitFieldCompatible, Clone, Copy)]
enum Scattered {
    A = 0,
    B = 0b0000000000000100,
    C = 0b0010000000000000,
}

#[derive(BitFieldCompatible, Clone, Copy)]
enum Dense {
    A = 0,
    B = 0b0000000100,
    C = 0b0000001000,
    D = 0b0000010000,
    E = 0b0000100000,
    F = 0b0001000000,
    G = 0b0010000000,
}

#[derive(BitFieldCompatible, Clone, Copy)]
enum BigOne {
    A = 0b1111111,
}

#[test]
fn test() {
    assert_values::<OneZeroVariantEnum>(0, 0, 0);
    assert_values::<OneNonZeroVariantEnum>(1, 1, 0b1);
    assert_values::<Scattered>(2, 12, 0b00111111111111);
    assert_values::<Dense>(2, 6, 0b00111111);
    assert_values::<BigOne>(0, 7, 0b1111111);
}
