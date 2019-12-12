use std::cmp;
use std::error::Error;
use std::fs::File;
use std::fmt;
use std::io;
use std::io::prelude::*;
use std::ops::{Add, Sub, AddAssign};
use std::result;
use std::str::FromStr;

use std::collections::BTreeMap;

use regex::Regex;

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

#[derive(Clone, Copy, Default, Eq, PartialEq, Hash)]
struct Coordinate {
    x: i32,
    y: i32,
    z: i32
}

impl Coordinate {
    fn new(x: i32, y: i32, z: i32) -> Coordinate {
        Coordinate { x, y, z }
    }
}

impl Ord for Coordinate {
    fn cmp(&self, other: &Coordinate) -> cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialOrd for Coordinate {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some((self.z, self.y, self.x).cmp(&(other.z, other.y, other.x)))
    }
}

impl Add for Coordinate {
    type Output = Coordinate;

    fn add(self, other: Coordinate) -> Coordinate {
        Coordinate {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Sub for Coordinate {
    type Output = Coordinate;

    fn sub(self, other: Coordinate) -> Coordinate {
        Coordinate {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.y - other.z,
        }
    }
}

impl AddAssign for Coordinate {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl FromStr for Coordinate {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self> {
        lazy_static! {
            static ref COORD_RE: Regex = Regex::new(
                r"<x=(?P<x>-?[0-9]+), y=(?P<y>-?[0-9]+), z=(?P<z>-?[0-9]+)>"
            ).unwrap();
        }

        if !s.is_ascii() {
            return err!("area must be in ASCII");
        }

        if s.lines().count() != 1 {
            return err!("Only accepts 1 line");
        }

        if let Some(caps) = COORD_RE.captures(s) {
            return Ok(
                Coordinate::new(
                    caps["x"].parse()?,
                    caps["y"].parse()?,
                    caps["z"].parse()?
                )
            );
        }

        err!("Cannot parse coordinate line: {}", s)
    }
}

impl fmt::Debug for Coordinate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

impl fmt::Display for Coordinate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

struct Moon {
    position: Coordinate,
    velocity: Coordinate
}

impl Moon {
    fn new(position: Coordinate) -> Moon {
        Moon {
            position,
            velocity: Coordinate::new(0, 0, 0)
        }
    }

    fn potential_energy(&self) -> usize {
        [
            self.position.x,
            self.position.y,
            self.position.z,

        ].iter().map(|n| n.abs()).sum() as usize
    }

    fn kinetic_energy(&self) -> usize {
        [
            self.velocity.x,
            self.velocity.y,
            self.velocity.z,

        ].iter().map(|n| n.abs()).sum() as usize
    }

    fn total_energy(&self) -> usize {
        self.potential_energy() * self.kinetic_energy()
    }
}

struct Jupiter{
    moon_positions: Vec<Moon>
}

impl Jupiter {
    fn new(coords: Vec<Coordinate>) -> Jupiter {
        Jupiter {
            moon_positions: coords.into_iter().map(|coord| Moon::new(coord)).collect()
        }
    }
}

pub fn q1(fname: String) -> usize {
    let mut f = File::open(fname).expect("File not found");
    let mut f_contents = String::new();

    f.read_to_string(&mut f_contents).expect("Couldn't find file");

    let coords: Vec<String> = f_contents.trim().lines().map(|line| line.trim().to_string()).collect();

    _q1(coords).unwrap()
}

fn _q1(coords: Vec<String>) -> Result<usize> {
    let moons: Result<Vec<Coordinate>> = coords.iter().map(|line| line.parse()).collect();
    let moons = moons?;

    let jupiter = Jupiter::new(moons);

    unimplemented!();
}

pub fn q2(fname: String) -> String {
    let mut f = File::open(fname).expect("File not found");
    let mut f_contents = String::new();

    f.read_to_string(&mut f_contents).expect("Couldn't find file");

    let memory: Vec<i64> = f_contents.trim().split(',').map(|s| s.parse().unwrap()).collect();

    _q2(memory).unwrap()
}

fn _q2(memory: Vec<i64>) -> Result<String> {
    unimplemented!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day12_q1_test1() {
        let moon_coords: Vec<String> = "
            <x=-1, y=0, z=2>
            <x=2, y=-10, z=-7>
            <x=4, y=-8, z=8>
            <x=3, y=5, z=-1>
        ".lines().map(|line| line.trim().to_string()).collect();

        assert_eq!(
            _q1(moon_coords).unwrap(),
            179
        )
    }

    #[test]
    fn day12_q1_test2() {
        let moon_coords: Vec<String> = "
            <x=-8, y=-10, z=0>
            <x=5, y=5, z=10>
            <x=2, y=-7, z=3>
            <x=9, y=-8, z=-3>
        ".lines().map(|line| line.trim().to_string()).collect();

        assert_eq!(
            _q1(moon_coords).unwrap(),
            1940
        )
    }
}