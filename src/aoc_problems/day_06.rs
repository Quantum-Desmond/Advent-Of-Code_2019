use std::error::Error;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::result;
use std::thread;
use std::time;

use std::collections::{HashMap, HashSet};

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

struct Orbits {
    orbit_count: HashMap<String, usize>,
    orbit_map: HashMap<String, HashSet<String>>,
    full_orbit_graph: HashMap<String, HashSet<String>>,
    visited_nodes: HashSet<String>,
    dist_map: HashMap<String, usize>
}

impl Orbits {
    fn new(orbit_list: Vec<String>) -> Result<Orbits> {
        let orbit_re: Regex = Regex::new(r"(?P<orbitee>[a-zA-Z0-9]+)\)(?P<orbiter>[a-zA-Z0-9]+)")?;

        let orbit_map: HashMap<String, HashSet<String>> = orbit_list.iter().fold(HashMap::new(), |mut acc, line| {
            if let Some(m) = orbit_re.captures(&line) {
                let _orbiter = acc.entry(m["orbiter"].to_string()).or_insert(HashSet::new());
                let orbiters = acc.entry(m["orbitee"].to_string()).or_insert(HashSet::new());
                orbiters.insert(m["orbiter"].to_string());
            } else {
                panic!("Should match line!");
            }

            acc
        });

        let full_orbit_graph: HashMap<String, HashSet<String>> = orbit_list.iter().fold(HashMap::new(), |mut acc, line| {
            if let Some(m) = orbit_re.captures(&line) {
                let orbitee_links = acc.entry(m["orbiter"].to_string()).or_insert(HashSet::new());
                orbitee_links.insert(m["orbitee"].to_string());

                let orbiters = acc.entry(m["orbitee"].to_string()).or_insert(HashSet::new());
                orbiters.insert(m["orbiter"].to_string());
            } else {
                panic!("Should match line!");
            }

            acc
        });

        Ok(Orbits {
            orbit_map,
            orbit_count: HashMap::new(),
            full_orbit_graph,
            visited_nodes: HashSet::new(),
            dist_map: HashMap::new(),
        })
    }

    fn orbit_sum_of(&mut self, obj: &String) -> usize {
        if self.orbit_count.contains_key(obj) {
            *self.orbit_count.get(obj).unwrap()
        } else {
            let orbiters: HashSet<String> = self.orbit_map.get(obj).unwrap().clone();
            let number_of_orbits = orbiters.iter().map(|s| 1 + self.orbit_sum_of(s)).sum();
            self.orbit_count.insert(obj.clone(), number_of_orbits);
            number_of_orbits
        }
    }

    fn orbit_sum(&mut self) -> Result<usize> {
        let orbiters: Vec<String> = self.orbit_map.keys().map(|orbitee| {
            orbitee.clone()
        }).collect();
        let total_orbits = orbiters.into_iter().map(|orbitee| {
            self.orbit_sum_of(&orbitee)
        }).sum();
        Ok(total_orbits)
    }

    fn parent_of(&self, obj: &String) -> Result<String> {
        let source_parents: Vec<String> = self.orbit_map.iter().filter_map(|(k, v)| {
            if v.contains(obj) {
                Some(k.clone())
            } else {
                None
            }
        }).collect();

        let parent = source_parents.get(0).ok_or("No parents found".to_string())?;
        Ok(parent.clone())
    }

    fn __dfs(&mut self, v: &String, dist: usize, target: &String) -> Option<usize> {
        if v == target {
            return Some(self.dist_map[&target.clone()]);
        }
        self.visited_nodes.insert(v.clone());

        let edges: Vec<String> = self.full_orbit_graph.get(v)
                                     .unwrap()
                                     .iter()
                                     .map(|s| s.clone())
                                     .collect();

        for edge in edges {
            if !self.visited_nodes.contains(&edge) {
                let new_dist = 1 + dist;
                if !self.dist_map.contains_key(&edge) || new_dist < self.dist_map[&edge] {
                    self.dist_map.insert(edge.clone(), new_dist);
                }

                if edge == *target {
                    println!("{:?}", self.dist_map);
                    return Some(self.dist_map[&target.clone()]);
                }
                if let Some(dist) = self.__dfs(&edge, new_dist, &target) {
                    return Some(dist);
                }
            }
        }

        None
    }

    fn shortest_path_from(&mut self, source: String, target: String) -> Result<usize> {
        let source_parent = self.parent_of(&source)?;
        let target_parent = self.parent_of(&target)?;

        println!("Parent of YOU = {}", source_parent);
        println!("Parent of SAN = {}", target_parent);

        let result = self.__dfs(&source_parent, 0, &target_parent);

        println!("Result = {:?}", result);

        Ok(result.unwrap())
    }
}

pub fn q1(fname: String) -> usize {
    let mut f = File::open(fname).expect("File not found");
    let mut f_contents = String::new();

    f.read_to_string(&mut f_contents).expect("Couldn't find file");
    let orbits: Vec<String> = f_contents.trim().lines().map(|x: &str| {
        x.trim().to_string()
    }).collect();

    _q1(orbits).unwrap()
}

fn _q1(orbits: Vec<String>) -> Result<usize> {
    let mut orbit_info = Orbits::new(orbits)?;
    let orbit_count = orbit_info.orbit_sum()?;

    Ok(orbit_count)
}

pub fn q2(fname: String) -> usize {
    let mut f = File::open(fname).expect("File not found");
    let mut f_contents = String::new();

    f.read_to_string(&mut f_contents).expect("Couldn't find file");
    let orbits: Vec<String> = f_contents.trim().lines().map(|x: &str| {
        x.trim().to_string()
    }).collect();

    _q2(orbits).unwrap()
}

fn _q2(orbits: Vec<String>) -> Result<usize> {
    let mut orbit_info = Orbits::new(orbits)?;

    orbit_info.shortest_path_from("YOU".to_string(), "SAN".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day06_q1_tests() {
        let orbits_str: Vec<String> = "
            COM)B
            B)C
            C)D
            D)E
            E)F
            B)G
            G)H
            D)I
            E)J
            J)K
            K)L
            ".to_string().trim().lines().map(|x: &str| {
                x.to_string()
            }).collect();
        assert_eq!(
            _q1(orbits_str).unwrap(),
            42
        )
    }

    #[test]
    fn day06_q2_test() {
        let orbits_str: Vec<String> = "
            COM)B
            B)C
            C)D
            D)E
            E)F
            B)G
            G)H
            D)I
            E)J
            J)K
            K)L
            K)YOU
            I)SAN
            ".to_string().trim().lines().map(|x: &str| {
                x.to_string()
            }).collect();
        assert_eq!(
            _q2(orbits_str).unwrap(),
            4
        )
    }
}
