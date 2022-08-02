//! A bitfields crate.
//!
//! ```
//! use superbitty::{bitfields, BitFieldCompatible};
//!
//! #[derive(BitFieldCompatible, Debug, Clone, Copy, PartialEq, Eq)]
//! pub enum Enum {
//!     A,
//!     B,
//!     C,
//!     D,
//! }
//!
//! #[derive(Clone, Copy)]
//! pub struct Rest(pub u8);
//!
//! // SAFETY: We only set this via `Bitfields`, and thus the values are guaranteed
//! // to stay in range.
//! unsafe impl BitFieldCompatible for Rest {
//!     const SHIFT: u32 = 0;
//!     const BITS_LEN: u32 = 6;
//!     fn into_raw(self) -> u128 { self.0 as u128 }
//!     unsafe fn from_raw(v: u128) -> Self { Self(v as u8) }
//! }
//!
//! bitfields! {
//!     pub struct Bitfields : u8 {
//!         pub e: Enum,
//!         pub r: Rest,
//!     }
//! }
//!
//! fn main() {
//!     let mut instance = Bitfields::new(Enum::B, Rest(0b010));
//!     assert_eq!(instance.e(), Enum::B);
//!     instance.set_r(Rest(0b101));
//!     assert_eq!(instance.r().0, 0b101);
//! }
//! ```

#![no_std]
#![forbid(unsafe_op_in_unsafe_fn, rust_2018_idioms)]
#![warn(missing_docs)]

mod raw;

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
/// The base type is stored as [`Raw`] in a (private) tuple field `0`. You can call its
/// [`raw()`] method to get the raw bitfields data (but you cannot modify it):
/// ```rust
/// # use superbitty::{bitfields, BitFieldCompatible};
/// #[derive(BitFieldCompatible, Clone, Copy)]
/// enum Enum { A, B }
///
/// bitfields! {
///     pub struct Bitfields : u8 {
///         a: Enum,
///         b: Enum,
///         c: Enum,
///     }
/// }
///
/// let mut instance = Bitfields::new(Enum::B, Enum::A, Enum::B);
/// assert_eq!(instance.0.raw(), 0b101); // `Enum::B` is 1, `Enum::A` is 0.
/// ```
///
/// [`BitFieldCompatible`]: crate::BitFieldCompatible
/// [`Debug`]: core::fmt::Debug
/// [`Hash`]: core::hash::Hash
/// [`raw()`]: Raw::raw
pub use superbitty_macros::bitfields;
/// An enum that can be used as a bitfield.
///
/// It must be [`Copy`].
///
/// It must carry no payload and have no negative discriminants.
///
/// ```
/// # use superbitty::BitFieldCompatible;
/// #[derive(BitFieldCompatible, Clone, Copy)]
/// enum BitFieldCompatibleEnum { A, B, C }
/// ```
pub use superbitty_macros::BitFieldCompatible;

pub use crate::raw::Raw;

/// A type that can be used as a bitfield. This is usually [derived] for enums.
/// Structs and unions can implement this explicitly, as the safety requirements cannot be
/// guaranteed for them with `#[derive()]` (at least not easily).
///
/// # Safety
///
/// [`into_raw()`] must provide a value within the range specified by [`SHIFT`] and [`BITS_LEN`].
///
/// [derived]: macro@BitFieldCompatible
/// [`into_raw()`]: BitFieldCompatible::into_raw
/// [`SHIFT`]: BitFieldCompatible::SHIFT
/// [`BITS_LEN`]: BitFieldCompatible::BITS_LEN
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
