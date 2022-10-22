fn main() {
    let s: String = get_primes::get_primes(25).iter()
        .map(|p| p.to_string())
        .collect::<Vec<String>>().join(", ");
    println!("{}", s);
}

mod get_primes {
    // Calculate all prime numbers that are at most max_prime.
    //
    // Store all numbers from 2 to max_prime. For each prime,
    // remove all multiples of that prime from the list.
    // If a prime satisfies prime * prime > max_prime, there
    // will not be values to be removed from the list. If there
    // were a value v <= max_prime that satisfies d1 * d2 == v,
    // then d1 or d2 <= prime.

    use std::cmp::min;

    const TOP_ROOT_OF_U32: u32 = (1 as u32) << 16;

    pub fn get_primes(max_prime: u32) -> Vec<u32> {
        if max_prime <= 1 {
            return vec![];
        }
        let mut result = vec![];
        for i in 2 ..= max_prime {
            result.push(i);
        }
        let max_current_prime = sqrt_floor(max_prime);
        let mut current_prime_index = 0;
        loop {
            let current_prime = result[current_prime_index];
            if current_prime > max_current_prime {
                break;
            }
            result = remove_multiples_of(current_prime, result);
            current_prime_index += 1;
        }
        return result;
    }

    fn sqrt_floor(v: u32) -> u32 {
        if (v == 0) || (v == 1) {
            return v;
        }
        return sqrt_bounds(1, min(v, TOP_ROOT_OF_U32), v);
    }

    fn sqrt_bounds(min_root: u32, top: u32, v: u32) -> u32 {
        if top == (min_root + 1) {
            return min_root;
        }
        let middle = get_middle(min_root, top);
        if middle * middle <= v {
            return sqrt_bounds(middle, top, v);
        } else {
            return sqrt_bounds(min_root, middle, v);
        }
    }

    fn get_middle(v1: u32, v2: u32) -> u32 {
        let v164 = v1 as u64;
        let v264 = v2 as u64;
        let middle64 = (v164 + v264) / 2;
        return middle64 as u32;
    }

    fn remove_multiples_of(p: u32, v: Vec<u32>) -> Vec<u32> {
        let mut result: Vec<u32> = vec![];
        for e in v {
            let quot = e / p;
            let rem = e % p;
            if ! ((rem == 0) && (quot >= 2)) {
                result.push(e);
            }
        }
        return result;
    }

    #[cfg(test)]
    mod tests {
        use super::{
            TOP_ROOT_OF_U32,
            get_middle,
            sqrt_floor
        };

        use std::u32::MAX;

        #[test]
        fn max_u32_is_ffffffff() {
            assert_eq!(0xffffffff, MAX)
        }

        #[test]
        fn max_root_of_uint32_is_2_pow_16() {
            let expected: u32 = (256 as u32) * (256 as u32);
            assert_eq!(expected, TOP_ROOT_OF_U32);
        }

        #[test]
        fn middle_with_difference_2_does_not_repeat_bounds() {
            assert_eq!(get_middle(3, 5), 4);
            assert_eq!(get_middle(4, 6), 5);
            assert_eq!(get_middle(5, 3), 4);
            assert_eq!(get_middle(6, 4), 5);
        }

        #[test]
        fn test_sqrt_floor() {
            assert_eq!(sqrt_floor(0), 0);
            assert_eq!(sqrt_floor(1), 1);
            assert_eq!(sqrt_floor(2), 1);
            assert_eq!(sqrt_floor(3), 1);
            assert_eq!(sqrt_floor(4), 2);
            assert_eq!(sqrt_floor(5), 2);
            assert_eq!(sqrt_floor(99), 9);
            assert_eq!(sqrt_floor(100), 10);
            assert_eq!(sqrt_floor(101), 10);
            assert_eq!(sqrt_floor(MAX), TOP_ROOT_OF_U32 - 1);
            assert_eq!(sqrt_floor(MAX - 1), TOP_ROOT_OF_U32 - 1);
            assert_eq!(sqrt_floor(0xffff * 0xffff), 0xffff);
            assert_eq!(sqrt_floor(0xffff * 0xffff - 1), 0xfffe);
            assert_eq!(sqrt_floor(0xfffe * 0xfffe), 0xfffe);
        }
    }

    #[test]
    fn test_primes() {
        assert_eq!(get_primes(0), vec![]);
        assert_eq!(get_primes(1), vec![]);
        assert_eq!(get_primes(2), vec![2]);
        assert_eq!(get_primes(9), vec![2, 3, 5, 7]);
        assert_eq!(get_primes(10), vec![2, 3, 5, 7]);
        assert_eq!(get_primes(25), vec![2, 3, 5, 7, 11, 13, 17, 19, 23]);
    }
}