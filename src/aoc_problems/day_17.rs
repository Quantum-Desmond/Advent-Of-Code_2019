use std::cmp;
use std::error::Error;
use std::fs::File;
use std::fmt;
use std::io;
use std::io::prelude::*;
use std::ops::{Add, Sub, AddAssign};
use std::result;

use std::collections::{BTreeMap, VecDeque};

use itertools::Itertools;

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
    x: usize,
    y: usize
}

impl Coordinate {
    fn new(x: usize, y: usize) -> Coordinate {
        Coordinate { x, y }
    }

    fn neighbours(&self) -> Vec<Coordinate> {
        let mut result = vec![
            Coordinate::new(self.x, self.y + 1),
            Coordinate::new(self.x + 1, self.y),
        ];
        if self.x > 0 {
            result.push(Coordinate::new(self.x - 1, self.y));
        }

        if self.y > 0 {
            result.push(Coordinate::new(self.x, self.y - 1));
        }

        result
    }
}

impl Ord for Coordinate {
    fn cmp(&self, other: &Coordinate) -> cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialOrd for Coordinate {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some((self.y, self.x).cmp(&(other.y, other.x)))
    }
}

impl Add for Coordinate {
    type Output = Coordinate;

    fn add(self, other: Coordinate) -> Coordinate {
        Coordinate {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Coordinate {
    type Output = Coordinate;

    fn sub(self, other: Coordinate) -> Coordinate {
        Coordinate {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl AddAssign for Coordinate {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl fmt::Debug for Coordinate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl fmt::Display for Coordinate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}


pub fn q1(fname: String) -> usize {
    let mut f = File::open(fname).expect("File not found");
    let mut f_contents = String::new();

    f.read_to_string(&mut f_contents).expect("Couldn't find file");

    let map_char_list: Vec<Vec<char>> = f_contents.trim().split('\n').map(|s| s.trim().chars().collect()).collect();

    _q1(map_char_list).unwrap()
}

fn _q1(chars: Vec<Vec<char>>) -> Result<usize> {
    unimplemented!();
}

pub fn q2(fname: String) -> usize {
    let mut f = File::open(fname).expect("File not found");
    let mut f_contents = String::new();

    f.read_to_string(&mut f_contents).expect("Couldn't find file");

    let map_char_list: Vec<Vec<char>> = f_contents.trim().split('\n').map(|s| s.trim().chars().collect()).collect();

    _q2(map_char_list).unwrap()
}

fn _q2(chars: Vec<Vec<char>>) -> Result<usize> {
    unimplemented!();
}

