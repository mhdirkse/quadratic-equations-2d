fn main() {
    let s: Vec<String> = get_primes::get_primes(25).iter()
        .map(|p| p.to_string())
        .collect::<Vec<String>>();
    println!("{}", format_table::make_table(&s, ", ", 4, ",\n"));
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
    // multituples.

    use std::cmp::min;

    const TOP_ROOT_OF_U32: u32 = (1 as u32) << 16;

    pub fn get_primes(prime_list_upper_bound: u32) -> Vec<u32> {
        if prime_list_upper_bound <= 1 {
            return vec![];
        }
        let mut prime_list: Vec<u32> = (2 ..= prime_list_upper_bound).collect();
        let max_prime = sqrt_floor(prime_list_upper_bound);
        let mut prime_index = 0;
        loop {
            let prime = prime_list[prime_index];
            if prime > max_prime {
                break;
            }
            prime_list.retain(|e| ! ((e % prime == 0) && (e / prime >= 2)));
            prime_index += 1;
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
    // middle := (2 * min_root + k) / 2 == min_root + k / 2 where / is rounding-down
    // integer division.
    //
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
        while top != (min_root + 1) {
            let middle = (min_root + top) / 2;
            if middle * middle <= v {
                min_root = middle;
            } else {
                top = middle;
            }
        }
        return min_root;
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

mod format_table {
    pub fn make_table(v: &Vec<String>, field_delim: &str, num_per_line: u32, line_delim: &str) -> String {
        if num_per_line <= 1 {
            panic!("Using make_table only makes sense if there is more than one item per line, got {}", num_per_line);
        }
        let mut result = String::new();
        let len = v.len() as u32;
        for i in 0 .. len {
            result.push_str(&v[i as usize]);
            if i < (len - 1) {
                if i % num_per_line == (num_per_line - 1) {
                    result.push_str(line_delim);
                } else {
                    result.push_str(field_delim);
                }
            }
        }
        return result;
    }

    #[cfg(test)]
    mod test {
        use super::make_table;

        #[test]
        fn empty() {
            let input: Vec<String> = vec![];
            assert_eq!(String::from(""), make_table(&input, ", ", 2, "\n"));
        }

        #[test]
        fn one() {
            let input: Vec<String> = vec![String::from("item")];
            assert_eq!(String::from("item"), make_table(&input, ", ", 2, "\n"));
        }

        #[test]
        fn two_on_one_line() {
            let input: Vec<String> = vec![String::from("first"), String::from("second")];
            assert_eq!("first, second", make_table(&input, ", ", 2, "\n"));
        }

        #[test]
        fn two_lines() {
            let input: Vec<String> = vec![String::from("first"), String::from("second"), String::from("third")];
            assert_eq!("first, second\nthird", make_table(&input, ", ", 2, "\n"));
        }

        #[test]
        fn three_per_line() {
            let input: Vec<String> = vec![String::from("first"), String::from("second"), String::from("third"), String::from("fourth")];
            assert_eq!("first, second, third\nfourth", make_table(&input, ", ", 3, "\n"));
        }
    }
}