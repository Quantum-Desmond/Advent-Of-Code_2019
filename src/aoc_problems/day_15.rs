use std::cmp;
use std::error::Error;
use std::fs::File;
use std::fmt;
use std::io;
use std::io::prelude::*;
use std::ops::{Add, Sub, AddAssign};
use std::result;

use std::collections::{BTreeMap, BTreeSet, VecDeque};

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

    fn neighbours(&self) -> Vec<Coordinate> {
        vec![
            Coordinate::new(self.x, self.y + 1),
            Coordinate::new(self.x - 1, self.y),
            Coordinate::new(self.x + 1, self.y),
            Coordinate::new(self.x, self.y - 1)
        ]
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

#[derive(Clone, Copy, Eq, Debug, PartialEq, Hash)]
enum Parameter {
    Position,
    Immediate,
    Relative
}

#[derive(Clone, Eq, Default, Debug, PartialEq, Hash)]
struct Instruction {
    opcode: usize,
    parameters: Vec<Parameter>
}

impl Instruction {
    fn new(number: usize) -> Result<Instruction> {
        let opcode = number % 100;
        let mut digit_list: Vec<_> = (number / 100).to_string().chars().map(|d| d.to_digit(10).unwrap()).collect();
        digit_list.reverse();

        let params_length = match opcode {
            1 => 3,
            2 => 3,
            3 => 1,
            4 => 1,
            5 => 2,
            6 => 2,
            7 => 3,
            8 => 3,
            9 => 1,
            99 => 0,
            x => return err!("{}", format!("Cannot read opcode: {}", x))
        };

        digit_list.resize(params_length, 0);
        let parameters: Result<Vec<Parameter>> = digit_list.into_iter().map(|d| match d {
            0 => Ok(Parameter::Position),
            1 => Ok(Parameter::Immediate),
            2 => Ok(Parameter::Relative),
            x => err!("{}", format!("Cannot read parameter digit: {}", x))
        }).collect();
        let parameters = parameters?;

        Ok(
            Instruction {
                opcode,
                parameters,
            }
        )

    }
}

struct Program {
    memory: Vec<i64>,
    input: i64,
    current_input: usize,
    pointer_idx: usize,
    relative_base: i64,
}

impl Program {
    fn new(memory: Vec<i64>)  -> Program {
        Program {
            memory,
            input: 0,
            current_input: 1,
            pointer_idx: 0,
            relative_base: 0,
        }
    }

    fn get_input(&mut self) -> Result<i64> {
        Ok(self.input as i64)
    }

    fn set_input(&mut self, input: i64) {
        self.input = input;
    }

    fn get_parameter(&mut self, parameter_form: Parameter, val: i64) -> i64 {
        use self::Parameter::*;

        match parameter_form {
            Position => {
                let idx = val as usize;
                if self.memory.len() < idx+1 {
                    self.memory.resize(idx+1, 0);
                }

                self.memory[idx]
            },
            Immediate => val,
            Relative => {
                let idx = (self.relative_base + val) as usize;
                if self.memory.len() < idx+1 {
                    self.memory.resize(idx+1, 0);
                }

                self.memory[idx]
            }
        }
    }

    fn set_parameter(&mut self, idx: usize, val: i64) -> Result<()> {
        if self.memory.len() < idx+1 {
            self.memory.resize(idx+1, 0);
        }

        self.memory[idx] = val;

        Ok(())
    }

    fn get_output_idx(&mut self, idx: usize, parameter_type: Parameter) -> usize {
        use self::Parameter::*;
        if self.memory.len() < idx+1 {
            self.memory.resize(idx+1, 0);
        }
        match parameter_type {
            Position => {
                self.memory[idx] as usize
            },
            Relative => {
                (self.memory[idx] + self.relative_base) as usize
            },
            _ => panic!("Should never be here")
        }
    }

    fn run_program(&mut self) -> Result<Option<i64>> {
        loop {
            let current_instruction = Instruction::new(self.memory[self.pointer_idx] as usize)?;

            match current_instruction.opcode {
                1 => {
                    let input_1 = self.get_parameter(
                        current_instruction.parameters[0],
                        self.memory[self.pointer_idx+1],
                    );
                    let input_2 = self.get_parameter(
                        current_instruction.parameters[1],
                        self.memory[self.pointer_idx+2],
                    );
                    let output_idx = self.get_output_idx(
                        self.pointer_idx + 3,
                        current_instruction.parameters[2]
                    );
                    self.set_parameter(output_idx, input_1 + input_2)?;

                    self.pointer_idx += 4;
                },
                2 => {
                    let input_1 = self.get_parameter(
                        current_instruction.parameters[0],
                        self.memory[self.pointer_idx+1],
                    );
                    let input_2 = self.get_parameter(
                        current_instruction.parameters[1],
                        self.memory[self.pointer_idx+2],
                    );
                    let output_idx = self.get_output_idx(
                        self.pointer_idx + 3,
                        current_instruction.parameters[2]
                    );
                    self.set_parameter(output_idx, input_1 * input_2)?;

                    self.pointer_idx += 4;
                },
                3 => {
                    let output_idx = self.get_output_idx(
                        self.pointer_idx + 1,
                        current_instruction.parameters[0]
                    );
                    let input = self.get_input()?;
                    self.set_parameter(output_idx, input)?;

                    self.pointer_idx += 2;
                },
                4 => {
                    let output_val = self.get_parameter(
                        current_instruction.parameters[0],
                        self.memory[self.pointer_idx+1]
                    );

                    // let output_idx = self.memory[self.pointer_idx+1];
                    self.pointer_idx += 2;

                    return Ok(Some(output_val));
                },
                5 => {
                    let input_1 = self.get_parameter(
                        current_instruction.parameters[0],
                        self.memory[self.pointer_idx+1],
                    );
                    let input_2 = self.get_parameter(
                        current_instruction.parameters[1],
                        self.memory[self.pointer_idx+2],
                    );
                    if input_1 != 0 {
                        self.pointer_idx = input_2 as usize;
                    } else {
                        self.pointer_idx += 3;
                    }
                },
                6 => {
                    let input_1 = self.get_parameter(
                        current_instruction.parameters[0],
                        self.memory[self.pointer_idx+1],
                    );
                    let input_2 = self.get_parameter(
                        current_instruction.parameters[1],
                        self.memory[self.pointer_idx+2],
                    );
                    if input_1 == 0 {
                        self.pointer_idx = input_2 as usize;
                    } else {
                        self.pointer_idx += 3;
                    }
                },
                7 => {
                    let input_1 = self.get_parameter(
                        current_instruction.parameters[0],
                        self.memory[self.pointer_idx+1],
                    );
                    let input_2 = self.get_parameter(
                        current_instruction.parameters[1],
                        self.memory[self.pointer_idx+2],
                    );
                    let output_idx = self.get_output_idx(
                        self.pointer_idx + 3,
                        current_instruction.parameters[2]
                    );
                    self.set_parameter(output_idx, if input_1 < input_2 {1} else {0})?;

                    self.pointer_idx += 4;
                },
                8 => {
                    let input_1 = self.get_parameter(
                        current_instruction.parameters[0],
                        self.memory[self.pointer_idx+1],
                    );
                    let input_2 = self.get_parameter(
                        current_instruction.parameters[1],
                        self.memory[self.pointer_idx+2],
                    );
                    let output_idx = self.get_output_idx(
                        self.pointer_idx + 3,
                        current_instruction.parameters[2]
                    );
                    self.set_parameter(output_idx, if input_1 == input_2 {1} else {0})?;

                    self.pointer_idx += 4;
                },
                9 => {
                    let input_1 = self.get_parameter(
                        current_instruction.parameters[0],
                        self.memory[self.pointer_idx+1],
                    );
                    self.relative_base += input_1;

                    self.pointer_idx += 2;
                },
                99 => break,
                x => return err!("{}", format!("Incorrect opcode: {}", x))
            }
        }
        Ok(None)
    }
}

#[derive(Clone, Copy, Eq, Debug, PartialEq, Hash)]
enum SquareType {
    Wall,
    Open,
    System
}

impl fmt::Display for SquareType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::SquareType::*;
        match &self {
            Wall => write!(f, "#"),
            Open => write!(f, "."),
            System => write!(f, "x"),
        }
    }
}

#[derive(Clone, Copy, Eq, Debug, PartialEq, Hash)]
enum Direction {
    Up,
    Down,
    Left,
    Right
}

impl Direction {
    fn new(n: i64) -> Result<Direction> {
        use self::Direction::*;
        match n {
            1 => Ok(Up),
            2 => Ok(Down),
            3 => Ok(Left),
            4 => Ok(Right),
            x => err!("Cannot read direction: {}", x)
        }
    }

    fn to_digit(self) -> i64 {
        use self::Direction::*;
        match self {
            Up => 1,
            Down => 2,
            Left => 3,
            Right => 4
        }
    }

    fn to_coordinate(&self) -> Coordinate {
        use self::Direction::*;
        match self {
            Up    => Coordinate { x: 0 , y: 1  },
            Down  => Coordinate { x: 0 , y: -1 },
            Left  => Coordinate { x: -1, y: 0  },
            Right => Coordinate { x: 1 , y: 0  },
        }
    }
}

struct Droid {
    program: Program,
    floor_map: BTreeMap<Coordinate, SquareType>,
    leak_location: Coordinate,
    current_coord: Coordinate,
}

impl Droid {
    fn new(memory: Vec<i64>) -> Droid {
        Droid {
            program: Program::new(memory),
            floor_map: BTreeMap::new(),
            leak_location: Coordinate::new(0, 0),
            current_coord: Coordinate::new(0, 0)
        }
    }

    fn shortest_path_from_to(&self, from: Coordinate, to: Coordinate) -> Result<Vec<Coordinate>> {
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

            for neighbour in c.neighbours().into_iter().filter(|coord| self.floor_map.get(&coord) == Some(&SquareType::Open)) {
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

        let mut path_to_take: Vec<Coordinate> = vec![to];
        let mut current_position = to;
        loop {
            let next_position = current_position.neighbours().into_iter().filter(|&c| d.contains_key(&c)).min_by_key(|&c| d[&c]);
            if next_position.is_none() {
                println!("{}", self);
                return err!("Cannot find any neighbours for {}", current_position);
            }

            let next_position = next_position.unwrap();
            path_to_take.push(next_position);
            if next_position == from {
                break;
            }
            current_position = next_position;
        }

        path_to_take.reverse();
        Ok(path_to_take)
    }

    fn steps_to_get_to(&self, coord: Coordinate) -> Result<usize> {
        let path = self.shortest_path_from_to(Coordinate::new(0, 0), coord)?;

        Ok(path.len() - 1)
    }

    fn dist_to_leak(&self) -> Result<usize> {
        self.steps_to_get_to(self.leak_location)
    }

    fn find_leak(&mut self, stop_on_leak: bool) -> Result<()> {
        self.floor_map.insert(self.current_coord, SquareType::Open);

        let mut current_target = Coordinate::new(0, 0);

        // set input to chosen direction
        // run program
        // figure out what to do based on output values
        'main: loop {
            if self.floor_map.contains_key(&current_target) {
                let next_potential_target = self.floor_map.iter()
                    .filter(|(_, &square)| square == SquareType::Open)
                    .flat_map(|(&coord, _)| coord.neighbours())
                    .filter(|&coord| !self.floor_map.contains_key(&coord))
                    .next();
                if let Some(c) = next_potential_target {
                    current_target = c;
                } else {
                    return Ok(());
                }
            }

            // println!("Current target = {:?}", current_target);

            let path_to_next_target = self.shortest_path_from_to(self.current_coord, current_target)?;

            // println!("Path from {} to {} = {:?}", self.current_coord, current_target, path_to_next_target);

            let directions = convert_path_to_directions(path_to_next_target)?;

            for direction in directions {
                // println!("Inputting {:?} ({}) into program", direction, direction.to_digit());
                self.program.set_input(direction.to_digit());
                if let Some(result) = self.program.run_program()? {
                    match result {
                        0 => {
                            // hit a wall
                            // println!("{} is a wall, droid doesn't move", self.current_coord + direction.to_coordinate());
                            self.floor_map.insert(self.current_coord + direction.to_coordinate(), SquareType::Wall);
                            continue 'main;
                        },
                        1 => {
                            // all is well
                            // println!("{} is clear, droid moves", self.current_coord + direction.to_coordinate());
                            self.floor_map.insert(self.current_coord + direction.to_coordinate(), SquareType::Open);
                            self.current_coord += direction.to_coordinate();
                        },
                        2 => {
                            // moved and found leak!
                            println!("Found leak at {}!", self.current_coord + direction.to_coordinate());
                            // println!("{}", self);
                            self.floor_map.insert(self.current_coord + direction.to_coordinate(), SquareType::System);
                            self.leak_location = self.current_coord + direction.to_coordinate();
                            self.current_coord += direction.to_coordinate();
                            if stop_on_leak {
                                break 'main;
                            }
                        },
                        x => panic!("Unexpected output from program: {}", x)
                    }
                }
            }
        }

        Ok(())
    }

    fn time_for_oxygen_spread(&mut self) -> Result<usize> {
        let mut oxygen_squares: BTreeSet<Coordinate> = BTreeSet::new();
        oxygen_squares.insert(self.leak_location);

        let mut t = 0;

        loop {
            let near_oxygen_squares: Vec<_> = oxygen_squares.iter()
                .flat_map(|&coord| coord.neighbours())
                .filter(|&coord| self.floor_map[&coord] == SquareType::Open && !oxygen_squares.contains(&coord)).collect();

            if near_oxygen_squares.is_empty() {
                break;
            }

            oxygen_squares.extend(near_oxygen_squares.iter());

            t += 1;
        }

        Ok(t)
    }
}

impl fmt::Display for Droid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let first_x = self.floor_map.keys().map(|&c| c.x).min().unwrap();
        let last_x = self.floor_map.keys().map(|&c| c.x).max().unwrap();
        let first_y = self.floor_map.keys().map(|&c| c.y).min().unwrap();
        let last_y = self.floor_map.keys().map(|&c| c.y).max().unwrap();

        for y in (first_y..=last_y).rev() {
            for x in first_x..=last_x {
                let coord = Coordinate::new(x, y);
                if coord == self.current_coord {
                    write!(f, "D")?;
                    continue;
                }

                if coord == Coordinate::new(0, 0) {
                    write!(f, "O")?;
                    continue;
                }

                match self.floor_map.get(&coord) {
                    Some(square_type) => {
                        write!(f, "{}", square_type)?;
                    },
                    None => {
                        write!(f, " ")?;
                    }
                }
            }
            write!(f, "{}", '\n')?;
        }

        Ok(())
    }
}

fn convert_path_to_directions(path: Vec<Coordinate>) -> Result<Vec<Direction>> {
    path.windows(2).map(|t| {
        use self::Direction::*;

        let coord_difference = t[1] - t[0];

        match (coord_difference.x, coord_difference.y) {
            (1, 0) => Ok(Right),
            (-1, 0) => Ok(Left),
            (0, 1) => Ok(Up),
            (0, -1) => Ok(Down),
            (x, y) => err!("{}", format!("Invalid difference: {}, {}", x, y))
        }
    }).collect()
}


pub fn q1(fname: String) -> usize {
    let mut f = File::open(fname).expect("File not found");
    let mut f_contents = String::new();

    f.read_to_string(&mut f_contents).expect("Couldn't find file");

    let memory: Vec<i64> = f_contents.trim().split(',').map(|s| s.parse().unwrap()).collect();

    _q1(memory).unwrap()
}

fn _q1(memory: Vec<i64>) -> Result<usize> {
    let mut droid = Droid::new(memory);
    droid.find_leak(true)?;
    droid.dist_to_leak()
}

pub fn q2(fname: String) -> usize {
    let mut f = File::open(fname).expect("File not found");
    let mut f_contents = String::new();

    f.read_to_string(&mut f_contents).expect("Couldn't find file");

    let memory: Vec<i64> = f_contents.trim().split(',').map(|s| s.parse().unwrap()).collect();

    _q2(memory).unwrap()
}

fn _q2(memory: Vec<i64>) -> Result<usize> {
    let mut droid = Droid::new(memory);

    droid.find_leak(false)?;

    // Map has been completely filled in
    println!("{}", droid);

    droid.time_for_oxygen_spread()
}
