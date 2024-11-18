use std::marker::PhantomData;
use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

pub trait BinaryFieldPoly<const N: usize> {
    fn get_poly<'a>() -> &'a [bool; N];
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BinaryField<const N: usize, P: BinaryFieldPoly<N>> {
    pub data: [bool; N],
    pub marker: PhantomData<P>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct AESPoly;
impl BinaryFieldPoly<8> for AESPoly {
    fn get_poly<'a>() -> &'a [bool; 8] {
        &[false, false, false, true, true, false, true, true]
    }
}

impl<const N: usize, P: BinaryFieldPoly<N>> Default for BinaryField<N, P> {
    fn default() -> Self {
        Self {
            data: [false; N],
            marker: PhantomData,
        }
    }
}

impl<const N: usize, P: BinaryFieldPoly<N>> From<u8> for BinaryField<N, P> {
    fn from(mut value: u8) -> Self {
        assert!(N >= 8);
        let mut res = BinaryField::<N, P>::default();
        for i in 0..8 {
            res.data[i] = value & 1 == 1;
            value >>= 1;
        }
        res
    }
}

impl<const N: usize, P: BinaryFieldPoly<N>> From<u16> for BinaryField<N, P> {
    fn from(mut value: u16) -> Self {
        assert!(N >= 16);
        let mut res = BinaryField::<N, P>::default();
        for i in 0..16 {
            res.data[i] = value & 1 == 1;
            value >>= 1;
        }
        res
    }
}

impl<const N: usize, P: BinaryFieldPoly<N>> From<u32> for BinaryField<N, P> {
    fn from(mut value: u32) -> Self {
        assert!(N >= 32);
        let mut res = BinaryField::<N, P>::default();
        for i in 0..32 {
            res.data[i] = value & 1 == 1;
            value >>= 1;
        }
        res
    }
}

impl<const N: usize, P: BinaryFieldPoly<N>> From<u64> for BinaryField<N, P> {
    fn from(mut value: u64) -> Self {
        assert!(N >= 64);
        let mut res = BinaryField::<N, P>::default();
        for i in 0..64 {
            res.data[i] = value & 1 == 1;
            value >>= 1;
        }
        res
    }
}

impl<const N: usize, P: BinaryFieldPoly<N>> From<u128> for BinaryField<N, P> {
    fn from(mut value: u128) -> Self {
        assert!(N >= 128);
        let mut res = BinaryField::<N, P>::default();
        for i in 0..128 {
            res.data[i] = value & 1 == 1;
            value >>= 1;
        }
        res
    }
}

impl<const N: usize, P: BinaryFieldPoly<N>> Add<&BinaryField<N, P>> for &BinaryField<N, P> {
    type Output = BinaryField<N, P>;

    fn add(self, rhs: &BinaryField<N, P>) -> BinaryField<N, P> {
        let mut res = BinaryField::<N, P>::default();
        for i in 0..N {
            res.data[i] = self.data[i] ^ rhs.data[i];
        }
        res
    }
}

impl<const N: usize, P: BinaryFieldPoly<N>> AddAssign<&BinaryField<N, P>> for BinaryField<N, P> {
    fn add_assign(&mut self, rhs: &BinaryField<N, P>) {
        for i in 0..N {
            self.data[i] ^= rhs.data[i];
        }
    }
}

impl<const N: usize, P: BinaryFieldPoly<N>> Sub<&BinaryField<N, P>> for &BinaryField<N, P> {
    type Output = BinaryField<N, P>;

    fn sub(self, rhs: &BinaryField<N, P>) -> Self::Output {
        self.add(rhs)
    }
}

impl<const N: usize, P: BinaryFieldPoly<N>> SubAssign<&BinaryField<N, P>> for BinaryField<N, P> {
    fn sub_assign(&mut self, rhs: &BinaryField<N, P>) {
        self.add_assign(rhs)
    }
}

impl<const N: usize, P: BinaryFieldPoly<N>> Mul<&BinaryField<N, P>> for &BinaryField<N, P> {
    type Output = BinaryField<N, P>;

    fn mul(self, rhs: &BinaryField<N, P>) -> Self::Output {
        let mut temp = vec![false; 2 * N - 1];
        for i in 0..N {
            for j in 0..N {
                temp[i + j] ^= self.data[i] & rhs.data[j];
            }
        }

        let poly = P::get_poly();

        for i in (N..(2 * N - 1)).rev() {
            if temp[i] {
                temp[i] = false;
                for j in 0..N {
                    temp[i - 1 - j] ^= poly[j];
                }
            }
        }

        BinaryField::<N, P> {
            data: temp[0..N].try_into().unwrap(),
            marker: PhantomData,
        }
    }
}

impl<const N: usize, P: BinaryFieldPoly<N>> MulAssign<&BinaryField<N, P>> for BinaryField<N, P> {
    fn mul_assign(&mut self, rhs: &BinaryField<N, P>) {
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

            let a_bf = BinaryField::<8, AESPoly>::from(a);
            let b_bf = BinaryField::<8, AESPoly>::from(b);

            let result = a_bf.mul(&b_bf);
            let expected_bf = BinaryField::<8, AESPoly>::from(expected);
            assert_eq!(result, expected_bf);
        }
    }
}
