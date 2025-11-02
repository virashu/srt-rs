pub trait Bit {
    /// Get bit by index, starting from 0, *big endian - from left*
    fn bit(&self, index: usize) -> bool;
}

// // Blanket scalar
//
// impl<T> Bit for T
// where
//     for<'a> &'a T: std::ops::Shr<usize>,
//     for<'a> <&'a T as std::ops::Shr<usize>>::Output: std::ops::BitAnd<i32>,
//     for<'a> <<&'a T as std::ops::Shr<usize>>::Output as std::ops::BitAnd<i32>>::Output:
//         std::cmp::PartialEq<i32>,
// {
//     fn bit(&self, index: usize) -> bool {
//         (self >> (size_of::<T>() * 8 - 1 - index)) & 1 != 0
//     }
// }

impl<T> Bit for [T]
where
    T: Copy + std::ops::Shr<usize>,
    <T as std::ops::Shr<usize>>::Output: std::ops::BitAnd<u8>,
    <<T as std::ops::Shr<usize>>::Output as std::ops::BitAnd<u8>>::Output: std::cmp::PartialEq<u8>,
{
    fn bit(&self, index: usize) -> bool {
        let size = size_of::<T>() * 8;
        let byte_offset = index / size;
        let bit_offset = index % size;
        (self[byte_offset] >> (size - 1 - bit_offset)) & 1 != 0
    }
}

macro_rules! impl_bit_for_scalar {
    ( $type:ident ) => {
        impl Bit for $type {
            fn bit(&self, index: usize) -> bool {
                let max_shift = size_of::<$type>() * 8 - 1;
                (self >> (max_shift - index)) & 1 != 0
            }
        }
    };
}

impl_bit_for_scalar!(u8);
impl_bit_for_scalar!(u16);
impl_bit_for_scalar!(u32);
impl_bit_for_scalar!(u64);
impl_bit_for_scalar!(u128);

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(clippy::bool_assert_comparison)]
    #[test]
    fn test_get_bit_scalar() {
        let num: u16 = 0b0000_0001_1000_0000;

        assert_eq!(num.bit(6), false);
        assert_eq!(num.bit(7), true);
        assert_eq!(num.bit(8), true);
        assert_eq!(num.bit(9), false);
    }

    #[allow(clippy::bool_assert_comparison)]
    #[test]
    fn test_get_bit_collection() {
        let buf: [u8; _] = [0b0000_0001, 0b1000_0000];

        assert_eq!(buf.bit(6), false);
        assert_eq!(buf.bit(7), true);
        assert_eq!(buf.bit(8), true);
        assert_eq!(buf.bit(9), false);
    }
}
