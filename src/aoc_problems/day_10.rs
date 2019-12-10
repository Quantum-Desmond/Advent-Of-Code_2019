use std::cmp;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::ops::{Add, Sub, AddAssign};
use std::result;

use std::collections::BTreeMap;

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
    x: u32,
    y: u32
}

impl Coordinate {
    fn new(x: u32, y: u32) -> Coordinate {
        Coordinate { x, y }
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
enum SpaceType {
    Empty,
    Asteroid
}

impl SpaceType {
    fn new(c: char) -> SpaceType {
        use self::SpaceType::*;
        match c {
            '.' => Empty,
            '#' => Asteroid,
            c => panic!("Cannot decipher character: {}", c),
        }
    }
}

impl fmt::Display for SpaceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::SpaceType::*;
        match self {
            Empty => write!(f, "."),
            Asteroid => write!(f, "#"),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
struct AsteroidField {
    map: BTreeMap<Coordinate, SpaceType>
}

impl AsteroidField {
    fn new(pixels: Vec<Vec<char>>, width: u32, height: u32) -> Result<AsteroidField> {
        unimplemented!();
    }
}

impl fmt::Display for AsteroidField {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut current_y = 0;
        for (coord, square) in self.map.iter() {
            if coord.y != current_y {
                write!(f, "{}", '\n')?;
                current_y = coord.y;
            }
            write!(f, "{}", square)?;
        }

        Ok(())
    }
}

pub fn q1(fname: String) -> usize {
    let mut f = File::open(fname).expect("File not found");
    let mut f_contents = String::new();

    f.read_to_string(&mut f_contents).expect("Couldn't find file");
    let asteroid_data: Vec<Vec<char>> = f_contents.trim().lines().map(|x| {
        x.chars().collect()
    }).collect();

    _q1(asteroid_data).unwrap()
}

fn _q1(mut pixels: Vec<Vec<char>>) -> Result<usize> {
    // Tactic for this is find mow many of the lines of sight for the asteroids dy/dx
    // are distinct

    unimplemented!();
}

pub fn q2(fname: String) -> usize {
    let mut f = File::open(fname).expect("File not found");
    let mut f_contents = String::new();

    f.read_to_string(&mut f_contents).expect("Couldn't find file");
    let asteroid_data: Vec<Vec<char>> = f_contents.trim().lines().map(|x| {
        x.chars().collect()
    }).collect();

    _q2(asteroid_data).unwrap()
}

fn _q2(mut pixels: Vec<Vec<char>>) -> Result<usize> {
    unimplemented!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day10_q1_test1() {
        let asteroid_data : Vec<Vec<char>> = "
            .#..#
            .....
            #####
            ....#
            ...##
        ".trim().lines().map(|l| l.trim().chars().collect()).collect();

        assert_eq!(
            _q1(asteroid_data).unwrap(),
            8
        )
    }

    #[test]
    fn day10_q1_test2() {
        let asteroid_data : Vec<Vec<char>> = "
            ......#.#.
            #..#.#....
            ..#######.
            .#.#.###..
            .#..#.....
            ..#....#.#
            #..#....#.
            .##.#..###
            ##...#..#.
            .#....####
        ".trim().lines().map(|l| l.trim().chars().collect()).collect();

        assert_eq!(
            _q1(asteroid_data).unwrap(),
           33
        )
    }

    #[test]
    fn day10_q1_test3() {
        let asteroid_data : Vec<Vec<char>> = "
            #.#...#.#.
            .###....#.
            .#....#...
            ##.#.#.#.#
            ....#.#.#.
            .##..###.#
            ..#...##..
            ..##....##
            ......#...
            .####.###.
        ".trim().lines().map(|l| l.trim().chars().collect()).collect();

        assert_eq!(
            _q1(asteroid_data).unwrap(),
           35
        )
    }

    #[test]
    fn day10_q1_test4() {
        let asteroid_data : Vec<Vec<char>> = "
            .#..#..###
            ####.###.#
            ....###.#.
            ..###.##.#
            ##.##.#.#.
            ....###..#
            ..#.#..#.#
            #..#.#.###
            .##...##.#
            .....#.#..
        ".trim().lines().map(|l| l.trim().chars().collect()).collect();

        assert_eq!(
            _q1(asteroid_data).unwrap(),
           41
        )
    }

    #[test]
    fn day10_q1_test5() {
        let asteroid_data : Vec<Vec<char>> = "
            .#..##.###...#######
            ##.############..##.
            .#.######.########.#
            .###.#######.####.#.
            #####.##.#.##.###.##
            ..#####..#.#########
            ####################
            #.####....###.#.#.##
            ##.#################
            #####.##.###..####..
            ..######..##.#######
            ####.##.####...##..#
            .#####..#.######.###
            ##...#.##########...
            #.##########.#######
            .####.#.###.###.#.##
            ....##.##.###..#####
            .#.#.###########.###
            #.#.#.#####.####.###
            ###.##.####.##.#..##
        ".trim().lines().map(|l| l.trim().chars().collect()).collect();

        assert_eq!(
            _q1(asteroid_data).unwrap(),
           210
        )
    }
}
