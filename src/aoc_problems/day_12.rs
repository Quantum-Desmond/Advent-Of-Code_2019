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

#[derive(Clone, Copy, Default, Eq, PartialEq, Hash)]
struct Moon1D {
    position: i32,
    velocity: i32
}

impl Moon1D {
    fn new(position: i32) -> Moon1D {
        Moon1D {
            position,
            velocity: 0
        }
    }

    fn gravity_from(&self, other: &Moon1D) -> i32 {
        (other.position - self.position).signum()
    }
}

impl fmt::Display for Moon1D {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Pos: {}, velocity: {}", self.position, self.velocity)
    }
}

struct Jupiter1D {
    moons: Vec<Moon1D>
}

impl Jupiter1D {
    fn new(coords: Vec<i32>) -> Jupiter1D {
        Jupiter1D {
            moons: coords.into_iter().map(|coord| Moon1D::new(coord)).collect()
        }
    }

    fn add_gravity_effect(&mut self) -> Result<()> {
        let other_moons = self.moons.clone();

        for moon in &mut self.moons {
            for other_moon in &other_moons {
                moon.velocity += moon.gravity_from(&other_moon);
            }
        }
        Ok(())
    }

    fn move_moons(&mut self) -> Result<()> {
        for moon in &mut self.moons {
            moon.position += moon.velocity;
        }

        Ok(())
    }

    fn increment_time(&mut self) -> Result<()> {
        self.add_gravity_effect()?;
        self.move_moons()?;

        Ok(())
    }

    fn period(&mut self) -> Result<usize> {
        let initial_positions: Vec<i32> = self.moons.iter().map(|&moon| moon.position).collect();

        let mut t = 0;
        loop {
            t += 1;
            self.increment_time()?;

            let current_positions: Vec<i32> = self.moons.iter().map(|&moon| moon.position).collect();
            if current_positions == initial_positions {
                break;
            }
        }

        Ok(t+1)
    }
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Hash)]
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

    fn potential_energy(&self) -> i32 {
        [
            self.position.x,
            self.position.y,
            self.position.z,

        ].iter().map(|n| n.abs()).sum()
    }

    fn kinetic_energy(&self) -> i32 {
        [
            self.velocity.x,
            self.velocity.y,
            self.velocity.z,

        ].iter().map(|n| n.abs()).sum()
    }

    fn total_energy(&self) -> usize {
        (self.potential_energy() * self.kinetic_energy()) as usize
    }

    fn gravity_from(&self, other: &Moon) -> Coordinate {
        Coordinate::new(
            (other.position.x - self.position.x).signum(),
            (other.position.y - self.position.y).signum(),
            (other.position.z - self.position.z).signum(),
        )
    }
}

impl fmt::Display for Moon {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Pos: {}, velocity: {}", self.position, self.velocity)
    }
}

struct Jupiter {
    moons: Vec<Moon>
}

impl Jupiter {
    fn new(coords: Vec<Coordinate>) -> Jupiter {
        Jupiter {
            moons: coords.into_iter().map(|coord| Moon::new(coord)).collect()
        }
    }

    fn add_gravity_effect(&mut self) -> Result<()> {
        let other_moons = self.moons.clone();

        for moon in &mut self.moons {
            for other_moon in &other_moons {
                moon.velocity += moon.gravity_from(&other_moon);
            }
        }
        Ok(())
    }

    fn move_moons(&mut self) -> Result<()> {
        for moon in &mut self.moons {
            moon.position += moon.velocity;
        }

        Ok(())
    }

    fn increment_time(&mut self) -> Result<()> {
        self.add_gravity_effect()?;
        self.move_moons()?;

        Ok(())
    }

    fn total_energy(&self) -> usize {
        self.moons.iter().map(|moon| moon.total_energy()).sum()
    }
}

impl fmt::Display for Jupiter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (idx, moon) in self.moons.iter().enumerate() {
            writeln!(f, "Moon {} -  {}", idx, moon)?;
        }
        Ok(())
    }
}

fn gcd(m: usize, n: usize) -> usize {
    if m == 0 {
        n
    } else {
        gcd(n % m, m)
    }
}

fn lcm(m: usize, n: usize) -> usize {
    m * n / gcd(m, n)
}

pub fn q1(fname: String) -> usize {
    let mut f = File::open(fname).expect("File not found");
    let mut f_contents = String::new();

    f.read_to_string(&mut f_contents).expect("Couldn't find file");

    let coords: Vec<String> = f_contents.trim().lines().map(|line| line.trim().to_string()).collect();

    match _q1(coords, 1000) {
        Ok(result) => result,
        Err(e) => {
            println!("Error: {}", e);
            panic!();
        }
    }
}

fn period_1d(initial_vals: Vec<i32>) -> Result<usize> {
    let mut jupiter_1d = Jupiter1D::new(initial_vals);

    jupiter_1d.period()
}

fn _q1(coords: Vec<String>, t: usize) -> Result<usize> {
    let moons: Result<Vec<Coordinate>> = coords.iter().map(|line| line.parse()).collect();
    let moons = moons?;

    let mut jupiter = Jupiter::new(moons);

    for _ in 0..t {
        jupiter.increment_time()?;
    }

    Ok(jupiter.total_energy())
}

pub fn q2(fname: String) -> usize {
    let mut f = File::open(fname).expect("File not found");
    let mut f_contents = String::new();

    f.read_to_string(&mut f_contents).expect("Couldn't find file");

    let coords: Vec<String> = f_contents.trim().lines().map(|line| line.trim().to_string()).collect();

    match _q2(coords) {
        Ok(result) => result,
        Err(e) => {
            println!("Error: {}", e);
            panic!();
        }
    }
}

fn _q2(coords: Vec<String>) -> Result<usize> {
    // Consider using a hashset (the obvious path)
    // but consider looking for the same setup as before, but with some permuation on the moons
    // using this, it might be quicker to iterate based on that step size
    let moons: Result<Vec<Coordinate>> = coords.iter().map(|line| line.parse()).collect();
    let moons = moons?;

    let initial_x: Vec<i32> = moons.iter().map(|&moon| moon.x).collect();
    let initial_y: Vec<i32> = moons.iter().map(|&moon| moon.y).collect();
    let initial_z: Vec<i32> = moons.iter().map(|&moon| moon.z).collect();

    let x_period = period_1d(initial_x)?;
    println!("x period is {}", x_period);
    let y_period = period_1d(initial_y)?;
    println!("y period is {}", y_period);
    let z_period = period_1d(initial_z)?;
    println!("z period is {}", z_period);

    Ok(lcm(x_period, lcm(y_period, z_period)))
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
        ".trim().lines().map(|line| line.trim().to_string()).collect();

        assert_eq!(
            _q1(moon_coords, 10).unwrap(),
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
        ".trim().lines().map(|line| line.trim().to_string()).collect();

        assert_eq!(
            _q1(moon_coords, 100).unwrap(),
            1940
        )
    }

    #[test]
    fn day12_q2_test1() {
        let moon_coords: Vec<String> = "
            <x=-1, y=0, z=2>
            <x=2, y=-10, z=-7>
            <x=4, y=-8, z=8>
            <x=3, y=5, z=-1>
        ".trim().lines().map(|line| line.trim().to_string()).collect();

        assert_eq!(
            _q2(moon_coords).unwrap(),
            2772
        )
    }

    #[test]
    fn day12_q2_test2() {
        let moon_coords: Vec<String> = "
            <x=-8, y=-10, z=0>
            <x=5, y=5, z=10>
            <x=2, y=-7, z=3>
            <x=9, y=-8, z=-3>
        ".trim().lines().map(|line| line.trim().to_string()).collect();

        assert_eq!(
            _q2(moon_coords).unwrap(),
            4686774924
        )
    }
}