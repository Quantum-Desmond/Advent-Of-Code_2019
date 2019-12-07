use std::error::Error;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::result;

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

#[derive(Clone, Copy, Eq, Debug, PartialEq, Hash)]
enum Parameter {
    Position,
    Immediate
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
            99 => 0,
            x => return err!("{}", format!("Cannot read opcode: {}", x))
        };

        digit_list.resize(params_length, 0);
        let parameters: Result<Vec<Parameter>> = digit_list.into_iter().map(|d| match d {
            0 => Ok(Parameter::Position),
            1 => Ok(Parameter::Immediate),
            x => err!("{}", format!("Cannot read parameter digit: {}", x))
        }).collect();
        let parameters = parameters?;

        Ok(
            Instruction {
                opcode,
                parameters
            }
        )

    }
}

struct Amplifier {
    memory: Vec<i32>,
    first_input: i32,
    second_input: i32,
    current_input: usize
}

impl Amplifier {
    fn new(memory: Vec<i32>, first_input: i32, second_input: i32)  -> Amplifier {
        Amplifier {
            memory,
            first_input,
            second_input,
            current_input: 1
        }
    }

    fn get_input(&mut self) -> Result<i32> {
        let return_value = match self.current_input {
            1 => self.first_input,
            2 => self.second_input,
            x => return err!("{}", format!("Cannot understand input number {}", x))
        };

        self.current_input += 1;

        Ok(return_value)
    }

    fn get_parameter(&self, parameter_form: Parameter, val: i32) -> i32 {
        use self::Parameter::*;

        match parameter_form {
            Position => self.memory[val as usize],
            Immediate => val
        }
    }

    fn run_program(&mut self) -> Result<i32> {
        let mut pointer_idx = 0;
        loop {
            let current_instruction = Instruction::new(self.memory[pointer_idx] as usize)?;
            match current_instruction.opcode {
                1 => {
                    let input_1 = self.get_parameter(
                        current_instruction.parameters[0],
                        self.memory[pointer_idx+1],
                    );
                    let input_2 = self.get_parameter(
                        current_instruction.parameters[1],
                        self.memory[pointer_idx+2],
                    );
                    let output_idx = self.memory[pointer_idx+3] as usize;
                    self.memory[output_idx] = input_1 + input_2;

                    pointer_idx += 4;
                },
                2 => {
                    let input_1 = self.get_parameter(
                        current_instruction.parameters[0],
                        self.memory[pointer_idx+1],
                    );
                    let input_2 = self.get_parameter(
                        current_instruction.parameters[1],
                        self.memory[pointer_idx+2],
                    );
                    let output_idx = self.memory[pointer_idx+3] as usize;
                    self.memory[output_idx] = input_1 * input_2;

                    pointer_idx += 4;
                },
                3 => {
                    let output_idx = self.memory[pointer_idx+1] as usize;
                    self.memory[output_idx] = self.get_input()?;

                    pointer_idx += 2;
                },
                4 => {
                    let output_idx = self.memory[pointer_idx+1] as usize;

                    return Ok(self.memory[output_idx]);

                    // pointer_idx += 2;
                },
                5 => {
                    let input_1 = self.get_parameter(
                        current_instruction.parameters[0],
                        self.memory[pointer_idx+1],
                    );
                    let input_2 = self.get_parameter(
                        current_instruction.parameters[1],
                        self.memory[pointer_idx+2],
                    );
                    if input_1 != 0 {
                        pointer_idx = input_2 as usize;
                    } else {
                        pointer_idx += 3;
                    }
                },
                6 => {
                    let input_1 = self.get_parameter(
                        current_instruction.parameters[0],
                        self.memory[pointer_idx+1],
                    );
                    let input_2 = self.get_parameter(
                        current_instruction.parameters[1],
                        self.memory[pointer_idx+2],
                    );
                    if input_1 == 0 {
                        pointer_idx = input_2 as usize;
                    } else {
                        pointer_idx += 3;
                    }
                },
                7 => {
                    let input_1 = self.get_parameter(
                        current_instruction.parameters[0],
                        self.memory[pointer_idx+1],
                    );
                    let input_2 = self.get_parameter(
                        current_instruction.parameters[1],
                        self.memory[pointer_idx+2],
                    );
                    let output_idx = self.memory[pointer_idx+3] as usize;
                    self.memory[output_idx] = if input_1 < input_2 {
                        1
                    } else {
                        0
                    };

                    pointer_idx += 4;
                },
                8 => {
                    let input_1 = self.get_parameter(
                        current_instruction.parameters[0],
                        self.memory[pointer_idx+1],
                    );
                    let input_2 = self.get_parameter(
                        current_instruction.parameters[1],
                        self.memory[pointer_idx+2],
                    );
                    let output_idx = self.memory[pointer_idx+3] as usize;
                    self.memory[output_idx] = if input_1 == input_2 {
                        1
                    } else {
                        0
                    };

                    pointer_idx += 4;
                },
                99 => break,
                x => return err!("{}", format!("Incorrect opcode: {}", x))
            }
        }
        err!("Shouldn't get here!")
    }
}

pub fn permutations(size: usize) -> Permutations {
    Permutations { idxs: (0..size).collect(), swaps: vec![0; size], i: 0 }
}
 
pub struct Permutations {
    idxs: Vec<usize>,
    swaps: Vec<usize>,
    i: usize,
}
 
impl Iterator for Permutations {
    type Item = Vec<usize>;
 
    fn next(&mut self) -> Option<Self::Item> {
        if self.i > 0 {
            loop {
                if self.i >= self.swaps.len() { return None; }
                if self.swaps[self.i] < self.i { break; }
                self.swaps[self.i] = 0;
                self.i += 1;
            }
            self.idxs.swap(self.i, (self.i & 1) * self.swaps[self.i]);
            self.swaps[self.i] += 1;
        }
        self.i = 1;
        Some(self.idxs.clone())
    }
}

fn get_permutations(size: usize) -> Vec<Vec<usize>> {
    let perms = Permutations { idxs: (0..size).collect(), swaps: vec![0; size], i: 0 };
    perms.collect::<Vec<_>>()
}
 
pub fn q1(fname: String) -> usize {
    let mut f = File::open(fname).expect("File not found");
    let mut f_contents = String::new();

    f.read_to_string(&mut f_contents).expect("Couldn't find file");

    let memory: Vec<i32> = f_contents.trim().split(',').map(|s| s.parse().unwrap()).collect();

    _q1(memory).unwrap()
}

fn _q1(memory: Vec<i32>) -> Result<usize> {
    let amp_count = 5;
    let permutations = get_permutations(amp_count);

    let mut input: i32 = 0;
    let mut max_signal = 0;
    for permutation in permutations {
        for phase_setting in permutation {
            let mut amp = Amplifier::new(memory.clone(), phase_setting as i32, input);
            input = amp.run_program()?;
        }

        if input > max_signal {
            max_signal = input;
        }
        
    }

    Ok(max_signal as usize)
}

pub fn q2(fname: String) -> usize {
    let mut f = File::open(fname).expect("File not found");
    let mut f_contents = String::new();

    f.read_to_string(&mut f_contents).expect("Couldn't find file");

    let memory: Vec<i32> = f_contents.trim().split(',').map(|s| s.parse().unwrap()).collect();

    _q2(memory).unwrap()
}

fn _q2(_test_programs: Vec<i32>) -> Result<usize> {
    unimplemented!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day07_q1_test1() {
        let memory = "3,15,3,16,1002,16,10,16,1,16,15,15,4,15,99,0,0".to_string();
        let memory: Vec<i32> = memory.trim().split(',').map(|s| s.parse().unwrap()).collect();
        assert_eq!(
            _q1(memory).unwrap(),
            43210
        );
    }

    #[test]
    fn day07_q1_test2() {
        let memory = "3,23,3,24,1002,24,10,24,1002,23,-1,23,101,5,23,23,1,24,23,23,4,23,99,0,0".to_string();
        let memory: Vec<i32> = memory.trim().split(',').map(|s| s.parse().unwrap()).collect();
        assert_eq!(
            _q1(memory).unwrap(),
            54321
        );
    }

    #[test]
    fn day07_q1_test3() {
        let memory = "3,31,3,32,1002,32,10,32,1001,31,-2,31,1007,31,0,33,1002,33,7,33,1,33,31,31,1,32,31,31,4,31,99,0,0,0".to_string();
        let memory: Vec<i32> = memory.trim().split(',').map(|s| s.parse().unwrap()).collect();
        assert_eq!(
            _q1(memory).unwrap(),
            65210
        );
    }

    #[test]
    fn day07_permutations() {
        let perms = get_permutations(5);
        assert_eq!(
            perms.len(),
            120
        );
    }
}
