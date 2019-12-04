#[macro_use]
extern crate lazy_static;

extern crate regex;
extern crate itertools;

use std::time::Instant;

mod aoc_problems;

fn main() {
    let now = Instant::now();
    let result = aoc_problems::day_04::q2(240920, 789857);
    // let result = aoc_problems::day_04::q1("./inputs/day03.txt".to_string());
    let elapsed = now.elapsed();
    println!("Answer: {:?}", result);
    println!("Elapsed time: {:?}", elapsed);
}
