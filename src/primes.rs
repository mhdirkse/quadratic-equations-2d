include!(concat!(env!("OUT_DIR"), "/primes.rs"));
include!("primes_include.rs");

use crate::generation::get_primes::sqrt_floor;

#[derive(Debug, PartialEq)]
pub struct Factor {
    pub factor: u32,
    pub count: u32
}

pub fn factors(n: u32) -> Vec<Factor> {
    if (n == 0) || (n == 1) {
        panic!("Cannot calculate factors of 0 or 1");
    }
    let mut result: Vec<Factor> = Vec::new();
    let mut prime_index: u32 = 0;
    let mut remaining = n;
    loop {
        if remaining == 1 {
            return result;
        }
        if (prime_index >= NUM_PRIMES) || (PRIMES[prime_index as usize] > sqrt_floor(remaining)) {
            result.push(Factor {factor: remaining, count: 1});
            return result;
        }
        let factor = PRIMES[prime_index as usize];
        let count: u32;
        (count, remaining) = next_factor(remaining, factor);
        if count >= 1 {
            result.push(Factor{factor: factor, count: count});
        }
        prime_index += 1;
    }
}

fn next_factor(n: u32, factor: u32) -> (u32, u32) {
    let mut remaining = n;
    let mut count: u32 = 0;
    loop {
        let division = remaining / factor;
        let remainder = remaining % factor;
        if remainder == 0 {
            count += 1;
            remaining = division;
        } else {
            return (count, remaining);
        }
    }
}

#[cfg(test)]
mod test {
    use super::Factor;
    use super::factors;

    #[test]
    fn test_we_do_not_have_reduced_primes_array() {
        let prime_outside_reduced_primes_vector: u32 = 29;
        let n = prime_outside_reduced_primes_vector * prime_outside_reduced_primes_vector;
        assert_eq!(841, n);
        assert!(! super::is_prime(n));
    }

    #[test]
    fn factors_two() {
        assert_eq!(vec![Factor{factor: 2, count: 1}], factors(2));
    }

    #[test]
    fn factors_three() {
        assert_eq!(vec![Factor{factor: 3, count: 1}], factors(3));
    }

    #[test]
    fn factors_four() {
        assert_eq!(vec![Factor{factor: 2, count: 2}], factors(4));
    }

    #[test]
    fn test_two_different_factors_finish_remainder_one() {
        assert_eq!(vec![Factor{factor:2, count: 1}, Factor{factor:3, count:1}], factors(6));
    }

    #[test]
    fn test_remaining_factor_added() {
        assert_eq!(vec![Factor{factor:3, count:2}, Factor{factor:11, count:1}], factors(99));
    }

    #[test]
    fn factors_twelve() {
        assert_eq!(vec![Factor{factor:2, count:2}, Factor{factor:3, count:1}], factors(12));
    }
}