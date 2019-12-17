use std::cmp;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::ops::{Add, Sub, AddAssign};
use std::result;

use std::collections::{BTreeMap, HashMap, HashSet};

type Result<T> = result::Result<T, Box<dyn Error>>;

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

    fn is_asteroid(&self) -> bool {
        use self::SpaceType::*;
        match self {
            Empty => false,
            Asteroid => true
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

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
struct LineOfSight {
    x: i32,
    y: i32,
}

impl LineOfSight {
    fn new(x: i32, y: i32) -> LineOfSight {
        let input_gcd = gcd(x, y);

        LineOfSight {
            x: x / input_gcd,
            y: y / input_gcd,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
struct AsteroidField {
    map: BTreeMap<Coordinate, SpaceType>
}

impl AsteroidField {
    fn new(asteroid_data: Vec<Vec<char>>) -> Result<AsteroidField> {
        let mut asteroid_map = BTreeMap::new();

        for (y, line) in asteroid_data.iter().enumerate() {
            for (x, &c) in line.iter().enumerate() {
                asteroid_map.insert(
                    Coordinate::new(x as u32, y as u32),
                    SpaceType::new(c)
                );
            }
        }

        Ok(
            AsteroidField {
                map: asteroid_map
            }
        )
    }

    fn most_visible_asteroid(&self) -> (Coordinate, usize) {
        let asteroid_coords: Vec<Coordinate> = self.map
            .iter()
            .filter_map(|(&c, square)| {
                    if square.is_asteroid() {
                        Some(c)
                    } else { None }
                }
            ).collect();

        let visible_asteroids: Vec<(Coordinate, usize)> = asteroid_coords
            .iter()
            .map(|&c|
                (c, number_of_visible_coords(c, asteroid_coords.clone()))
            ).collect();

        *visible_asteroids.iter().max_by_key(|t| t.1).unwrap()
    }
}

fn number_of_visible_coords(current_coord: Coordinate, coords: Vec<Coordinate>) -> usize {
    let mut lines_of_sight: HashSet<LineOfSight> = HashSet::new();

    for coord in coords {
        if current_coord == coord {
            continue;
        }

        lines_of_sight.insert(
            LineOfSight::new(
                (coord.y as i32) - (current_coord.y as i32),
                (coord.x as i32) - (current_coord.x as i32),
            )
        );
    }

    lines_of_sight.len()
}

fn line_of_sight_info(current_coord: Coordinate, coords: Vec<Coordinate>) -> HashMap<LineOfSight, Coordinate> {
    let mut lines_of_sight: HashMap<LineOfSight, Coordinate> = HashMap::new();

    for coord in coords {
        if current_coord == coord {
            continue;
        }

        let dy = (coord.y as i32) - (current_coord.y as i32);
        let dx = (coord.x as i32) - (current_coord.x as i32);
        let line_of_sight_info = LineOfSight::new(dy, dx);

        // if line of sight hasn't already been seen, or if this line of sight is closer
        // than already added
        if !lines_of_sight.contains_key(&line_of_sight_info)
            || (dx.abs() < (lines_of_sight[&line_of_sight_info].x as i32 - current_coord.x as i32).abs())
        {
            lines_of_sight.insert(
                LineOfSight::new(
                    (coord.y as i32) - (current_coord.y as i32),
                    (coord.x as i32) - (current_coord.x as i32),
                ),
                coord
            );

        }

    }

    lines_of_sight
}

fn gcd(m: i32, n: i32) -> i32 {
    if m == 0 {
        n.abs()
    } else {
        gcd(n % m, m)
    }
}

fn gradient_of(line_of_sight: LineOfSight) -> f32 {
    match (line_of_sight.x, line_of_sight.y) {
        (x, 0) if x > 0 => std::f32::INFINITY,
        (x, 0) if x < 0 => std::f32::NEG_INFINITY,
        (x, y) => (x as f32) / (y as f32)
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

fn _q1(asteroid_data: Vec<Vec<char>>) -> Result<usize> {
    // Tactic for this is find mow many of the lines of sight for the asteroids dy/dx
    // are distinct
    let asteroid_field = AsteroidField::new(asteroid_data)?;

    let most_visible_asteroid = asteroid_field.most_visible_asteroid();

    println!("Most visible asteroid = {:?}", most_visible_asteroid);

    Ok(most_visible_asteroid.1)
}

pub fn q2(fname: String) -> u32 {
    let mut f = File::open(fname).expect("File not found");
    let mut f_contents = String::new();

    f.read_to_string(&mut f_contents).expect("Couldn't find file");
    let asteroid_data: Vec<Vec<char>> = f_contents.trim().lines().map(|x| {
        x.chars().collect()
    }).collect();

    _q2(asteroid_data).unwrap()
}

fn _q2(asteroid_data: Vec<Vec<char>>) -> Result<u32> {
    let asteroid_field = AsteroidField::new(asteroid_data)?;

    let station_coord = asteroid_field.most_visible_asteroid().0;

    let asteroid_coords: Vec<Coordinate> = asteroid_field.map.iter().filter_map(|(&c, square)| {
            if square.is_asteroid() && (station_coord != c) {
                Some(c)
            } else { None }
        }
    ).collect();

    // Don't have to worry about the points behind, as I only need to count up to 200
    // and from part 1, there are > 300 in sight
    let line_of_sight_info = line_of_sight_info(station_coord, asteroid_coords);

    // Going clockwise: split into 2 parts
    // Right of station: blow up in decreasing gradient
    // Left of station: blow up in increasing gradient
    let mut first_half: Vec<_> = line_of_sight_info.iter().filter_map(|(&line_of_sight, &coord)| {
        if (coord.x > station_coord.x)
            || (coord.x == station_coord.x) && (coord.y > station_coord.y)
        {
            Some((coord, gradient_of(line_of_sight)))
        } else { None }
    }).collect();

    // reverse sorting
    first_half.sort_by(|t1, t2| t2.1.partial_cmp(&t1.1).unwrap());

    let mut second_half: Vec<_> = line_of_sight_info.iter().filter_map(|(&line_of_sight, &coord)| {
        if (coord.x < station_coord.x)
            || (coord.x == station_coord.x) && (coord.y < station_coord.y)
        {
            Some((coord, gradient_of(line_of_sight)))
        } else { None }
    }).collect();
    second_half.sort_by(|t1, t2| t1.1.partial_cmp(&t2.1).unwrap());

    first_half.extend(second_half);

    // Looking for 200th
    let relevant_coord: Coordinate = first_half[199].0;

    println!("200th coordinate = {}", relevant_coord);

    Ok((relevant_coord.x * 100) + relevant_coord.y)
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

    #[test]
    fn day10_q2_test() {
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
            _q2(asteroid_data).unwrap(),
           802
        )
    }
}
