include!(concat!(env!("OUT_DIR"), "/primes25.rs"));

include!("../src/primes_include.rs");

#[test]
fn test_stored_prime_index() {
    assert_eq!(stored_prime_index(2), Option::from(0));
    assert_eq!(stored_prime_index(7), Option::from(3));
    assert_eq!(stored_prime_index(11), Option::from(4));
    assert_eq!(stored_prime_index(19), Option::from(7));
    assert_eq!(stored_prime_index(23), Option::from(8));
    assert_eq!(stored_prime_index(4), Option::None);
    assert_eq!(stored_prime_index(6), Option::None);
    assert_eq!(stored_prime_index(20), Option::None);
    assert_eq!(stored_prime_index(22), Option::None);
    assert_eq!(stored_prime_index(24), Option::None);
}

#[test]
fn test_is_prime() {
    // These can be searched in PRIMES
    assert!(is_prime(2));
    assert!(! is_prime(4));
    // These must be examined by dividing by prime numbers
    assert!(is_prime(29));
    assert!(! is_prime(30));
}
