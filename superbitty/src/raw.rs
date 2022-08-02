/// A raw representation of a bitfield with base type `T`. This
/// is used for storing the bitfields data.
///
/// You can get the `T` via the [`raw()`] method.
///
/// [`raw()`]: Raw::raw
pub struct Raw<T: OnlyPrimitiveUnsignedIntegersAreAllowedAsBaseTypesForSuperbittyBitfields>(T);

impl<T: OnlyPrimitiveUnsignedIntegersAreAllowedAsBaseTypesForSuperbittyBitfields> Clone for Raw<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: OnlyPrimitiveUnsignedIntegersAreAllowedAsBaseTypesForSuperbittyBitfields> Copy for Raw<T> {}

impl<T: OnlyPrimitiveUnsignedIntegersAreAllowedAsBaseTypesForSuperbittyBitfields> Raw<T> {
    /// # Safety
    ///
    /// For anyone that is not the `bitfields!` macro, this is always undefined behavior
    /// to call.
    ///
    /// For it, this should uphold the invariant of the bitfields struct that it always
    /// contains valid instances.
    #[doc(hidden)]
    pub const unsafe fn new(v: T) -> Self {
        Self(v)
    }

    /// The raw value of this bitfields.
    pub fn raw(self) -> T {
        self.0
    }

    /// # Safety
    ///
    /// For anyone that is not the `bitfields!` macro, this is always undefined behavior
    /// to call.
    ///
    /// For it, this should uphold the invariant of the bitfields struct that it always
    /// contains valid instances.
    #[doc(hidden)]
    pub unsafe fn get_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

// This trait should not be exposed, it is sealed.
pub trait OnlyPrimitiveUnsignedIntegersAreAllowedAsBaseTypesForSuperbittyBitfields: Copy {}
impl OnlyPrimitiveUnsignedIntegersAreAllowedAsBaseTypesForSuperbittyBitfields for u8 {}
impl OnlyPrimitiveUnsignedIntegersAreAllowedAsBaseTypesForSuperbittyBitfields for u16 {}
impl OnlyPrimitiveUnsignedIntegersAreAllowedAsBaseTypesForSuperbittyBitfields for u32 {}
impl OnlyPrimitiveUnsignedIntegersAreAllowedAsBaseTypesForSuperbittyBitfields for u64 {}
impl OnlyPrimitiveUnsignedIntegersAreAllowedAsBaseTypesForSuperbittyBitfields for u128 {}
impl OnlyPrimitiveUnsignedIntegersAreAllowedAsBaseTypesForSuperbittyBitfields for usize {}
