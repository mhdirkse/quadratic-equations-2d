use std::env;
use std::fs;
use std::path::Path;

include!("src/generation.rs");

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("primes.rs");
    fs::write(&dest_path, get_primes_program()).unwrap();
    println!("cargo:rerun-if-changed=build.rs");
}
