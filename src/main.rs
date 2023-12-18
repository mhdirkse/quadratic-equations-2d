#[macro_use] extern crate lalrpop_util;

lalrpop_mod!(pub lang);
// Added only to have the unit tests
mod generation;
mod primes;
mod expression;
mod parser;
mod sqrt;

pub fn main() {
    println!("{}", lang::TermParser::new().parse("(5)").is_ok());
}
