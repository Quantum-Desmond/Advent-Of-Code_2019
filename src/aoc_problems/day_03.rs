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

#[derive(Clone, Copy, Default, Eq, PartialEq, Hash)]
struct Coordinate {
    x: i32,
    y: i32
}

impl Coordinate {
    fn new(x: i32, y: i32) -> Coordinate {
        Coordinate { x, y }
    }

    fn distance_from(&self, other: Self) -> usize {
        (
            (self.x - other.x).abs()
                + (self.y - other.y).abs()
        ) as usize
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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT
}

impl Direction {
    fn to_coord(&self) -> Coordinate {
        use self::Direction::*;
        match &self {
            UP => Coordinate::new(0, 1),
            DOWN => Coordinate::new(0, -1),
            LEFT => Coordinate::new(-1, 0),
            RIGHT => Coordinate::new(1, 0),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct Displacement {
    dir: Direction,
    dist: usize
}

impl Displacement {
    fn new(displacement_str: String) -> Result<Displacement> {
        use self::Direction::*;
        let displacement_chars: Vec<char> = displacement_str.chars().collect();
        let dir = match displacement_chars[0] {
            'U' => UP,
            'D' => DOWN,
            'L' => LEFT,
            'R' => RIGHT,
            _ => return err!("Cannot parse input direction!")
        };

        let dist = displacement_chars[1..].iter().collect::<String>().parse()?;

        Ok(Displacement { dir, dist })
    }
}

struct WireGrid {
    grid: BTreeMap<Coordinate, HashMap<usize, usize>>
}

impl WireGrid {
    fn new() -> WireGrid {
        WireGrid { grid: BTreeMap::new() }
    }

    fn add_wire(&mut self, wire_str: String, marker: usize) -> Result<()> {
        let wire_displacements: Result<Vec<Displacement>> = wire_str.split(',').map(|x: &str| {
            Displacement::new(x.to_string())
        }).collect();
        let wire_displacements = wire_displacements?;

        let mut current_position = Coordinate::new(0, 0);
        let mut steps: usize = 0;
        for displacement in wire_displacements {
            let unit_displacement = displacement.dir.to_coord();
            for _ in 0..displacement.dist {
                steps += 1;
                current_position += unit_displacement;
                let wire_count = self.grid.entry(current_position).or_insert(HashMap::new());
                if !wire_count.contains_key(&marker) {
                    wire_count.insert(marker, steps);
                }
            }
        }

        Ok(())
    }
}

pub fn q1(fname: String) -> usize {
    let mut f = File::open(fname).expect("File not found");
    let mut f_contents = String::new();

    f.read_to_string(&mut f_contents).expect("Couldn't find file");
    let wires: Vec<String> = f_contents.trim().lines().map(|x: &str| {
        x.to_string()
    }).collect();

    let wire_1: String = wires[0].clone();
    let wire_2: String = wires[1].clone();

    _q1(wire_1, wire_2).unwrap()
}

fn _q1(wire_str_1: String, wire_str_2: String) -> Result<usize> {
    let mut wire_grid = WireGrid::new();

    wire_grid.add_wire(wire_str_1, 1)?;
    wire_grid.add_wire(wire_str_2, 2)?;

    let min_dist = wire_grid.grid.iter()
        .filter(|(_, n)| n.len() > 1)
        .map(|(coord, _)| coord.distance_from(Coordinate::new(0, 0)))
        .min()
        .ok_or("No elements in wire grid!")?;

    Ok(min_dist)
}

pub fn q2(fname: String) -> usize {
    let mut f = File::open(fname).expect("File not found");
    let mut f_contents = String::new();

    f.read_to_string(&mut f_contents).expect("Couldn't find file");
    let wires: Vec<String> = f_contents.trim().lines().map(|x: &str| {
        x.to_string()
    }).collect();

    let wire_1: String = wires[0].clone();
    let wire_2: String = wires[1].clone();

    _q2(wire_1, wire_2).unwrap()
}

fn _q2(wire_str_1: String, wire_str_2: String) -> Result<usize> {
    let mut wire_grid = WireGrid::new();

    wire_grid.add_wire(wire_str_1, 1)?;
    wire_grid.add_wire(wire_str_2, 2)?;

    let min_dist = wire_grid.grid.iter()
        .filter(|(_, n)| n.len() > 1)
        .map(|(_, step_count)| step_count.values().sum())
        .min()
        .ok_or("No elements in wire grid!")?;

    Ok(min_dist)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day03_q1_tests() {
        assert_eq!(
            _q1(
                "R75,D30,R83,U83,L12,D49,R71,U7,L72".to_string(),
                "U62,R66,U55,R34,D71,R55,D58,R83".to_string()
            ).unwrap(),
            159
        );

        assert_eq!(
            _q1(
                "R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51".to_string(),
                "U98,R91,D20,R16,D67,R40,U7,R15,U6,R7".to_string()
            ).unwrap(),
            135
        );
    }

    #[test]
    fn day03_q2_tests() {
        assert_eq!(
            _q2(
                "R75,D30,R83,U83,L12,D49,R71,U7,L72".to_string(),
                "U62,R66,U55,R34,D71,R55,D58,R83".to_string()
            ).unwrap(),
            610
        );

        assert_eq!(
            _q2(
                "R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51".to_string(),
                "U98,R91,D20,R16,D67,R40,U7,R15,U6,R7".to_string()
            ).unwrap(),
            410
        );
    }
}
