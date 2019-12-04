use std::cmp;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::ops::{Add, Sub, AddAssign};
use std::result;

use std::collections::{BTreeMap, HashSet, HashMap};

type Result<T> = result::Result<T, Box<dyn Error>>;

macro_rules! err {
    ($($tt:tt)*) => { Err(Box::<dyn Error>::from(format!($($tt)*))) }
}

fn pause() {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    // We want the cursor to stay at the end of the line, so we print without a newline and flush manually.
    write!(stdout, "Press any key to continue...").unwrap();
    stdout.flush().unwrap();

    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
}

fn matching_adjacent_digits(digit_list: &Vec<u32>) -> bool {
    for t in digit_list.windows(2) {
        if t[0] == t[1] {
            return true;
        }
    }
    false
}

fn matching_only_2_digits(digit_list: &Vec<u32>) -> bool {
    let mut digit_count: HashMap<u32, u32> = HashMap::new();

    for &digit in digit_list {
        let count = digit_count.entry(digit).or_insert(0);
        *count += 1;
    }

    digit_count.values().any(|&x| x == 2)
}

fn ordered_digits(digit_list: &Vec<u32>) -> bool {
    let mut ordered = true;
    for t in digit_list.windows(2) {
        if t[0] > t[1] {
            ordered = false;
        }
    }

    ordered
}

fn fits_password_criteria(n: u32) -> bool {
    let digit_list = n.to_string();
    let digit_list: Vec<_> = digit_list.chars().map(|d| d.to_digit(10).unwrap()).collect();

    matching_adjacent_digits(&digit_list) && ordered_digits(&digit_list)
}

fn fits_full_password_criteria(n: u32) -> bool {
    let digit_list = n.to_string();
    let digit_list: Vec<_> = digit_list.chars().map(|d| d.to_digit(10).unwrap()).collect();

    matching_only_2_digits(&digit_list) && ordered_digits(&digit_list)
}

pub fn q1(start: u32, finish: u32) -> usize {
    (start..finish+1).filter(|&n| fits_password_criteria(n)).count()
}

pub fn q2(start: u32, finish: u32) -> usize {
    (start..finish+1).filter(|&n| fits_full_password_criteria(n)).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day04_q1_tests() {
        assert_eq!(
            fits_password_criteria(111111),
            true
        );
        assert_eq!(
            fits_password_criteria(223450),
            false
        );
        assert_eq!(
            fits_password_criteria(123789),
            false
        );
    }

    #[test]
    fn day04_q2_tests() {
        assert_eq!(
            fits_full_password_criteria(112233),
            true
        );
        assert_eq!(
            fits_full_password_criteria(123444),
            false
        );
        assert_eq!(
            fits_full_password_criteria(111122),
            true
        );
    }
}
