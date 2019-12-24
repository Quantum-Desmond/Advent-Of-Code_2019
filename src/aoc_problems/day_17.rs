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
    inputs: VecDeque<i64>
}

impl Program {
    fn new(memory: Vec<i64>)  -> Program {
        Program {
            memory,
            input: 0,
            current_input: 1,
            pointer_idx: 0,
            relative_base: 0,
            inputs: VecDeque::new()
        }
    }

    fn get_input(&mut self) -> Result<i64> {
        let input = self.inputs.pop_front().ok_or("No inputs left")?;
        Ok(input)
    }

    fn add_input(&mut self, input: i64) {
        self.inputs.push_back(input);
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
}

#[derive(Clone, Copy, Eq, Debug, PartialEq, Hash)]
enum Tile {
    Scaffold,
    Open,
    Robot(Direction)
}

impl Tile {
    fn new(c: char) -> Result<Tile> {
        use self::Tile::*;
        match c {
            '.' => Ok(Open),
            '#' => Ok(Scaffold),
            '^' => Ok(Robot(Direction::Up)),
            'v' => Ok(Robot(Direction::Down)),
            '<' => Ok(Robot(Direction::Left)),
            '>' => Ok(Robot(Direction::Right)),
            c => err!("Cannot read character {}", c)
        }
    }
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Tile::*;
        match self {
            Open => write!(f, "{}", '.'),
            Scaffold => write!(f, "{}", '#'),
            Robot(Direction::Up) => write!(f, "{}", '^'),
            Robot(Direction::Down) => write!(f, "{}", 'v'),
            Robot(Direction::Left) => write!(f, "{}", '<'),
            Robot(Direction::Right) => write!(f, "{}", '>'),
        }
    }
}

#[derive(Clone, Eq, Debug, PartialEq, Hash)]
struct Scaffolding {
    map: BTreeMap<Coordinate, Tile>
}

impl Scaffolding {
    fn new(chars: Vec<char>) -> Result<Scaffolding> {
        let mut x = 0;
        let mut y = 0;

        let mut map: BTreeMap<Coordinate, Tile> = BTreeMap::new();

        for c in chars {
            if c == '\n' {
                y += 1;
                x = 0;
                continue;
            }

            map.insert(Coordinate::new(x, y), Tile::new(c)?);

            x += 1;
        }

        Ok(Scaffolding { map })
    }

    fn crossover_list(&self) -> Vec<Coordinate> {
        self.map
            .iter()
            .filter(|(&c, &tile)| {
                tile == Tile::Scaffold &&
                c.neighbours().iter().filter(|&cc| self.map.get(&cc) == Some(&Tile::Scaffold)).count() == 4
            }).map(|(&c, _)| c).collect()
    }
}

impl fmt::Display for Scaffolding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut current_y = 0;
        for (c, tile) in self.map.iter() {
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

    let memory: Vec<i64> = f_contents.trim().split(',').map(|s| s.parse().unwrap()).collect();

    _q1(memory).unwrap()
}

fn _q1(memory: Vec<i64>) -> Result<usize> {
    let mut program = Program::new(memory);

    let mut output: Vec<char> = vec![];
    while let Some(output_num) = program.run_program()? {
        output.push(output_num as u8 as char);
    }

    let scaffolding_map = Scaffolding::new(output)?;

    let crossovers = scaffolding_map.crossover_list();

    println!("Crossovers = {:?}", crossovers);

    println!("{}", scaffolding_map);

    Ok(crossovers.iter().map(|&c| c.x * c.y).sum())
}

pub fn q2(fname: String) -> usize {
    let mut f = File::open(fname).expect("File not found");
    let mut f_contents = String::new();

    f.read_to_string(&mut f_contents).expect("Couldn't find file");

    let memory: Vec<i64> = f_contents.trim().split(',').map(|s| s.parse().unwrap()).collect();

    _q2(memory).unwrap()
}

fn _q2(mut memory: Vec<i64>) -> Result<usize> {
    memory[0] = 2;

    let mut program = Program::new(memory);

    let mut main_fn: Vec<char> = "A,B,A,B,C,C,B,A,B,C".chars().collect();
    main_fn.push('\n');

    let mut a_fn: Vec<char> = "L,12,L,6,L,8,R,6".chars().collect();
    a_fn.push('\n');

    let mut b_fn: Vec<char> = "L,8,L,8,R,4,R,6,R,6".chars().collect();
    b_fn.push('\n');

    let mut c_fn: Vec<char> = "L,12,R,6,L,8".chars().collect();
    c_fn.push('\n');

    let video_fn: Vec<char> = vec!['y', '\n'];

    let inputs: Vec<&Vec<char>> = vec![&main_fn, &a_fn, &b_fn, &c_fn, &video_fn];

    for input_line in inputs {
        for c in input_line {
            program.add_input(*c as u8 as i64);
        }
    }

    while let Some(output) = program.run_program()? {
        if output > 255 {
            print!("{}", output);
        } else {
            print!("{}", output as u8 as char);
        }
    }

    Ok(0)
}
