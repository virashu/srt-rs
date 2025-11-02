use crate::Bit;

pub trait Bits {
    fn bits<T>(&self, offset: usize, amt: usize) -> T
    where
        T: From<bool>
            + std::ops::Shl<usize>
            + std::ops::BitOrAssign<<T as std::ops::Shl<usize>>::Output>;
}

impl Bits for [u8] {
    fn bits<T>(&self, offset: usize, amt: usize) -> T
    where
        T: From<bool>
            + std::ops::Shl<usize>
            + std::ops::BitOrAssign<<T as std::ops::Shl<usize>>::Output>,
    {
        let type_size = size_of::<T>() * 8;

        assert!(type_size >= amt);

        let mut res = T::from(false);
        let zeroes = type_size - amt;

        for (n, pos) in (offset..(offset + amt)).enumerate() {
            let abs_n = n + zeroes;
            let bit = self.bit(pos);
            let mask = T::from(bit) << (type_size - 1 - abs_n);
            res |= mask;
        }

        res
    }
}
