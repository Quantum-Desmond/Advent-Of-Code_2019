use std::collections::HashSet;

use std::fs::File;
use std::io::prelude::*;

fn fuel_needed_for(mass: usize) -> usize {
    if mass <= 8 {
        0
    } else {
        (mass / 3) - 2
    }
}

fn total_fuel_requirement_for(mass: usize) -> usize {
    let mut total_fuel = fuel_needed_for(mass);
    let mut fuel_mass = total_fuel;
    loop {
        fuel_mass = fuel_needed_for(fuel_mass);
        if fuel_mass <= 0 {
            break;
        }
        total_fuel += fuel_mass;
    }
    total_fuel
}

pub fn q1(fname: String) -> usize {
    let mut f = File::open(fname).expect("File not found");
    let mut f_contents = String::new();

    f.read_to_string(&mut f_contents).expect("Couldn't find file");
    let masses: Vec<usize> = f_contents.trim()
        .lines()
        .map(|l| l.trim().parse().unwrap())
        .collect();

    masses.iter().map(|&mass| fuel_needed_for(mass)).sum()
}

pub fn q2(fname: String) -> usize {
    let mut f = File::open(fname).expect("File not found");
    let mut f_contents = String::new();

    f.read_to_string(&mut f_contents).expect("Couldn't find file");
    let masses: Vec<usize> = f_contents.trim()
        .lines()
        .map(|l| l.trim().parse().unwrap())
        .collect();

    masses.iter().map(|&mass| total_fuel_requirement_for(mass)).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day01_fuel1() {
        assert_eq!(
            fuel_needed_for(12), 2
        );
    }

    #[test]
    fn day01_fuel2() {
        assert_eq!(
            fuel_needed_for(14), 2
        );
    }

    #[test]
    fn day01_fuel3() {
        assert_eq!(
            fuel_needed_for(1969), 654
        );
    }

    #[test]
    fn day01_fuel4() {
        assert_eq!(
            fuel_needed_for(100756), 33583
        );
    }

    #[test]
    fn day01_q2() {
        assert_eq!(
            total_fuel_requirement_for(100756), 50346
        );
    }
}
