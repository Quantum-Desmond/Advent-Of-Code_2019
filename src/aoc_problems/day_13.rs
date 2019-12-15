use std::cmp;
use std::error::Error;
use std::fs::File;
use std::fmt;
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
    x: usize,
    y: usize
}

impl Coordinate {
    fn new(x: usize, y: usize) -> Coordinate {
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
    display: BTreeMap<Coordinate, TileType>,
    score: usize,
    ball_coord: Coordinate,
    paddle_coord: Coordinate
}

impl Program {
    fn new(memory: Vec<i64>)  -> Program {
        Program {
            memory,
            input: 0,
            current_input: 1,
            pointer_idx: 0,
            relative_base: 0,
            display: BTreeMap::new(),
            score: 0,
            ball_coord: Coordinate::new(0, 0),
            paddle_coord: Coordinate::new(0, 0),
        }
    }

    fn get_input(&mut self) -> Result<i64> {
        Ok((self.ball_coord.x as i64 - self.paddle_coord.x as i64).signum())
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

    fn run_game(&mut self) -> Result<()> {
        loop {
            // First output: x coord
            let x = if let Some(output) = self.run_program()? {
                output
            } else { break; };

            // Second output: y coord
            let y = if let Some(output) = self.run_program()? {
                output
            } else { break; };

            // Third output: tile type
            let third_output = if let Some(output) = self.run_program()? {
                output
            } else { break; };

            match (x, y) {
                (-1, 0) => {
                    self.score = third_output as usize;
                },
                (x, y) => {
                    let coord = Coordinate::new(x as usize, y as usize);
                    let tile = TileType::new(third_output as usize)?;

                    if tile == TileType::Ball {
                        self.ball_coord = coord;
                    }

                    if tile == TileType::Paddle {
                        self.paddle_coord = coord;
                    }
                    self.display.insert(coord, tile);

                    if tile == TileType::Block {
                        let block_count = self.display.values().filter(|&&tile| tile == TileType::Block).count();
                        if block_count == 0 {
                            return Ok(());
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let current_y: usize = 0;
        for (coord, tile) in self.display.iter() {
            if coord.y != current_y {
                write!(f, "{}", '\n')?;
            }

            write!(f, "{}", tile)?;
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Eq, Debug, PartialEq, Hash)]
enum TileType {
    Empty,
    Wall,
    Block,
    Paddle,
    Ball
}

impl TileType {
    fn new(n: usize) -> Result<TileType> {
        use self::TileType::*;
        match n {
            0 => Ok(Empty),
            1 => Ok(Wall),
            2 => Ok(Block),
            3 => Ok(Paddle),
            4 => Ok(Ball),
            x => err!("Invalid colour: {}", x)
        }
    }

    fn to_digit(&self) -> i64 {
        use self::TileType::*;
        match self {
            Empty => 0,
            Wall => 1,
            Block => 2,
            Paddle => 3,
            Ball => 4
        }
    }
}

impl fmt::Display for TileType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::TileType::*;
        match self {
            Empty => write!(f, "."),
            Wall => write!(f, "█"),
            Block => write!(f, "x"),
            Paddle => write!(f, "-"),
            Ball => write!(f, "O"),
        }
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
    let mut tiles: BTreeMap<Coordinate, TileType> = BTreeMap::new();
    loop {
        // First output: x coord
        let x: usize = if let Some(output) = program.run_program()? {
            output as usize
        } else { break; };

        // Second output: y coord
        let y: usize = if let Some(output) = program.run_program()? {
            output as usize
        } else { break; };

        // Third output: tile type
        let tile_type: usize = if let Some(output) = program.run_program()? {
            output as usize
        } else { break; };

        tiles.insert(Coordinate::new(x, y), TileType::new(tile_type)?);
    }

    Ok(tiles.values().filter(|&&tile| tile == TileType::Block).count())
}

pub fn q2(fname: String) -> usize {
    let mut f = File::open(fname).expect("File not found");
    let mut f_contents = String::new();

    f.read_to_string(&mut f_contents).expect("Couldn't find file");

    let memory: Vec<i64> = f_contents.trim().split(',').map(|s| s.parse().unwrap()).collect();

    _q2(memory).unwrap()
}

fn _q2(mut memory: Vec<i64>) -> Result<usize> {
    // Change first number so game can play
    memory[0] = 2;

    let mut program = Program::new(memory);
    program.run_game()?;

    Ok(program.score)
}