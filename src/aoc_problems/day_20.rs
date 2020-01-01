use std::cmp;
use std::error::Error;
use std::fs::File;
use std::fmt;
use std::io;
use std::io::prelude::*;
use std::ops::{Add, Sub, AddAssign};
use std::result;

use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};

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
    inside_portals: BTreeSet<Coordinate>,
    outside_portals: BTreeSet<Coordinate>,
    starting_position: Coordinate,
    end_position: Coordinate,
    end_distance: usize
}

impl PlutoMaze {
    fn new(chars: Vec<Vec<char>>) -> Result<PlutoMaze> {
        println!("Size of input = {}", chars.len());
        println!("Size of line = {:?}", chars.iter().map(|line| line.len()).collect::<Vec<_>>());
        let mut tile_map: BTreeMap<Coordinate, TileType> = BTreeMap::new();
        let mut portal_map: HashMap<TileType, Vec<Coordinate>> = HashMap::new();
        let mut inside_portals: BTreeSet<Coordinate> = BTreeSet::new();
        let mut outside_portals: BTreeSet<Coordinate> = BTreeSet::new();
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

                            // second letter coord is on the outside of the map if outside portal
                            let is_outside_portal = vec![0, chars.len()-1].contains(&second_letter_coord.y)
                                || vec![0, chars[second_letter_coord.y].len()-1].contains(&second_letter_coord.x);

                            let mut letter_coords = vec![first_letter_coord, second_letter_coord];
                            letter_coords.sort();

                            let char_tuple = (chars[letter_coords[0].y][letter_coords[0].x], chars[letter_coords[1].y][letter_coords[1].x]);
                            if is_outside_portal {
                                outside_portals.insert(Coordinate::new(x, y));
                            } else {
                                inside_portals.insert(Coordinate::new(x, y));
                            }

                            let mut letter_coords = vec![first_letter_coord, second_letter_coord];
                            letter_coords.sort();

                            let char_tuple = (chars[letter_coords[0].y][letter_coords[0].x], chars[letter_coords[1].y][letter_coords[1].x]);

                            tile_map.insert(Coordinate::new(x, y), TileType::Portal(char_tuple));
                            let portal_entry = portal_map.entry(TileType::Portal(char_tuple)).or_insert(vec![]);
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
                outside_portals,
                inside_portals,
                starting_position,
                end_position,
                end_distance: 0
            }
        )
    }

    fn adjacent_tiles(&self, position: (Coordinate, usize), recursive: bool) -> Result<Vec<(Coordinate, usize)>> {
        let (coord, recursion_level) = position;
        let neighbours: Vec<(Coordinate, usize)> = match self.tile_map.get(&coord).ok_or(format!("Cannot find {} in tile map", coord))? {
            TileType::Open => {
                coord.neighbours()
                    .into_iter()
                    .filter_map(|coord| {
                        if self.tile_map.get(&coord) != Some(&TileType::Wall) && self.tile_map.get(&coord) != Some(&TileType::Blank) {
                            Some((coord, recursion_level))
                        } else {
                            None
                        }
                    })
                    .collect()
            },
            TileType::Portal(char_tuple) => {
                let mut neighbours = coord.neighbours()
                    .into_iter()
                    .filter_map(|coord| {
                        if self.tile_map.get(&coord) != Some(&TileType::Wall) && self.tile_map.get(&coord) != Some(&TileType::Blank) {
                            Some((coord, recursion_level))
                        } else {
                            None
                        }
                    })
                    .collect_vec();
                if recursive {
                    // if level = 0, only outside portals = AA, ZZ
                    // otherwise,
                    //     - inside goes to outside with recursion level + 1
                    //     - outside goes to inside with recursion level - 1
                    if self.inside_portals.contains(&coord) {
                        let other_portal_coord: Coordinate = self.portal_map.get(&TileType::Portal(*char_tuple))
                            .ok_or(format!("Cannot find portal for {:?}", char_tuple))?
                            .iter()
                            .filter(|&&cc| cc != coord)
                            .next()
                            .ok_or(format!("Cannot find other portal coordinate for inside portal {}", coord))?
                            .clone();
                        neighbours.push((other_portal_coord, recursion_level+1));
                    } else if self.outside_portals.contains(&coord) {
                        if recursion_level != 0 && *char_tuple != ('A', 'A') && *char_tuple != ('Z', 'Z') {
                            let other_portal_coord: Coordinate = self.portal_map.get(&TileType::Portal(*char_tuple))
                                .ok_or(format!("Cannot find portal for {:?}", char_tuple))?
                                .iter()
                                .filter(|&&cc| cc != coord)
                                .next()
                                .ok_or(format!("Cannot find other portal coordinate for outside portal {}", coord))?
                                .clone();
                            neighbours.push((other_portal_coord, recursion_level-1));
                        }
                    } else {
                        return err!("{} is neither inside or outside portal", coord);
                    }
                } else {
                    if *char_tuple != ('A', 'A') && *char_tuple != ('Z', 'Z') {
                        let other_portal_coord: Coordinate = self.portal_map.get(&TileType::Portal(*char_tuple))
                            .ok_or(format!("Cannot find portal for {:?}", char_tuple))?
                            .iter()
                            .filter(|&&cc| cc != coord)
                            .next()
                            .ok_or(format!("Cannot find other portal coordinate for {}", coord))?
                            .clone();
                        neighbours.push((other_portal_coord, recursion_level));
                    }
                }

                neighbours
            },
            tile_type => return err!("Cannot find neighbours of type {:?}", tile_type),
        };

        Ok(neighbours)
        // let neighbours: Vec<Coordinate> = match self.tile_map.get(&coord).ok_or(format!("Cannot find {} in tile map", coord))? {
        //     TileType::Open => {
        //         coord.neighbours()
        //             .into_iter()
        //             .filter(|coord| {
        //                 if recursive {

        //                 } else {
        //                     self.tile_map.get(&coord) != Some(&TileType::Wall) && self.tile_map.get(&coord) != Some(&TileType::Blank)
        //                 }
        //             })
        //             .collect()
        //     },
        //     TileType::Portal(char_tuple) => {
        //         let mut neighbours: Vec<_> = coord.neighbours()
        //             .into_iter()
        //             .filter(|coord| self.tile_map.get(&coord) != Some(&TileType::Wall) && self.tile_map.get(&coord) != Some(&TileType::Blank))
        //             .collect();
        //         if *char_tuple != ('A', 'A') && *char_tuple != ('Z', 'Z') {
        //             let other_portal_coord: Coordinate = self.portal_map.get(&TileType::Portal(*char_tuple))
        //                 .ok_or(format!("Cannot find portal for {:?}", char_tuple))?
        //                 .iter()
        //                 .filter(|&&cc| cc != coord)
        //                 .next()
        //                 .ok_or(format!("Cannot find other portal coordinate for {}", coord))?
        //                 .clone();
        //             neighbours.push(other_portal_coord);
        //         }
        //         neighbours
        //     },
        //     tile_type => return err!("Cannot find neighbours of type {:?}", tile_type),
        // };

        // Ok(neighbours)
    }

    fn find_path_through_maze(&mut self, recursive: bool) -> Result<()> {
        let mut d = BTreeMap::new();
        d.insert((self.starting_position, 0), 0);

        let mut queue: VecDeque<(Coordinate, usize)> = VecDeque::new();
        queue.push_front((self.starting_position, 0));

        let mut todo_set: BTreeSet<(Coordinate, usize)> = BTreeSet::new();
        let mut visited: BTreeSet<(Coordinate, usize)> = BTreeSet::new();

        while let Some(c) = queue.pop_front() {
            todo_set.remove(&c);
            visited.insert(c);

            if c == (self.end_position, 0) {
                break;
            }

            for neighbour in self.adjacent_tiles(c, recursive)? {
                // Don't add to squares to do
                if visited.contains(&neighbour) {
                    continue;
                }

                if !todo_set.contains(&neighbour) {
                    queue.push_back(neighbour);
                    todo_set.insert(neighbour);
                }

                let new_dist = 1 + *d.get(&c).unwrap_or(&0);
                if !d.contains_key(&neighbour) || new_dist < d[&neighbour] {
                    d.insert(neighbour, new_dist);
                }
            }
        }

        self.end_distance = *d.get(&(self.end_position, 0)).ok_or("End position not in distance map")?;

        Ok(())
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
    maze.find_path_through_maze(false)?;
    Ok(maze.end_distance)
}

pub fn q2(fname: String) -> usize {
    let mut f = File::open(fname).expect("File not found");
    let mut f_contents = String::new();

    f.read_to_string(&mut f_contents).expect("Couldn't find file");

    let chars: Vec<Vec<char>> = f_contents.split('\n').filter(|s| s.len() > 0).map(|s| s.chars().collect()).collect();

    _q2(chars).unwrap()
}

fn _q2(chars: Vec<Vec<char>>) -> Result<usize> {
    let mut maze = PlutoMaze::new(chars)?;
    println!("Created maze object");
    maze.find_path_through_maze(true)?;
    Ok(maze.end_distance)
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

    #[test]
    fn day20_q2_test() {
        let map: Vec<Vec<char>> = "
             Z L X W       C                 
             Z P Q B       K                 
  ###########.#.#.#.#######.###############  
  #...#.......#.#.......#.#.......#.#.#...#  
  ###.#.#.#.#.#.#.#.###.#.#.#######.#.#.###  
  #.#...#.#.#...#.#.#...#...#...#.#.......#  
  #.###.#######.###.###.#.###.###.#.#######  
  #...#.......#.#...#...#.............#...#  
  #.#########.#######.#.#######.#######.###  
  #...#.#    F       R I       Z    #.#.#.#  
  #.###.#    D       E C       H    #.#.#.#  
  #.#...#                           #...#.#  
  #.###.#                           #.###.#  
  #.#....OA                       WB..#.#..ZH
  #.###.#                           #.#.#.#  
CJ......#                           #.....#  
  #######                           #######  
  #.#....CK                         #......IC
  #.###.#                           #.###.#  
  #.....#                           #...#.#  
  ###.###                           #.#.#.#  
XF....#.#                         RF..#.#.#  
  #####.#                           #######  
  #......CJ                       NM..#...#  
  ###.#.#                           #.###.#  
RE....#.#                           #......RF
  ###.###        X   X       L      #.#.#.#  
  #.....#        F   Q       P      #.#.#.#  
  ###.###########.###.#######.#########.###  
  #.....#...#.....#.......#...#.....#.#...#  
  #####.#.###.#######.#######.###.###.#.#.#  
  #.......#.......#.#.#.#.#...#...#...#.#.#  
  #####.###.#####.#.#.#.#.###.###.#.###.###  
  #.......#.....#.#...#...............#...#  
  #############.#.#.###.###################  
               A O F   N                     
               A A D   M                     
        ".split('\n').filter(|&line| line.trim().len() > 0).map(|line| line.chars().collect()).collect();

        assert_eq!(
            _q2(map).unwrap(),
            396
        )
    }
}
