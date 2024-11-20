use crate::binary_field::{BinaryField, BinaryFieldConfig};
use std::marker::PhantomData;
use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ring<F: BinaryFieldConfig> {
    pub elements: Vec<BinaryField<F>>,
}

impl<F: BinaryFieldConfig> Default for Ring<F> {
    fn default() -> Self {
        Ring {
            elements: vec![BinaryField::<F>::default()],
        }
    }
}

impl<F: BinaryFieldConfig> Ring<F> {
    pub fn get_level(&self) -> usize {
        self.get_len().ilog2() as usize
    }

    pub fn get_len(&self) -> usize {
        self.elements.len()
    }
}

impl<F: BinaryFieldConfig> Add<&Ring<F>> for &Ring<F> {
    type Output = Ring<F>;

    fn add(self, rhs: &Ring<F>) -> Self::Output {
        let mut res = self.elements.clone();
        res.reserve(rhs.elements.len());
        for i in 0..rhs.elements.len() {
            res[i] += &rhs.elements[i];
        }
        Ring { elements: res }
    }
}

impl<F: BinaryFieldConfig> AddAssign<&Ring<F>> for Ring<F> {
    fn add_assign(&mut self, rhs: &Ring<F>) {
        *self = (self as &Ring<F>) + rhs;
    }
}

impl<F: BinaryFieldConfig> Sub<&Ring<F>> for &Ring<F> {
    type Output = Ring<F>;

    fn sub(self, rhs: &Ring<F>) -> Self::Output {
        self.add(rhs)
    }
}

impl<F: BinaryFieldConfig> SubAssign<&Ring<F>> for Ring<F> {
    fn sub_assign(&mut self, rhs: &Ring<F>) {
        *self = (self as &Ring<F>) - rhs;
    }
}

impl<F: BinaryFieldConfig> Mul<&Ring<F>> for &Ring<F> {
    type Output = Ring<F>;

    fn mul(self, rhs: &Ring<F>) -> Self::Output {
        if self.get_len() != rhs.get_len() {
            let mut long = self;
            let mut short = rhs;
            if long.get_len() < short.get_len() {
                std::mem::swap(&mut long, &mut short);
            }

            let long_len = long.get_len();
            let short_len = short.get_len();

            let k = long_len / short_len;
            let mut res = vec![];

            for i in 0..k {
                let chunk_result = &Ring::<F> {
                    elements: long.elements[(short_len * i)..(short_len * (i + 1))].to_vec(),
                } * short;
                res.extend(chunk_result.elements);
            }

            Ring { elements: res }
        } else {
            let res = recursive_mul(&self.elements, &rhs.elements);
            Ring { elements: res }
        }
    }
}

impl<F: BinaryFieldConfig> MulAssign<&Ring<F>> for Ring<F> {
    fn mul_assign(&mut self, rhs: &Ring<F>) {
        *self = (self as &Ring<F>) * rhs;
    }
}

impl<F: BinaryFieldConfig> Ring<F> {
    pub fn from_bytes(l: usize, value: &[u8]) -> Self {
        let mut bits_le = vec![];
        for byte in value.iter().rev() {
            let mut cur = *byte;
            for _ in 0..8 {
                bits_le.push(cur & 1 == 1);
                cur >>= 1;
            }
        }
        let mut iter = bits_le.iter();

        let mut elements = vec![];
        for _ in 0..l {
            let mut data = vec![];
            for _ in 0..F::N {
                data.push(*iter.next().unwrap());
            }
            elements.push(BinaryField::<F> {
                data,
                marker: PhantomData,
            });
        }
        Ring { elements }
    }
}

fn add_limbs_helper<F: BinaryFieldConfig>(
    a: &[BinaryField<F>],
    b: &[BinaryField<F>],
) -> Vec<BinaryField<F>> {
    a.iter()
        .zip(b.iter())
        .map(|(a, b)| a + b)
        .collect::<Vec<BinaryField<F>>>()
}

fn mul_by_imag_unit<F: BinaryFieldConfig>(a: &[BinaryField<F>]) -> Vec<BinaryField<F>> {
    let len = a.len();
    if len == 1 {
        return vec![a[0].mul_by_imag_unit()];
    }

    assert!(len.is_power_of_two());
    assert!(len >= 2);
    if len == 2 {
        let high = &a[0] + &a[1].mul_by_imag_unit();
        let low = a[1].clone();
        vec![low, high]
    } else {
        let half_len = len / 2;

        let low = a[half_len..].to_vec();
        let shift = mul_by_imag_unit(&low);
        let high = add_limbs_helper(&shift, &a[..half_len]);

        let mut res = low;
        res.extend(high);
        res
    }
}

fn recursive_mul<F: BinaryFieldConfig>(
    a: &[BinaryField<F>],
    b: &[BinaryField<F>],
) -> Vec<BinaryField<F>> {
    assert_eq!(a.len(), b.len());
    if a.len() == 1 {
        return vec![&a[0] * &b[0]];
    }

    assert!(a.len().is_power_of_two());
    let half_len = a.len() / 2;

    let a_low = a[0..half_len].to_vec();
    let a_high = a[half_len..].to_vec();
    let b_low = b[0..half_len].to_vec();
    let b_high = b[half_len..].to_vec();

    let a_sum = add_limbs_helper(&a_low, &a_high);
    let b_sum = add_limbs_helper(&b_low, &b_high);

    let a_low_times_b_low = recursive_mul(&a_low, &b_low);
    let a_high_times_b_high = recursive_mul(&a_high, &b_high);
    let a_sum_times_b_sum = recursive_mul(&a_sum, &b_sum);

    let mut mid_term = add_limbs_helper(&a_sum_times_b_sum, &a_low_times_b_low);
    mid_term = add_limbs_helper(&mid_term, &a_high_times_b_high);

    let shift = mul_by_imag_unit(&a_high_times_b_high);
    let high_term = add_limbs_helper(&mid_term, &shift);

    let low_term = add_limbs_helper(&a_low_times_b_low, &a_high_times_b_high);

    let mut res = low_term;
    res.extend(high_term);
    res
}

#[cfg(test)]
mod test {
    use crate::binary_field::{AESPoly, F2};
    use crate::tower::Ring;

    #[test]
    fn test_simple() {
        let a = Ring::<F2>::from_bytes(4, &[0x02]);
        let b = Ring::<F2>::from_bytes(4, &[0x04]);
        let c = &a * &b;
        assert_eq!(c, Ring::<F2>::from_bytes(4, &[0x08]));

        let a = Ring::<F2>::from_bytes(4, &[0x02]);
        let b = Ring::<F2>::from_bytes(4, &[0x02]);
        let c = &a * &b;
        assert_eq!(c, Ring::<F2>::from_bytes(4, &[0x03]));

        let a = Ring::<F2>::from_bytes(4, &[0x08]);
        let b = Ring::<F2>::from_bytes(4, &[0x08]);
        let c = &a * &b;
        assert_eq!(c, Ring::<F2>::from_bytes(4, &[0x07]));
    }

    #[test]
    fn test_complex() {
        let a = Ring::<AESPoly>::from_bytes(2, &[0x95, 0xf9]);
        let b = Ring::<AESPoly>::from_bytes(2, &[0xcd, 0x19]);
        let c = &a * &b;
        let expected_c = Ring::<AESPoly>::from_bytes(2, &[0x49, 0xc4]);
        assert_eq!(c, expected_c);

        let a = Ring::<AESPoly>::from_bytes(4, &[0xb7, 0x36, 0x28, 0x63]);
        let b = Ring::<AESPoly>::from_bytes(4, &[0x25, 0xdd, 0x21, 0xea]);
        let c = &a * &b;
        let expected_c = Ring::<AESPoly>::from_bytes(4, &[0xa4, 0xfc, 0x16, 0x83]);
        assert_eq!(c, expected_c);

        let a = Ring::<AESPoly>::from_bytes(8, &[0x0d, 0xf0, 0x57, 0xe3, 0xb4, 0xa3, 0x1b, 0x25]);
        let b = Ring::<AESPoly>::from_bytes(8, &[0xb0, 0x6d, 0x02, 0x58, 0x8e, 0x45, 0x21, 0x43]);
        let c = &a * &b;
        let expected_c =
            Ring::<AESPoly>::from_bytes(8, &[0x86, 0x2d, 0x67, 0xe0, 0xcc, 0xcd, 0x37, 0xc6]);
        assert_eq!(c, expected_c);

        let a = Ring::<AESPoly>::from_bytes(
            16,
            &[
                0x3a, 0x24, 0x4a, 0xd6, 0xe1, 0x8b, 0x42, 0xf5, 0x98, 0x3e, 0x40, 0x63, 0x27, 0xe2,
                0x37, 0x53,
            ],
        );
        let b = Ring::<AESPoly>::from_bytes(
            16,
            &[
                0x79, 0x8d, 0x75, 0x51, 0x44, 0x46, 0x1e, 0xf3, 0x58, 0x89, 0x58, 0xf3, 0x49, 0xad,
                0xeb, 0xe4,
            ],
        );
        let c = &a * &b;
        let expected_c = Ring::<AESPoly>::from_bytes(
            16,
            &[
                0x70, 0x57, 0xea, 0x93, 0x10, 0xe6, 0x39, 0x9e, 0x07, 0xe5, 0x8e, 0x18, 0xce, 0x80,
                0xd8, 0xa4,
            ],
        );
        assert_eq!(c, expected_c);
    }
}
