use std::cmp;
use std::error::Error;
use std::fs::File;
use std::fmt;
use std::io;
use std::io::prelude::*;
use std::ops::{Add, Sub, AddAssign};
use std::result;

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};

type Result<T> = result::Result<T, Box<dyn Error>>;

type GraphEdge = (usize, HashSet<TileType>);

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
enum GraphNode {
    Start(Coordinate),
    Key(TileType)
}

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

    fn adjacent_squares(&self) -> Vec<Coordinate> {
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
    Current,
    Door(char),
    Key(char)
}

impl TileType {
    fn new(c: char) -> Result<TileType> {
        use self::TileType::*;
        match c {
            '.' => Ok(Open),
            '#' => Ok(Wall),
            '@' => Ok(Current),
            c if c.is_ascii_lowercase() => Ok(Key(c)),
            c if c.is_ascii_uppercase() => Ok(Door(c.to_lowercase().next().unwrap())),
            c => err!("Cannot read tile type = {}", c)
        }
    }
}

impl fmt::Display for TileType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::TileType::*;
        match &self {
            Open => write!(f, "."),
            Wall => write!(f, "#"),
            Current => write!(f, "@"),
            Door(c) => write!(f, "{}", c.to_uppercase()),
            Key(c) => write!(f, "{}", c.to_lowercase()),
        }
    }
}

struct Vault {
    floor_map: BTreeMap<Coordinate, TileType>,
    current_location: Coordinate,
    key_locations: HashMap<TileType, Coordinate>,
    dists: HashMap<(GraphNode, GraphNode), (usize, HashSet<TileType>)>,
    reachable_keys: HashMap<GraphNode, Vec<TileType>>
}

impl Vault {
    fn new(map_lines: Vec<Vec<char>>) -> Result<Vault> {
        let mut floor_map = BTreeMap::new();
        let mut current_location = Coordinate::new(0, 0);
        let mut key_locations: HashMap<TileType, Coordinate> = HashMap::new();
        for (y, line) in map_lines.iter().enumerate() {
            for (x, &c) in line.iter().enumerate() {
                if c == '@' {
                    current_location = Coordinate::new(x, y);
                }
                let tile_type = TileType::new(c)?;
                let coord = Coordinate::new(x, y);

                if let TileType::Key(_c) = tile_type {
                    key_locations.insert(tile_type, coord);
                }

                floor_map.insert(coord, tile_type);
            }
        }

        Ok(
            Vault {
                floor_map,
                current_location,
                key_locations,
                dists: HashMap::new(),
                reachable_keys: HashMap::new()
            }
        )
    }

    fn all_reachable_keys(&mut self, keys: &Vec<TileType>) -> Result<Vec<TileType>> {
        // let mut d = BTreeMap::new();
        // d.insert(self.current_location, 0);

        let mut queue: VecDeque<Coordinate> = VecDeque::new();
        queue.push_front(self.current_location);

        let mut todo_set: BTreeSet<Coordinate> = BTreeSet::new();
        let mut visited: BTreeSet<Coordinate> = BTreeSet::new();

        let mut reachable_keys: Vec<TileType> = vec![];

        while let Some(c) = queue.pop_front() {
            todo_set.remove(&c);
            visited.insert(c);

            for neighbour in c.adjacent_squares().into_iter().filter(|coord| self.floor_map.get(&coord) != Some(&TileType::Wall)) {
                // Don't add to squares to do
                if visited.contains(&neighbour) {
                    continue;
                }

                // Don't add square if it's a door and we don't have the right key
                if let Some(TileType::Door(c)) = self.floor_map.get(&neighbour) {
                    if !keys.contains(&TileType::Key(*c)) {
                        continue;
                    }
                }

                // Add key to reachable keys if we don't already have it
                if let Some(TileType::Key(c)) = self.floor_map.get(&neighbour) {
                    if !keys.contains(&TileType::Key(*c)) {
                        reachable_keys.push(TileType::Key(*c));
                        continue;
                    }
                }

                if !todo_set.contains(&neighbour) {
                    queue.push_back(neighbour);
                    todo_set.insert(neighbour);
                }

                // let new_dist = 1 + *d.get(&c).unwrap_or(&0);
                // if !d.contains_key(&neighbour) || new_dist < d[&neighbour] {
                //     d.insert(neighbour, new_dist);
                // }
            }
        }

        Ok(reachable_keys)
    }

    fn distance(&self, from: Coordinate, to: Coordinate) -> usize {
        let mut d = BTreeMap::new();
        d.insert(from, 0);

        let mut queue: VecDeque<Coordinate> = VecDeque::new();
        queue.push_front(from);

        let mut todo_set: BTreeSet<Coordinate> = BTreeSet::new();
        let mut visited: BTreeSet<Coordinate> = BTreeSet::new();

        while let Some(c) = queue.pop_front() {
            todo_set.remove(&c);
            visited.insert(c);

            if c == to {
                break;
            }

            for neighbour in c.adjacent_squares().into_iter().filter(|coord| self.floor_map.get(&coord) != Some(&TileType::Wall)) {
                // Don't add to squares to do
                if visited.contains(&neighbour) {
                    continue;
                }

                if let Some(TileType::Key(_c)) = self.floor_map.get(&neighbour) {
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

        d[&to]
    }

    fn keys_reachable_from(&self, pt: Coordinate) -> Result<Vec<TileType>> {
        let mut queue: VecDeque<Coordinate> = VecDeque::new();
        queue.push_front(pt);

        let mut todo_set: BTreeSet<Coordinate> = BTreeSet::new();
        let mut visited: BTreeSet<Coordinate> = BTreeSet::new();

        let mut keys: Vec<TileType> = vec![];

        while let Some(c) = queue.pop_front() {
            todo_set.remove(&c);
            visited.insert(c);

            for neighbour in c.adjacent_squares().into_iter().filter(|coord| self.floor_map.get(&coord) != Some(&TileType::Wall)) {
                // Don't add to squares to do
                if visited.contains(&neighbour) {
                    continue;
                }

                // don't walk past a key, as that's the end of the line
                if let Some(TileType::Key(c)) = self.floor_map.get(&neighbour) {
                    keys.push(TileType::Key(*c));
                    continue;
                }

                if !todo_set.contains(&neighbour) {
                    queue.push_back(neighbour);
                    todo_set.insert(neighbour);
                }
            }
        }

        Ok(keys)
    }

    fn graph_edge_for(&self, from: Coordinate, to: Coordinate) -> (usize, HashSet<TileType>) {
        let mut d = BTreeMap::new();
        d.insert(from, 0);

        let mut queue: VecDeque<Coordinate> = VecDeque::new();
        queue.push_front(from);

        let mut todo_set: BTreeSet<Coordinate> = BTreeSet::new();
        let mut visited: BTreeSet<Coordinate> = BTreeSet::new();

        while let Some(c) = queue.pop_front() {
            todo_set.remove(&c);
            visited.insert(c);

            if c == to {
                break;
            }

            for neighbour in c.adjacent_squares().into_iter().filter(|coord| self.floor_map.get(&coord) != Some(&TileType::Wall)) {
                // Don't add to squares to do
                if visited.contains(&neighbour) {
                    continue;
                }

                if let Some(TileType::Key(_c)) = self.floor_map.get(&neighbour) {
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

        let mut current_coordinate = to;
        let mut doors: HashSet<TileType> = HashSet::new();
        loop {
            let previous_coord = current_coordinate
                .adjacent_squares()
                .into_iter()
                .filter(|coord| d.contains_key(coord))
                .min_by_key(|coord| d[coord]).unwrap();

            if previous_coord == from {
                break;
            }

            if let Some(TileType::Door(c)) = self.floor_map.get(&previous_coord) {
                doors.insert(TileType::Key(*c));
            }
            current_coordinate = previous_coord;
        }

        (d[&to], doors)
    }

    fn generate_key_graph(&mut self) -> Result<()> {
        // first, add path from start to all reachable keys
        // then path from each key to all others

        // first, add everything reachable from the starting point
        let keys_from_start = self.keys_reachable_from(self.current_location)?;
        for key in &keys_from_start {
            self.dists.insert(
                (GraphNode::Start(self.current_location), GraphNode::Key(*key)),
                self.graph_edge_for(self.current_location, self.key_locations[&key])
            );
        }
        self.reachable_keys.insert(GraphNode::Start(self.current_location), keys_from_start);

        for (key, key_coordinate) in self.key_locations.iter() {
            let possible_targets = self.keys_reachable_from(*key_coordinate)?;
            for next_key in &possible_targets {
                self.dists.insert(
                    (GraphNode::Key(*key), GraphNode::Key(*next_key)),
                    self.graph_edge_for(*key_coordinate, self.key_locations[next_key])
                );
            }

            self.reachable_keys.insert(GraphNode::Key(*key), possible_targets);
        }

        Ok(())
    }

    fn path_from_to(&self, from: Coordinate, to: Coordinate) -> (usize, HashSet<TileType>) {
        let mut d = BTreeMap::new();
        d.insert(from, 0);

        let mut queue: VecDeque<Coordinate> = VecDeque::new();
        queue.push_front(from);

        let mut todo_set: BTreeSet<Coordinate> = BTreeSet::new();
        let mut visited: BTreeSet<Coordinate> = BTreeSet::new();

        while let Some(c) = queue.pop_front() {
            todo_set.remove(&c);
            visited.insert(c);

            if c == to {
                break;
            }

            for neighbour in c.adjacent_squares().into_iter().filter(|coord| self.floor_map.get(&coord) != Some(&TileType::Wall)) {
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

        // now distance has been found, backtrace to find out what doors we pass through
        let mut current_square = to;
        let mut doors: HashSet<TileType> = HashSet::new();
        loop {
            let mut min_dist = std::usize::MAX;
            for neighbour in current_square.adjacent_squares().into_iter().filter(|coord| self.floor_map.get(&coord) != Some(&TileType::Wall)) {
                if let Some(TileType::Door(c)) = self.floor_map.get(&neighbour) {
                    doors.insert(TileType::Key(*c));
                }
                if d.contains_key(&neighbour) && d[&neighbour] < min_dist {
                    min_dist = d[&neighbour];
                    current_square = neighbour;
                }
            }
            if current_square == from {
                break;
            }
        }

        (d[&to], doors)
    }

    fn total_steps_for_keys(&self, path: &Vec<TileType>) -> usize {
        let mut order = vec![self.current_location];
        order.extend(path.iter().map(|&key| self.key_locations[&key]));

        order.windows(2).map(|t| self.distance(t[0], t[1])).sum()
    }
}

impl fmt::Display for Vault {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut current_y = 0;
        for (c, tile) in self.floor_map.iter() {
            if c.y != current_y {
                write!(f, "{}", '\n')?;
                current_y = c.y;
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

    let map_lines: Vec<Vec<char>> = f_contents.trim().lines().map(|s| s.trim().chars().collect()).collect();

    _q1(map_lines).unwrap()
}

fn _q1(chars: Vec<Vec<char>>) -> Result<usize> {
    let mut vault = Vault::new(chars)?;

    vault.generate_key_graph()?;

    let mut current_path: Vec<GraphNode> = vec![];

    let mut potential_key_orderings: Vec<Vec<TileType>> = vec![vec![]];
    loop {
        let mut new_keys = false;
        let mut new_key_orderings: Vec<Vec<TileType>> = vec![];
        for key_list in &potential_key_orderings {
            let available_keys = vault.all_reachable_keys(key_list)?;
            println!("All available keys with {:?} is {:?}", key_list, available_keys);
            if !available_keys.is_empty() {
                new_keys = true;
            }
            for key in available_keys {
                let mut old_list = key_list.clone();
                old_list.push(key);
                new_key_orderings.push(old_list);
            }
        }

        if !new_keys {
            break;
        }

        potential_key_orderings = new_key_orderings;
    }

    println!("Number of potential orderings = {}", potential_key_orderings.len());

    let timings: Vec<usize> = potential_key_orderings.iter()
        .map(|path| vault.total_steps_for_keys(path))
        .collect();

    Ok(*timings.iter().min().unwrap())
}

pub fn q2(fname: String) -> usize {
    let mut f = File::open(fname).expect("File not found");
    let mut f_contents = String::new();

    f.read_to_string(&mut f_contents).expect("Couldn't find file");

    let map_lines: Vec<Vec<char>> = f_contents.trim().lines().map(|s| s.trim().chars().collect()).collect();

    _q2(map_lines).unwrap()
}

fn _q2(_chars: Vec<Vec<char>>) -> Result<usize> {
    unimplemented!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day18_q1_test1() {
        let map: Vec<Vec<char>> = "
        #########
        #b.A.@.a#
        #########
        ".trim().lines().map(|line| line.trim().chars().collect()).collect();

        assert_eq!(
            _q1(map).unwrap(),
            8
        )
    }

    #[test]
    fn day18_q1_test2() {
        let map: Vec<Vec<char>> = "
        ########################
        #f.D.E.e.C.b.A.@.a.B.c.#
        ######################.#
        #d.....................#
        ########################
        ".trim().lines().map(|line| line.trim().chars().collect()).collect();

        assert_eq!(
            _q1(map).unwrap(),
            86
        )
    }

    #[test]
    fn day18_q1_test3() {
        let map: Vec<Vec<char>> = "
        ########################
        #...............b.C.D.f#
        #.######################
        #.....@.a.B.c.d.A.e.F.g#
        ########################
        ".trim().lines().map(|line| line.trim().chars().collect()).collect();

        assert_eq!(
            _q1(map).unwrap(),
            132
        )
    }

    #[test]
    fn day18_q1_test4() {
        let map: Vec<Vec<char>> = "
        #################
        #i.G..c...e..H.p#
        ########.########
        #j.A..b...f..D.o#
        ########@########
        #k.E..a...g..B.n#
        ########.########
        #l.F..d...h..C.m#
        #################
        ".trim().lines().map(|line| line.trim().chars().collect()).collect();

        assert_eq!(
            _q1(map).unwrap(),
            136
        )
    }

    #[test]
    fn day18_q1_test5() {
        let map: Vec<Vec<char>> = "
        ########################
        #@..............ac.GI.b#
        ###d#e#f################
        ###A#B#C################
        ###g#h#i################
        ########################
        ".trim().lines().map(|line| line.trim().chars().collect()).collect();

        assert_eq!(
            _q1(map).unwrap(),
            81
        )
    }
}
