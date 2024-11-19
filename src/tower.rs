use crate::binary_field::{BinaryField, BinaryFieldConfig};
use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};

#[derive(Debug, Clone)]
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
    assert!(len.is_power_of_two());
    assert!(len >= 2);
    if len == 2 {
        let high = &a[0] + &a[1];
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

    let high_term = if a.len() != 2 {
        let shift = mul_by_imag_unit(&a_high_times_b_high);
        add_limbs_helper(&mid_term, &shift)
    } else {
        add_limbs_helper(&mid_term, &a_high_times_b_high)
    };

    let low_term = add_limbs_helper(&a_low_times_b_low, &a_high_times_b_high);

    let mut res = low_term;
    res.extend(high_term);
    res
}

#[cfg(test)]
mod test {
    use crate::binary_field::{BinaryField, F2};
    use crate::tower::Ring;
    use std::marker::PhantomData;

    #[test]
    fn test_simple() {
        type E = BinaryField<F2>;

        let zero = E {
            data: vec![false],
            marker: PhantomData,
        };
        let one = E {
            data: vec![true],
            marker: PhantomData,
        };

        let a = Ring::<F2> {
            elements: vec![zero.clone(), one.clone(), zero.clone(), zero.clone()],
        };
        let b = Ring::<F2> {
            elements: vec![zero.clone(), zero.clone(), one.clone(), zero.clone()],
        };
        let c = &a * &b;
        assert_eq!(
            c.elements,
            vec![zero.clone(), zero.clone(), zero.clone(), one.clone()]
        );
    }
}
