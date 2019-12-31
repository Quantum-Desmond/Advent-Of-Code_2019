use std::cmp;
use std::error::Error;
use std::fs::File;
use std::fmt;
use std::io;
use std::io::prelude::*;
use std::ops::{Add, Sub, AddAssign};
use std::result;

use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};

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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
enum TileType {
    Open,
    Wall,
    Blank,
    Portal((char, char))
}

impl fmt::Display for TileType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::TileType::*;
        match &self {
            Open => write!(f, "{}", '.'),
            Wall => write!(f, "{}", '#'),
            Blank => write!(f, "{}", ' '),
            Portal(_) => write!(f, "{}", 'O')
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PlutoMaze {
    tile_map: BTreeMap<Coordinate, TileType>,
    portal_map: HashMap<TileType, Vec<Coordinate>>,
    dist_map: BTreeMap<Coordinate, usize>,
    starting_position: Coordinate,
    end_position: Coordinate,
}

impl PlutoMaze {
    fn new(chars: Vec<Vec<char>>) -> Result<PlutoMaze> {
        println!("Size of input = {}", chars.len());
        println!("Size of line = {:?}", chars.iter().map(|line| line.len()).collect::<Vec<_>>());
        let mut tile_map: BTreeMap<Coordinate, TileType> = BTreeMap::new();
        let mut portal_map: HashMap<TileType, Vec<Coordinate>> = HashMap::new();
        for (y, line) in chars.iter().enumerate() {
            for (x, &c) in line.iter().enumerate() {
                // println!("Looking at coordinate ({}, {})", x, y);
                match c {
                    '#' => {
                        tile_map.insert(Coordinate::new(x, y), TileType::Wall);
                    },
                    '.' => {
                        if let Some(first_letter_coord) = Coordinate::new(x, y).neighbours().into_iter()
                            .filter(|coord| {
                                coord.y < chars.len()
                                    && coord.x < chars[coord.y].len()
                                    && chars[coord.y][coord.x].is_ascii_uppercase()
                        }).next() {
                            // this is a portal square
                            let second_letter_coord = first_letter_coord
                                .neighbours()
                                .into_iter()
                                .filter(|coord| {
                                    coord.y < chars.len()
                                        && coord.x < chars[coord.y].len()
                                        && chars[coord.y][coord.x].is_ascii_uppercase()
                                }).next()
                                .ok_or("Cannot find second portal character")?;

                            let mut letter_coords = vec![first_letter_coord, second_letter_coord];
                            letter_coords.sort();

                            let (first_char, second_char) = (chars[letter_coords[0].y][letter_coords[0].x], chars[letter_coords[1].y][letter_coords[1].x]);

                            tile_map.insert(Coordinate::new(x, y), TileType::Portal((first_char, second_char)));
                            let portal_entry = portal_map.entry(TileType::Portal((first_char, second_char))).or_insert(vec![]);
                            portal_entry.push(Coordinate::new(x, y));
                        } else {
                            tile_map.insert(Coordinate::new(x, y), TileType::Open);
                        }
                    },
                    cc if cc.is_ascii_uppercase() => {
                        tile_map.insert(Coordinate::new(x, y), TileType::Blank);
                    },
                    ' ' => {
                        tile_map.insert(Coordinate::new(x, y), TileType::Blank);
                    },
                    c => return err!("Cannot read character: {}", c)
                }
            }
        }

        let starting_position: Coordinate = portal_map[&TileType::Portal(('A', 'A'))]
            .first()
            .ok_or("Cannot find starting point in maze")?
            .clone();

        let end_position: Coordinate = portal_map[&TileType::Portal(('Z', 'Z'))]
            .first()
            .ok_or("Cannot find finishing point in maze")?
            .clone();

        Ok(
            PlutoMaze {
                tile_map,
                portal_map,
                dist_map: BTreeMap::new(),
                starting_position,
                end_position
            }
        )
    }

    fn adjacent_tiles(&self, coord: Coordinate) -> Result<Vec<Coordinate>> {
        let neighbours: Vec<Coordinate> = match self.tile_map.get(&coord).ok_or(format!("Cannot find {} in tile map", coord))? {
            TileType::Open => {
                coord.neighbours()
                    .into_iter()
                    .filter(|coord| self.tile_map.get(&coord) != Some(&TileType::Wall) && self.tile_map.get(&coord) != Some(&TileType::Blank))
                    .collect()
            },
            TileType::Portal(char_tuple) => {
                let mut neighbours: Vec<_> = coord.neighbours()
                    .into_iter()
                    .filter(|coord| self.tile_map.get(&coord) != Some(&TileType::Wall) && self.tile_map.get(&coord) != Some(&TileType::Blank))
                    .collect();
                if *char_tuple != ('A', 'A') && *char_tuple != ('Z', 'Z') {
                    let other_portal_coord: Coordinate = self.portal_map.get(&TileType::Portal(*char_tuple))
                        .ok_or(format!("Cannot find portal for {:?}", char_tuple))?
                        .iter()
                        .filter(|&&cc| cc != coord)
                        .next()
                        .ok_or(format!("Cannot find other portal coordinate for {}", coord))?
                        .clone();
                    neighbours.push(other_portal_coord);
                }
                neighbours
            },
            tile_type => return err!("Cannot find neighbours of type {:?}", tile_type),
        };

        Ok(neighbours)
    }

    fn find_path_through_maze(&mut self) -> Result<()> {
        self.dist_map.insert(self.starting_position, 0);

        let mut queue: VecDeque<Coordinate> = VecDeque::new();
        queue.push_front(self.starting_position);

        let mut todo_set: BTreeSet<Coordinate> = BTreeSet::new();
        let mut visited: BTreeSet<Coordinate> = BTreeSet::new();

        while let Some(c) = queue.pop_front() {
            todo_set.remove(&c);
            visited.insert(c);

            if c == self.end_position {
                break;
            }

            for neighbour in self.adjacent_tiles(c)? {
                // Don't add to squares to do
                if visited.contains(&neighbour) {
                    continue;
                }

                if !todo_set.contains(&neighbour) {
                    queue.push_back(neighbour);
                    todo_set.insert(neighbour);
                }

                let new_dist = 1 + *self.dist_map.get(&c).unwrap_or(&0);
                if !self.dist_map.contains_key(&neighbour) || new_dist < self.dist_map[&neighbour] {
                    self.dist_map.insert(neighbour, new_dist);
                }
            }
        }
        Ok(())
    }

    fn shortest_path_through_maze(&self) -> Result<usize> {
        let dist = self.dist_map.get(&self.end_position).ok_or("End position not in distance map")?;

        Ok(*dist)
    }
}

impl fmt::Display for PlutoMaze {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut current_y = 0;
        for (coord, tile) in self.tile_map.iter() {
            if coord.y != current_y {
                writeln!(f)?;
                current_y = coord.y;
            }
            write!(f, "{}", tile)?;
        }

        Ok(())
    }
}

pub fn q1(fname: String) -> usize {
    let mut f = File::open(fname).expect("File not found");
    let mut f_contents = String::new();

    f.read_to_string(&mut f_contents).expect("Couldn't find file");

    let chars: Vec<Vec<char>> = f_contents.split('\n').filter(|s| s.len() > 0).map(|s| s.chars().collect()).collect();

    _q1(chars).unwrap()
}

fn _q1(chars: Vec<Vec<char>>) -> Result<usize> {
    println!("Started Q1 calculation");
    let mut maze = PlutoMaze::new(chars)?;
    println!("{}", maze);
    println!("Created maze object");
    maze.find_path_through_maze()?;
    maze.shortest_path_through_maze()
}

pub fn q2(fname: String) -> usize {
    let mut f = File::open(fname).expect("File not found");
    let mut f_contents = String::new();

    f.read_to_string(&mut f_contents).expect("Couldn't find file");

    let map_char_list: Vec<Vec<char>> = f_contents.trim().split('\n').map(|s| s.trim().chars().collect()).collect();

    _q2(map_char_list).unwrap()
}

fn _q2(_chars: Vec<Vec<char>>) -> Result<usize> {
    unimplemented!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day20_q1_test1() {
        let map: Vec<Vec<char>> = "
         A           
         A           
  #######.#########  
  #######.........#  
  #######.#######.#  
  #######.#######.#  
  #######.#######.#  
  #####  B    ###.#  
BC...##  C    ###.#  
  ##.##       ###.#  
  ##...DE  F  ###.#  
  #####    G  ###.#  
  #########.#####.#  
DE..#######...###.#  
  #.#########.###.#  
FG..#########.....#  
  ###########.#####  
             Z       
             Z       
        ".split('\n').filter(|line| line.len() > 0).map(|line| line.chars().collect()).collect();

        assert_eq!(
            _q1(map).unwrap(),
            23
        )
    }

    #[test]
    fn day20_q1_test2() {
        let map: Vec<Vec<char>> = "
                   A               
                   A               
  #################.#############  
  #.#...#...................#.#.#  
  #.#.#.###.###.###.#########.#.#  
  #.#.#.......#...#.....#.#.#...#  
  #.#########.###.#####.#.#.###.#  
  #.............#.#.....#.......#  
  ###.###########.###.#####.#.#.#  
  #.....#        A   C    #.#.#.#  
  #######        S   P    #####.#  
  #.#...#                 #......VT
  #.#.#.#                 #.#####  
  #...#.#               YN....#.#  
  #.###.#                 #####.#  
DI....#.#                 #.....#  
  #####.#                 #.###.#  
ZZ......#               QG....#..AS
  ###.###                 #######  
JO..#.#.#                 #.....#  
  #.#.#.#                 ###.#.#  
  #...#..DI             BU....#..LF
  #####.#                 #.#####  
YN......#               VT..#....QG
  #.###.#                 #.###.#  
  #.#...#                 #.....#  
  ###.###    J L     J    #.#.###  
  #.....#    O F     P    #.#...#  
  #.###.#####.#.#####.#####.###.#  
  #...#.#.#...#.....#.....#.#...#  
  #.#####.###.###.#.#.#########.#  
  #...#.#.....#...#.#.#.#.....#.#  
  #.###.#####.###.###.#.#.#######  
  #.#.........#...#.............#  
  #########.###.###.#############  
           B   J   C               
           U   P   P               
        ".split('\n').filter(|line| line.len() > 0).map(|line| line.chars().collect()).collect();

        assert_eq!(
            _q1(map).unwrap(),
            58
        )
    }
}
