#![no_std]

/// A bitfield struct.
///
/// All fields must implement [`BitFieldCompatible`] and their ranges must be non-overlapping
/// and big enough to hold the bitfields.
///
/// ```
/// # use superbitty::{bitfields, BitFieldCompatible};
/// #[derive(BitFieldCompatible, Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// enum EnumA { #[default] A, B }
/// #[derive(BitFieldCompatible, Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// enum EnumB { #[default] A, B, C }
///
/// bitfields! {
///     #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
///     pub struct Bitfields : u8 {
///         enum_a: EnumA, // from bit 0 to bit 1 (exclusive)
///         pub enum_b: EnumB, // from bit 1 to bit 3
///     }
/// }
///
/// let mut instance = Bitfields::new(EnumA::B, EnumB::A);
/// assert_eq!(instance.enum_a(), EnumA::B);
/// assert_eq!(instance.enum_b(), EnumB::A);
///
/// instance.set_enum_b(EnumB::C);
/// assert_eq!(instance.enum_a(), EnumA::B);
/// assert_eq!(instance.enum_b(), EnumB::C);
/// ```
///
/// The traits derived in the example - [`Debug`], [`Clone`], [`Copy`], [`Default`],
/// [`PartialEq`], [`Eq`], [`PartialOrd`], [`Ord`] and [`Hash`] - are the only you
/// can `#[derive()]`. All others need to be manually implemented.
///
/// By default, the constructor function will be called `new` and will have the same
/// visibility as the struct. You can control that by adding
/// `construct = <visibility> fn <name>();` after the struct, like the following:
/// ```ignore
/// bitfields! {
///     // ...
///     construct = pub fn new();
/// }
/// ```
///
/// [`BitFieldCompatible`]: superbitty::BitFieldCompatible
/// [`Debug`]: core::fmt::Debug
/// [`Hash`]: core::hash::Hash
pub use superbitty_macros::bitfields;
/// An enum that can be used as a bitfield.
///
/// It must be [`Copy`], carry no payload, have no negative discriminants and
/// inhabitant (with at least one variant).
///
/// ```
/// # use superbitty::BitFieldCompatible;
/// #[derive(BitFieldCompatible, Clone, Copy)]
/// enum BitFieldCompatibleEnum { A, B, C }
/// ```
pub use superbitty_macros::BitFieldCompatible;

/// An enum or struct that can be used as a bitfield. This is usually [derived] for
/// enums and implemented by the [`bit_field_compatible`] attribute for structs.
///
/// # Safety
///
///   - An enum implementing this trait must not carry payload in any of its variants.
///   - The [`SHIFT`] and [`BITS_LEN`] values must be correct.
///   - The type must be [inhabited].
///   - [`into_raw()`] must return a valid value (something you can feed back
///     to [`from_raw()`]).
///
/// [derived]: macro@BitFieldCompatible
/// [`SHIFT`]: BitFieldCompatible::SHIFT
/// [`BITS_LEN`]: BitFieldCompatible::BITS_LEN
/// [inhabited]: https://rustc-dev-guide.rust-lang.org/appendix/glossary.html?highlight=uninhabited#glossary
/// [`into_raw()`]: BitFieldCompatible::into_raw
/// [`from_raw()`]: BitFieldCompatible::from_raw
pub unsafe trait BitFieldCompatible: Copy {
    /// The number we need to left-shift with to reach a valid value from a compressed
    /// value which has all trailing zeros trimmed.
    ///
    /// For example, in an enum with the discriminants `0b01000`, `0b10000` and `0b11000`,
    /// the value of [`BITS_LEN`] will be 2 and the value of `SHIFT` will be 3, because
    /// we need to left-shift three times to go from `0b01`, `0b10` and `0b11` to the
    /// corresponding variants.
    ///
    /// [`BITS_LEN`]: BitFieldCompatible::BITS_LEN
    const SHIFT: u32;

    /// The minimum number of bits required to represent any bit pattern that is valid
    /// for this type.
    ///
    /// Note that enums with only one variant may still have this value greater than zero
    /// if the discriminant of this variant is not zero, for example if the discriminant is
    /// `0b11` it needs two bits to be stored.
    ///
    /// You can play with this value as long as you keep this rule of thumb: if we call
    /// [`into_raw()`] and truncate the bits left after applying this value (that is,
    /// perform the operation `into_raw(v) & (((1 << BITS_LEN) - 1) << SHIFT)`),
    /// it is okay to call [`from_raw()`] with the resulted value and this will give back
    /// the original value.
    ///
    /// [`into_raw()`]: BitFieldCompatible::into_raw
    /// [`from_raw()`]: BitFieldCompatible::from_raw
    const BITS_LEN: u32;

    /// This value must not be overridden! Doing so can cause UB.
    #[doc(hidden)]
    const BITS_MASK: u128 = (1 << Self::BITS_LEN) - 1;

    /// Retrieves the raw int representation of a value.
    fn into_raw(self) -> u128;

    /// Converts a raw int representation to this type.
    ///
    /// # Safety
    ///
    /// `v` must be a valid value for this type.
    unsafe fn from_raw(v: u128) -> Self;
}

#[doc(hidden)]
pub mod __helpers {
    pub use core::clone::Clone;
    pub use core::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
    pub use core::default::Default;
    pub use core::fmt::{Debug, Formatter, Result as FmtResult};
    pub use core::hash::{Hash, Hasher};
    pub use core::hint::unreachable_unchecked;
    pub use core::marker::Copy;
    pub type PartialCmpResult = Option<Ordering>;
    pub const SOME_EQ: PartialCmpResult = Some(Ordering::Equal);

    pub const fn assert_bitfield_compatible<T: super::BitFieldCompatible>() {}
}
