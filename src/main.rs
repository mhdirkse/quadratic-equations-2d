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
    // were a value v <= max_prime that satisfies d * prime == v,
    // d < prime. The value v would have been marked as non-prime
    // when d or one of its factors would be considered for removing
    // multitudes.

    use std::cmp::min;

    const TOP_ROOT_OF_U32: u32 = (1 as u32) << 16;

    pub fn get_primes(prime_list_upper_bound: u32) -> Vec<u32> {
        if prime_list_upper_bound <= 1 {
            return vec![];
        }
        let mut prime_list: Vec<u32> = (2 ..= prime_list_upper_bound).collect();
        let filter_upper_bound = sqrt_floor(prime_list_upper_bound);
        let mut filter_index = 0;
        loop {
            let filter = prime_list[filter_index];
            if filter > filter_upper_bound {
                break;
            }
            prime_list.retain(|e| ! ((e % filter == 0) && (e / filter >= 2)));
            filter_index += 1;
        }
        return prime_list;
    }

    // For a value v, get the maximum value r that satisfies r * r <= v.
    // We maintain a value top that satisfies top * top > v.
    // We also maintain a value min_root that satisfies min_root * min_root <= v.
    // We adjust these bounds by bisection until min_root + 1 == top.
    //
    // We prove now that bisection always ends properly with min_root + 1 == top.
    // If for some k we have min_root + k == top, then bisection produces
    // middle := (2 * min_root + k) / 2 == min_root + k / 2.
    // If k >= 2, we know that the distance between 0 and k/2 and
    // the distance between k/2 and k will be strictly smaller than k and that
    // neither of these distances will be zero. These are true because k / 2 >= 1.
    // and because k - k/2 >= 1 for k >= 3.
    fn sqrt_floor(v: u32) -> u32 {
        if (v == 0) || (v == 1) {
            return v;
        }
        let mut min_root: u32 = 1;
        let mut top: u32 = min(v, TOP_ROOT_OF_U32);
        loop {
            if top == (min_root + 1) {
                return min_root;
            }
            let middle = (min_root + top) / 2;
            if middle * middle <= v {
                min_root = middle;
            } else {
                top = middle;
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::{
            get_primes,
            TOP_ROOT_OF_U32,
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
}