include!(concat!(env!("OUT_DIR"), "/primes.rs"));
include!("primes_include.rs");

#[cfg(test)]
mod test {
    #[test]
    fn test_we_do_not_have_reduced_primes_array() {
        let prime_outside_reduced_primes_vector: u32 = 29;
        let n = prime_outside_reduced_primes_vector * prime_outside_reduced_primes_vector;
        assert_eq!(841, n);
        assert!(! super::is_prime(n));
    }
}