pub trait Key: Ord + Sized {
    const SIZE: u8;

    const MAX: Self;
    const MIN: Self;

    fn to_usize(&self) -> usize;

    /// Return maximum depth for the given the branching factor.
    fn max_depth(branching_factor: usize) -> usize;

    /// Return the index of this key given a branching factor and depth.
    fn index_at(&self, branching_factor: usize, depth: usize) -> usize {
        let key = self.to_usize();
        let depth = Self::max_depth(branching_factor) - depth;

        if branching_factor.is_power_of_two() {
            (key >> (branching_factor.ilog2() as usize * depth)) & (branching_factor - 1)
        } else {
            (key / branching_factor.pow(depth as u32)) % branching_factor
        }
    }
}

macro_rules! impl_key {
    ($key:ty) => {
        impl Key for $key {
            const SIZE: u8 = <$key>::BITS as u8;

            const MIN: Self = <$key>::MIN;
            const MAX: Self = <$key>::MAX;

            fn to_usize(&self) -> usize {
                (*self).try_into().unwrap()
            }

            fn max_depth(branching_factor: usize) -> usize {
                Self::MAX.ilog(branching_factor.try_into().unwrap()) as usize
            }
        }
    };
}

impl_key!(u8);
impl_key!(u16);
impl_key!(u32);
impl_key!(u64);
impl_key!(u128);
impl_key!(usize);

impl_key!(i8);
impl_key!(i16);
impl_key!(i32);
impl_key!(i64);
impl_key!(i128);
impl_key!(isize);

// TODO: non power of two branching factors
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_depth() {
        assert_eq!(u64::max_depth(2), 63);
        assert_eq!(u64::max_depth(16), 15);
        assert_eq!(u64::max_depth(32), 12);

        assert_eq!(u32::max_depth(2), 31);
        assert_eq!(u32::max_depth(16), 7);
        assert_eq!(u32::max_depth(32), 6);
    }

    #[test]
    fn test_index_at() {
        assert_eq!(u32::index_at(&31, 16, 0), 0);
        assert_eq!(u32::index_at(&31, 16, 1), 0);
        assert_eq!(u32::index_at(&31, 16, 2), 0);
        assert_eq!(u32::index_at(&31, 16, 3), 0);
        assert_eq!(u32::index_at(&31, 16, 4), 0);
        assert_eq!(u32::index_at(&31, 16, 5), 0);
        assert_eq!(u32::index_at(&31, 16, 6), 1);
        assert_eq!(u32::index_at(&31, 16, 7), 15);
    }
}
