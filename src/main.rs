extern crate itertools;
extern crate regex;

use std::time::Instant;

mod aoc_problems;

fn main() {
    let now = Instant::now();
    let result = aoc_problems::day_09::q1("./inputs/day09.txt".to_string());
    let elapsed = now.elapsed();
    println!("Answer: {:?}", result);
    println!("Elapsed time: {:?}", elapsed);
}
