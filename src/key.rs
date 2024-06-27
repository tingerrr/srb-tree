pub trait Key {
    const SIZE: usize;

    fn to_usize(&self) -> usize;

    // TODO: i don't think this is it
    fn max_depth(branching_factor: usize) -> usize {
        Self::SIZE.ilog(branching_factor) as usize
    }

    /// Yield the index of this key for a given branching factor at the given depth.
    fn index_at(&self, branching_factor: usize, depth: usize) -> usize {
        let key = self.to_usize();
        if branching_factor.is_power_of_two() {
            (key >> (branching_factor.ilog2() as usize * depth)) & (branching_factor - 1)
        } else {
            (key / branching_factor.pow(depth as u32)) % branching_factor
        }
    }

    fn range_at(&self, branching_factor: usize, depth: usize) -> [usize; 2] {
        todo!()
    }
}

macro_rules! impl_key {
    ($key:ty) => {
        impl Key for $key {
            const SIZE: usize = <$key>::BITS as usize;

            fn to_usize(&self) -> usize {
                (*self).try_into().unwrap()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_depth() {
        assert_eq!(usize::max_depth(2), 1);
        assert_eq!(usize::max_depth(32), 6);
    }

    #[test]
    fn test_index_at() {
        assert_eq!(usize::index_at(&31, 32, 0), 31);
        assert_eq!(usize::index_at(&31, 32, 0), 31);
    }
}
