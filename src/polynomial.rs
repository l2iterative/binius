use crate::binary_field::BinaryFieldConfig;
use crate::tower::Ring;

pub struct Polynomial<P: BinaryFieldConfig> {
    pub evaluations: Vec<Ring<P>>,
}

impl<P: BinaryFieldConfig> Polynomial<P> {
    pub fn is_power_of_two(&self) -> bool {
        self.evaluations.len().is_power_of_two()
    }

    pub fn evaluate(&self, x: &[Ring<P>]) -> Ring<P> {
        let dim = (self.evaluations.len() as u32).ilog2() as usize;
        assert_eq!(dim, x.len());

        let mut poly = self.evaluations.clone();
        for i in 1..dim + 1 {
            let r = &x[i - 1];
            for b in 0..(1 << (dim - i)) {
                let left = &poly[b << 1];
                let right = &poly[(b << 1) + 1];
                poly[b] = left + &(r * &(right - left));
            }
        }
        poly[0].clone()
    }
}

#[cfg(test)]
mod test {
    use crate::binary_field::AESPoly;
    use crate::polynomial::Polynomial;
    use crate::tower::Ring;
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;

    #[test]
    fn test_simple() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let mut evaluations = vec![];
        for _ in 0..4 {
            evaluations.push(Ring::<AESPoly>::random(4, &mut prng));
        }

        let polynomial = Polynomial { evaluations };

        let r1 = Ring::<AESPoly>::random(16, &mut prng);
        let r2 = Ring::<AESPoly>::random(16, &mut prng);

        let one_minus_r1 = &Ring::<AESPoly>::one() - &r1;
        let one_minus_r2 = &Ring::<AESPoly>::one() - &r2;

        let res = polynomial.evaluate(&[r1.clone(), r2.clone()]);

        let mut expected = Ring::<AESPoly>::zero();
        expected += &(&(&one_minus_r1 * &one_minus_r2) * &polynomial.evaluations[0]);
        expected += &(&(&r1 * &one_minus_r2) * &polynomial.evaluations[1]);
        expected += &(&(&one_minus_r1 * &r2) * &polynomial.evaluations[2]);
        expected += &(&(&r1 * &r2) * &polynomial.evaluations[3]);

        assert_eq!(expected, res);
    }
}
