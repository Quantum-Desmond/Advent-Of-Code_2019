#[macro_use]
extern crate lazy_static;

extern crate itertools;
extern crate regex;

use std::time::Instant;

mod aoc_problems;

fn main() {
    let now = Instant::now();
    let result = aoc_problems::day_06::q2("./inputs/day06.txt".to_string());
    let elapsed = now.elapsed();
    println!("Answer: {:?}", result);
    println!("Elapsed time: {:?}", elapsed);
}
