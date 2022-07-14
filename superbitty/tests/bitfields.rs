use std::cmp::Ordering;
use std::hash::Hash;

use superbitty::{bitfields, BitFieldCompatible};

#[derive(BitFieldCompatible, Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum EnumA {
    #[default]
    A,
    B,
}
#[derive(BitFieldCompatible, Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EnumB {
    A,
    B,
    #[default]
    C,
}

bitfields! {
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Bitfields : u8 {
        enum_a: EnumA,
        pub enum_b: EnumB,
    }
    construct = fn new();
}

#[test]
fn new_and_set() {
    let mut instance = Bitfields::new(EnumA::B, EnumB::A);
    assert_eq!(instance.enum_a(), EnumA::B);
    assert_eq!(instance.enum_b(), EnumB::A);

    instance.set_enum_b(EnumB::C);
    assert_eq!(instance.enum_a(), EnumA::B);
    assert_eq!(instance.enum_b(), EnumB::C);

    instance.set_enum_a(EnumA::A);
    assert_eq!(instance.enum_a(), EnumA::A);
    assert_eq!(instance.enum_b(), EnumB::C);
}

#[test]
fn default() {
    let instance = Bitfields::default();
    assert_eq!(instance.enum_a(), EnumA::default());
    assert_eq!(instance.enum_b(), EnumB::default());
}

#[test]
fn debug() {
    let debug_str = format!("{:?}", Bitfields::new(EnumA::B, EnumB::B));
    assert_eq!(debug_str, "Bitfields { enum_a: B, enum_b: B }");
}

#[test]
fn clone_copy() {
    let original = Bitfields::new(EnumA::A, EnumB::C);
    let clone = original.clone();
    assert_eq!(original.enum_a(), clone.enum_a());
    assert_eq!(original.enum_b(), clone.enum_b());
    let copy = original;
    assert_eq!(original.enum_a(), copy.enum_a());
    assert_eq!(original.enum_b(), copy.enum_b());
}

#[test]
fn equality() {
    fn assert_eq<T: Eq>() {}
    assert_eq::<Bitfields>();

    for a1 in [EnumA::A, EnumA::B] {
        for b1 in [EnumB::A, EnumB::B, EnumB::C] {
            for a2 in [EnumA::A, EnumA::B] {
                for b2 in [EnumB::A, EnumB::B, EnumB::C] {
                    let one = Bitfields::new(a1, b1);
                    let two = Bitfields::new(a2, b2);
                    assert_eq!(one == two, a1 == a2 && b1 == b2);
                }
            }
        }
    }
}

#[test]
fn comparison() {
    for a1 in [EnumA::A, EnumA::B] {
        for b1 in [EnumB::A, EnumB::B, EnumB::C] {
            for a2 in [EnumA::A, EnumA::B] {
                for b2 in [EnumB::A, EnumB::B, EnumB::C] {
                    let one = Bitfields::new(a1, b1);
                    let two = Bitfields::new(a2, b2);
                    assert_eq!(one.cmp(&two), a1.cmp(&a2).then(b1.cmp(&b2)));
                    assert_eq!(one.partial_cmp(&two), Some(a1.cmp(&a2).then(b1.cmp(&b2))));
                }
            }
        }
    }
}

#[test]
fn hash() {
    for a in [EnumA::A, EnumA::B] {
        for b in [EnumB::A, EnumB::B, EnumB::C] {
            let bitfields = Bitfields::new(a, b);

            #[derive(Debug, Default, PartialEq, Eq)]
            struct VecHasher(Vec<u8>);
            impl std::hash::Hasher for VecHasher {
                fn finish(&self) -> u64 {
                    0
                }
                fn write(&mut self, bytes: &[u8]) {
                    self.0.extend_from_slice(bytes);
                }
            }

            let mut bitfields_hasher = VecHasher::default();
            bitfields.hash(&mut bitfields_hasher);
            let mut individual_hasher = VecHasher::default();
            a.hash(&mut individual_hasher);
            b.hash(&mut individual_hasher);

            assert_eq!(bitfields_hasher, individual_hasher);
        }
    }
}

bitfields! {
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub(crate) struct Empty : u32 {}
}

#[test]
fn empty() {
    _ = Empty::new().clone();
    _ = Empty::default();
    assert_eq!(Empty::default(), Empty::new());
    assert!(!(Empty::default() != Empty::new()));
    assert!(Empty::default() >= Empty::new());
    assert_eq!(Empty::new().partial_cmp(&Empty::new()), Some(Ordering::Equal));
}
