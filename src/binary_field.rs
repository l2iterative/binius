use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

pub trait BinaryFieldConfig: Clone + Debug {
    const N: usize;

    fn get_poly<'a>() -> &'a [bool];
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BinaryField<P: BinaryFieldConfig> {
    pub data: Vec<bool>,
    pub marker: PhantomData<P>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AESPoly;
impl BinaryFieldConfig for AESPoly {
    const N: usize = 8;

    fn get_poly<'a>() -> &'a [bool] {
        &[false, false, false, true, true, false, true, true]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct F2;
impl BinaryFieldConfig for F2 {
    const N: usize = 1;

    fn get_poly<'a>() -> &'a [bool] {
        &[false]
    }
}

impl<P: BinaryFieldConfig> Default for BinaryField<P> {
    fn default() -> Self {
        Self {
            data: vec![false; P::N],
            marker: PhantomData,
        }
    }
}

impl<P: BinaryFieldConfig> From<u8> for BinaryField<P> {
    fn from(mut value: u8) -> Self {
        assert!(P::N >= 8);
        let mut res = BinaryField::<P>::default();
        for i in 0..8 {
            res.data[i] = value & 1 == 1;
            value >>= 1;
        }
        res
    }
}

impl<P: BinaryFieldConfig> From<u16> for BinaryField<P> {
    fn from(mut value: u16) -> Self {
        assert!(P::N >= 16);
        let mut res = BinaryField::<P>::default();
        for i in 0..16 {
            res.data[i] = value & 1 == 1;
            value >>= 1;
        }
        res
    }
}

impl<P: BinaryFieldConfig> From<u32> for BinaryField<P> {
    fn from(mut value: u32) -> Self {
        assert!(P::N >= 32);
        let mut res = BinaryField::<P>::default();
        for i in 0..32 {
            res.data[i] = value & 1 == 1;
            value >>= 1;
        }
        res
    }
}

impl<P: BinaryFieldConfig> From<u64> for BinaryField<P> {
    fn from(mut value: u64) -> Self {
        assert!(P::N >= 64);
        let mut res = BinaryField::<P>::default();
        for i in 0..64 {
            res.data[i] = value & 1 == 1;
            value >>= 1;
        }
        res
    }
}

impl<P: BinaryFieldConfig> From<u128> for BinaryField<P> {
    fn from(mut value: u128) -> Self {
        assert!(P::N >= 128);
        let mut res = BinaryField::<P>::default();
        for i in 0..128 {
            res.data[i] = value & 1 == 1;
            value >>= 1;
        }
        res
    }
}

impl<P: BinaryFieldConfig> Add<&BinaryField<P>> for &BinaryField<P> {
    type Output = BinaryField<P>;

    fn add(self, rhs: &BinaryField<P>) -> BinaryField<P> {
        let mut res = BinaryField::<P>::default();
        for i in 0..P::N {
            res.data[i] = self.data[i] ^ rhs.data[i];
        }
        res
    }
}

impl<P: BinaryFieldConfig> AddAssign<&BinaryField<P>> for BinaryField<P> {
    fn add_assign(&mut self, rhs: &BinaryField<P>) {
        for i in 0..P::N {
            self.data[i] ^= rhs.data[i];
        }
    }
}

impl<P: BinaryFieldConfig> Sub<&BinaryField<P>> for &BinaryField<P> {
    type Output = BinaryField<P>;

    fn sub(self, rhs: &BinaryField<P>) -> Self::Output {
        self.add(rhs)
    }
}

impl<P: BinaryFieldConfig> SubAssign<&BinaryField<P>> for BinaryField<P> {
    fn sub_assign(&mut self, rhs: &BinaryField<P>) {
        self.add_assign(rhs)
    }
}

impl<P: BinaryFieldConfig> Mul<&BinaryField<P>> for &BinaryField<P> {
    type Output = BinaryField<P>;

    fn mul(self, rhs: &BinaryField<P>) -> Self::Output {
        let mut temp = vec![false; 2 * P::N - 1];
        for i in 0..P::N {
            for j in 0..P::N {
                temp[i + j] ^= self.data[i] & rhs.data[j];
            }
        }

        let poly = P::get_poly();

        for i in (P::N..(2 * P::N - 1)).rev() {
            if temp[i] {
                temp[i] = false;
                for j in 0..P::N {
                    temp[i - 1 - j] ^= poly[j];
                }
            }
        }

        BinaryField::<P> {
            data: temp[0..P::N].try_into().unwrap(),
            marker: PhantomData,
        }
    }
}

impl<P: BinaryFieldConfig> MulAssign<&BinaryField<P>> for BinaryField<P> {
    fn mul_assign(&mut self, rhs: &BinaryField<P>) {
        *self = self.mul(rhs);
    }
}

#[cfg(test)]
mod test {
    use crate::binary_field::{AESPoly, BinaryField};
    use rand::Rng;
    use rand_chacha::rand_core::SeedableRng;
    use rand_chacha::ChaCha20Rng;
    use std::ops::Mul;

    #[test]
    fn test_aes_field_mul() {
        fn reference_implementation(mut a: u8, mut b: u8) -> u8 {
            let mut sum = 0u8;

            for _ in 0..8 {
                if b & 1 == 1 {
                    sum ^= a;
                }
                b >>= 1;

                let h = a & 0x80 != 0;
                a <<= 1;
                if h {
                    a ^= 0x1b;
                }
            }

            sum
        }

        let mut prng = ChaCha20Rng::seed_from_u64(0);

        for _ in 0..100 {
            let a: u8 = prng.gen();
            let b: u8 = prng.gen();

            let expected = reference_implementation(a, b);

            let a_bf = BinaryField::<AESPoly>::from(a);
            let b_bf = BinaryField::<AESPoly>::from(b);

            let result = a_bf.mul(&b_bf);
            let expected_bf = BinaryField::<AESPoly>::from(expected);
            assert_eq!(result, expected_bf);
        }
    }
}
