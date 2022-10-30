// This file is only used by including it in other files.
//
// In test/primes.rs, it is included in combination with
// stored primes bounded by 25. This way, we can test
// easily with numbers that are outside the range of stored
// primes. Without working with a reduced array of stored primes,
// we would have to test with large numbers to cover all code.
use std::option::Option;

pub fn is_prime(n: u32) -> bool {
    if n <= PRIMES_BOUND {
        return match stored_prime_index(n) {
            Some(_) => true,
            None => false
        }
    }
    for p in PRIMES {
        if n % p == 0 {
            return false;
        }
    }
    return true;
}

fn stored_prime_index(n: u32) -> Option<usize> {
    let index_max_stored_prime = (NUM_PRIMES - 1) as usize;
    let max_stored_prime = PRIMES[index_max_stored_prime];
    if n >=  max_stored_prime {
        if n == max_stored_prime {
            return Option::from(index_max_stored_prime);
        } else {
            return Option::None;
        }
    }
    if n <= 1 {
        return Option::None;
    }
    let mut min_index: usize = 0;
    let mut top_index = index_max_stored_prime;
    while min_index + 1 < top_index {
        let middle_index = (min_index + top_index) / 2;
        if n < PRIMES[middle_index] {
            top_index = middle_index;
        } else {
            min_index = middle_index;
        }
    }
    if PRIMES[min_index] == n {
        return Option::from(min_index);
    } else {
        return Option::None;
    }
}
