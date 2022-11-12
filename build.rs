extern crate lalrpop;

include!("src/generation.rs");

use get_primes::TOP_ROOT_OF_U32;

fn main() {
    write_primes_program(25, "primes25");
    write_primes_program(TOP_ROOT_OF_U32, "primes");
    println!("cargo:rerun-if-changed=build.rs");
    lalrpop::process_root().unwrap();
}
