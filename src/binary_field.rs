use rand::distributions::Standard;
use rand::prelude::Distribution;
use rand::Rng;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

pub trait BinaryFieldConfig: Clone + Debug + PartialEq + Eq {
    const N: usize;

    fn get_poly<'a>() -> &'a [bool];

    fn get_imag_unit<'a>() -> &'a [bool];
}

#[derive(Clone, PartialEq, Eq)]
pub struct BinaryField<F: BinaryFieldConfig> {
    pub data: Vec<bool>,
    pub marker: PhantomData<F>,
}

impl<F: BinaryFieldConfig> Debug for BinaryField<F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.data.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AESPoly;
impl BinaryFieldConfig for AESPoly {
    const N: usize = 8;

    fn get_poly<'a>() -> &'a [bool] {
        &[false, false, false, true, true, false, true, true]
    }

    fn get_imag_unit<'a>() -> &'a [bool] {
        &[true, true, false, false, true, false, true, true]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct F2;
impl BinaryFieldConfig for F2 {
    const N: usize = 1;

    fn get_poly<'a>() -> &'a [bool] {
        &[false]
    }

    fn get_imag_unit<'a>() -> &'a [bool] {
        &[true]
    }
}

impl<F: BinaryFieldConfig> BinaryField<F> {
    pub fn zero() -> Self {
        Self::default()
    }

    pub fn one() -> Self {
        let mut res = Self::default();
        res.data[0] = true;
        res
    }
}

impl<F: BinaryFieldConfig> Default for BinaryField<F> {
    fn default() -> Self {
        Self {
            data: vec![false; F::N],
            marker: PhantomData,
        }
    }
}

impl<F: BinaryFieldConfig> From<u8> for BinaryField<F> {
    fn from(mut value: u8) -> Self {
        assert!(F::N >= 8);
        let mut res = BinaryField::<F>::default();
        for i in 0..8 {
            res.data[i] = value & 1 == 1;
            value >>= 1;
        }
        res
    }
}

impl<F: BinaryFieldConfig> From<u16> for BinaryField<F> {
    fn from(mut value: u16) -> Self {
        assert!(F::N >= 16);
        let mut res = BinaryField::<F>::default();
        for i in 0..16 {
            res.data[i] = value & 1 == 1;
            value >>= 1;
        }
        res
    }
}

impl<F: BinaryFieldConfig> From<u32> for BinaryField<F> {
    fn from(mut value: u32) -> Self {
        assert!(F::N >= 32);
        let mut res = BinaryField::<F>::default();
        for i in 0..32 {
            res.data[i] = value & 1 == 1;
            value >>= 1;
        }
        res
    }
}

impl<F: BinaryFieldConfig> From<u64> for BinaryField<F> {
    fn from(mut value: u64) -> Self {
        assert!(F::N >= 64);
        let mut res = BinaryField::<F>::default();
        for i in 0..64 {
            res.data[i] = value & 1 == 1;
            value >>= 1;
        }
        res
    }
}

impl<F: BinaryFieldConfig> From<u128> for BinaryField<F> {
    fn from(mut value: u128) -> Self {
        assert!(F::N >= 128);
        let mut res = BinaryField::<F>::default();
        for i in 0..128 {
            res.data[i] = value & 1 == 1;
            value >>= 1;
        }
        res
    }
}

impl<F: BinaryFieldConfig> Add<&BinaryField<F>> for &BinaryField<F> {
    type Output = BinaryField<F>;

    fn add(self, rhs: &BinaryField<F>) -> BinaryField<F> {
        let mut res = BinaryField::<F>::default();
        for i in 0..F::N {
            res.data[i] = self.data[i] ^ rhs.data[i];
        }
        res
    }
}

impl<F: BinaryFieldConfig> Add<BinaryField<F>> for BinaryField<F> {
    type Output = BinaryField<F>;

    fn add(self, rhs: Self) -> Self::Output {
        &self + &rhs
    }
}

impl<F: BinaryFieldConfig> AddAssign<&BinaryField<F>> for BinaryField<F> {
    fn add_assign(&mut self, rhs: &BinaryField<F>) {
        for i in 0..F::N {
            self.data[i] ^= rhs.data[i];
        }
    }
}

impl<F: BinaryFieldConfig> Sub<&BinaryField<F>> for &BinaryField<F> {
    type Output = BinaryField<F>;

    fn sub(self, rhs: &BinaryField<F>) -> Self::Output {
        self.add(rhs)
    }
}

impl<F: BinaryFieldConfig> SubAssign<&BinaryField<F>> for BinaryField<F> {
    fn sub_assign(&mut self, rhs: &BinaryField<F>) {
        self.add_assign(rhs)
    }
}

impl<F: BinaryFieldConfig> Mul<&BinaryField<F>> for &BinaryField<F> {
    type Output = BinaryField<F>;

    fn mul(self, rhs: &BinaryField<F>) -> Self::Output {
        let mut temp = vec![false; 2 * F::N - 1];
        for i in 0..F::N {
            for j in 0..F::N {
                temp[i + j] ^= self.data[i] & rhs.data[j];
            }
        }

        let poly = F::get_poly();

        for i in (F::N..(2 * F::N - 1)).rev() {
            if temp[i] {
                temp[i] = false;
                for j in 0..F::N {
                    temp[i - 1 - j] ^= poly[j];
                }
            }
        }

        BinaryField::<F> {
            data: temp[0..F::N].try_into().unwrap(),
            marker: PhantomData,
        }
    }
}

impl<F: BinaryFieldConfig> MulAssign<&BinaryField<F>> for BinaryField<F> {
    fn mul_assign(&mut self, rhs: &BinaryField<F>) {
        *self = (self as &Self).mul(rhs);
    }
}

impl<F: BinaryFieldConfig> BinaryField<F> {
    pub fn mul_by_imag_unit(&self) -> BinaryField<F> {
        let imag_unit = BinaryField::<F> {
            data: F::get_imag_unit().to_vec(),
            marker: PhantomData,
        };
        self * &imag_unit
    }
}

impl<F: BinaryFieldConfig> Distribution<BinaryField<F>> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> BinaryField<F> {
        let mut data = vec![];
        for _ in 0..F::N {
            data.push(rng.gen());
        }
        BinaryField::<F> {
            data,
            marker: PhantomData,
        }
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
