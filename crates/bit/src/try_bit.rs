pub trait TryBit {
    /// Get bit by index, starting from 0, *big endian - from left*
    ///
    /// Checked for any overflows and wraps.
    ///
    /// # Errors
    /// Error is returned on overflow/wrap
    fn try_bit(&self, index: usize) -> Result<bool, crate::Error>;
}

macro_rules! impl_try_bit_for_scalar {
    ( $type:ident ) => {
        impl TryBit for $type {
            fn try_bit(&self, index: usize) -> Result<bool, crate::Error> {
                use std::convert::TryInto;

                let max_shift = size_of::<$type>() * 8 - 1;
                let shift = max_shift
                    .checked_sub(index)
                    .ok_or(crate::Error {})?
                    .try_into()
                    .map_err(|_e| crate::Error {})?;
                let shifted = self.checked_shr(shift).ok_or(crate::Error {})?;
                Ok(shifted & 1 != 0)
            }
        }
    };
}

impl_try_bit_for_scalar!(u8);
impl_try_bit_for_scalar!(u16);
impl_try_bit_for_scalar!(u32);
impl_try_bit_for_scalar!(u64);
impl_try_bit_for_scalar!(u128);
