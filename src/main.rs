include!(concat!(env!("OUT_DIR"), "/primes.rs"));

mod generation;

pub fn main() {
    println!("{}", PRIMES[2]);
}
