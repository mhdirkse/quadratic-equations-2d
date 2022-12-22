use ::num_rational::Rational32;
use ::num_traits::ops::checked::CheckedMul;
use ::num_traits::ops::checked::CheckedAdd;
use ::fail::fail_point;

#[test]
fn automatically_simplified() {
    let f= Rational32::new(4, 6);
    assert_eq!(f.numer(), &2);
    assert_eq!(f.denom(), &3);
}

#[test]
fn big_automatically_simplified() {
    let biggest_prime: i32 = 65521;
    let n = 13 * biggest_prime;
    let d: i32 = 17 * biggest_prime;
    let r = Rational32::new(n, d);
    assert_eq!(&13, r.numer());
    assert_eq!(&17, r.denom());
}

#[test]
fn do_plus() {
    let r = Rational32::new(1, 2) + Rational32::new(1, 3);
    assert_eq!(r.numer(), &5);
    assert_eq!(r.denom(), &6);
}

#[test]
fn do_minus() {
    let r = Rational32::new(5, 6) - Rational32::new(1, 3);
    assert_eq!(r.numer(), &1);
    assert_eq!(r.denom(), &2);
}

#[test]
fn do_multiply() {
    let r = Rational32::new(2, 3) * Rational32::new(3, 4);
    assert_eq!(r.numer(), &1);
    assert_eq!(r.denom(), &2);
}

#[test]
fn do_divide() {
    let r = Rational32::new(2, 3) / Rational32::new(4, 3);
    assert_eq!(r.numer(), &1);
    assert_eq!(r.denom(), &2);
}

#[test]
fn check_equality() {
    let r = Rational32::new(1, 2) - Rational32::new(2, 4);
    assert!(r == Rational32::new(0, 2));
    assert!(r != Rational32::new(1, 1));
}

#[test]
fn compare() {
    let one_half = Rational32::new(1, 2);
    let one_third = Rational32::new(1, 3);
    assert!(one_third < one_half);
    assert!(one_half > one_third);
}

#[test]
fn checked_add_no_overflow() {
    let r = Rational32::from(3);
    let no_overflow = r.checked_add(&r);
    match no_overflow {
        None => fail_point!("No overflow expected"),
        Some(x) => assert_eq!(Rational32::from(6), x)
    }
}

#[test]
fn checked_add_overflow() {
    let r = Rational32::from(2 * 1000 * 1000 * 1000);
    let overflow = r.checked_add(&r);
    if let Some(_) = overflow {
        fail_point!("Overflow expected");
    }
}

#[test]
fn checked_mul_no_overflow() {
    let r = Rational32::from(3);
    let no_overflow = r.checked_mul(&r);
    match no_overflow {
        None => fail_point!("No overflow expected"),
        Some(x) => assert_eq!(Rational32::from(9), x)
    }
}

#[test]
fn checked_mul_overflow() {
    let r = Rational32::new(1000 * 1000, 3);
    let overflow = r.checked_mul(&r);
    if let Some(_) = overflow {
        fail_point!("Overflow expected");
    }
}