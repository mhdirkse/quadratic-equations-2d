// Calculate the square root of a rational such that only the root
// of a positive integer appears in the result.

use ::num_rational::Rational32;
use num_traits::Zero;
use crate::primes::factors;

pub fn sqrt(r: Rational32) -> Option<(Rational32, u32)> {
    if r < Rational32::zero() {
        panic!("Cannot take sqrt of negative number");
    }
    let numer = r.numer().to_owned() as u32;
    let denom = r.denom().to_owned() as u32;
    let (denom_square, denom_root) = split_square_div_root(denom);
    let num_norm = numer.checked_mul(denom_root);
    if ! num_norm.is_some() {
        return None;
    }
    let (num_square, num_root) = split_square_times_root(num_norm.unwrap());
    if (num_square > (i32::MAX as u32)) || (denom_square > (i32::MAX as u32)) {
        return None;
    }
    let result_square = Rational32::new(num_square as i32, denom_square as i32);
    return Some((result_square, num_root));
}

fn split_square_times_root(n: u32) -> (u32, u32) {
    if (n == 0) || (n == 1) {
        return (n, 1);
    }
    let mut square: u32 = 1;
    let mut root: u32 = 1;
    let the_factors = factors(n);
    for f in the_factors {
        let remainder = f.count % 2;
        let division = f.count / 2;
        square *= f.factor.pow(division);
        if remainder == 1 {
            root *= f.factor;
        }
    }
    return (square, root);
}

fn split_square_div_root(n: u32) -> (u32, u32) {
    if (n == 0) || (n == 1) {
        return (n, 1);
    }
    let mut square: u32 = 1;
    let mut root: u32 = 1;
    let the_factors = factors(n);
    for f in the_factors {
        let remainder = f.count % 2;
        let division = f.count / 2;
        if remainder == 0 {
            square *= f.factor.pow(division);
        } else {
            square *= f.factor.pow(division + 1);
            root *= f.factor;
        }
    }
    return (square, root);
}

#[cfg(test)]
mod test {
    use super::sqrt;
    use super::split_square_times_root;
    use super::split_square_div_root;
    use ::num_rational::Rational32;

    #[test]
    fn test_split_square_times_root() {
        assert_eq!(split_square_times_root(12), (2, 3));
    }

    #[test]
    fn test_split_square_div_root() {
        assert_eq!(split_square_div_root(12), (6, 3));
    }

    #[test]
    fn test_sqrt() {
        assert_eq!(sqrt(Rational32::new(12, 1)), Some((Rational32::new(2, 1), 3)));
        assert_eq!(sqrt(Rational32::new(1, 2)), Some((Rational32::new(1, 2), 2)));
        assert_eq!(sqrt(Rational32::new(64, 25)), Some((Rational32::new(8, 5), 1)));
    }
}