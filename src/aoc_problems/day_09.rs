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
    first_input: i64,
    second_input: i64,
    current_input: usize,
    pointer_idx: usize,
    relative_base: i64
}

impl Program {
    fn new(memory: Vec<i64>, first_input: i64, second_input: i64)  -> Program {
        Program {
            memory,
            first_input,
            second_input,
            current_input: 1,
            pointer_idx: 0,
            relative_base: 0
        }
    }

    fn get_input(&mut self) -> Result<i64> {
        Ok(self.first_input)
    }

    fn set_input(&mut self, input: i64) {
        self.second_input = input;
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

                println!("Index being read = {}", idx);

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
                println!(
                    "Changing output idx from {} to add relative base {}",
                    self.memory[idx],
                    self.relative_base
                );
                (self.memory[idx] + self.relative_base) as usize
            },
            _ => panic!("Should never be here")
        }
    }

    fn run_program(&mut self) -> Result<Option<i64>> {
        loop {
            let current_instruction = Instruction::new(self.memory[self.pointer_idx] as usize)?;

            println!(
                "Instruction being run ({}) is {:?}",
                self.memory[self.pointer_idx] as usize,
                current_instruction
            );

            match current_instruction.opcode {
                1 => {
                    println!("Parameters = {:?}", &self.memory[self.pointer_idx+1..self.pointer_idx+4]);
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
                    println!("Parameters = {:?}", &self.memory[self.pointer_idx+1..self.pointer_idx+4]);
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
                    // println!("{} * {} set to position {}", input_1, input_2, output_idx);

                    self.pointer_idx += 4;
                },
                3 => {
                    println!("Parameters = {:?}", &self.memory[self.pointer_idx+1]);
                    let output_idx = self.get_output_idx(
                        self.pointer_idx + 1,
                        current_instruction.parameters[0]
                    );
                    let input = self.get_input()?;
                    println!("Adding input {} to m[{}]", input, output_idx);
                    self.set_parameter(output_idx, input)?;

                    self.pointer_idx += 2;
                },
                4 => {
                    println!("Parameters = {:?}", &self.memory[self.pointer_idx+1]);
                    let output_val = self.get_parameter(
                        current_instruction.parameters[0],
                        self.memory[self.pointer_idx+1]
                    );

                    // let output_idx = self.memory[self.pointer_idx+1];
                    self.pointer_idx += 2;

                    return Ok(Some(output_val));
                },
                5 => {
                    println!("Parameters = {:?}", &self.memory[self.pointer_idx+1..self.pointer_idx+3]);
                    let input_1 = self.get_parameter(
                        current_instruction.parameters[0],
                        self.memory[self.pointer_idx+1],
                    );
                    let input_2 = self.get_parameter(
                        current_instruction.parameters[1],
                        self.memory[self.pointer_idx+2],
                    );
                    println!("Input 1 = {}, input 2 = {}", input_1, input_2);
                    if input_1 != 0 {
                        println!("Jumping to pointer index {}", input_2);
                        self.pointer_idx = input_2 as usize;
                    } else {
                        self.pointer_idx += 3;
                    }
                },
                6 => {
                    println!("Parameters = {:?}", &self.memory[self.pointer_idx+1..self.pointer_idx+3]);
                    let input_1 = self.get_parameter(
                        current_instruction.parameters[0],
                        self.memory[self.pointer_idx+1],
                    );
                    let input_2 = self.get_parameter(
                        current_instruction.parameters[1],
                        self.memory[self.pointer_idx+2],
                    );
                    if input_1 == 0 {
                        println!("Jumping to pointer index {}", input_2);
                        self.pointer_idx = input_2 as usize;
                    } else {
                        self.pointer_idx += 3;
                    }
                },
                7 => {
                    println!("Parameters = {:?}", &self.memory[self.pointer_idx+1..self.pointer_idx+4]);
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
                    println!("Parameters = {:?}", &self.memory[self.pointer_idx+1..self.pointer_idx+4]);
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
                    println!("Setting m[{}] to {} == {}", output_idx, input_1, input_2);
                    self.set_parameter(output_idx, if input_1 == input_2 {1} else {0})?;

                    self.pointer_idx += 4;
                },
                9 => {
                    println!("Parameters = {:?}", &self.memory[self.pointer_idx+1]);
                    let input_1 = self.get_parameter(
                        current_instruction.parameters[0],
                        self.memory[self.pointer_idx+1],
                    );
                    self.relative_base += input_1;

                    println!("Relative base is now {}", self.relative_base);

                    self.pointer_idx += 2;
                },
                99 => break,
                x => return err!("{}", format!("Incorrect opcode: {}", x))
            }
            println!("----------------------");
        }
        Ok(None)
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
    let mut program = Program::new(memory, 1, 1);
    let mut last_output = 0;
    while let Some(result) = program.run_program()? {
        last_output = result;
        println!("Result outputted = {}", result);
    }

    Ok(last_output as usize)
}

pub fn q2(fname: String) -> String {
    let mut f = File::open(fname).expect("File not found");
    let mut f_contents = String::new();

    f.read_to_string(&mut f_contents).expect("Couldn't find file");

    let memory: Vec<i64> = f_contents.trim().split(',').map(|s| s.parse().unwrap()).collect();

    _q2(memory).unwrap()
}

fn _q2(memory: Vec<i64>) -> Result<String> {
    let mut program = Program::new(memory, 2, 2);
    let mut last_output = 0;
    let mut output = vec![];
    while let Some(result) = program.run_program()? {
        output.push(result);
    }

    println!("Output = {:?}", output);

    Ok(output.iter().map(|&n| n.to_string()).collect::<Vec<_>>().join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day09_q1_test1() {
        let new_program: Vec<i64> = "104,1125899906842624,99".to_string().split(',').map(|s| s.parse().unwrap()).collect();

        assert_eq!(
            _q1(new_program).unwrap(),
            1125899906842624
        )
    }

    #[test]
    fn day09_q1_test2() {
        let new_program: Vec<i64> = "1102,34915192,34915192,7,4,7,99,0".to_string().split(',').map(|s| s.parse().unwrap()).collect();

        let mut program = Program::new(new_program, 1, 1);
        let mut output = vec![];
        while let Some(result) = program.run_program().unwrap() {
            output.push(result);
        }

        if !output.iter().any(|n: &i64| (*n).to_string().chars().count() == 16) {
            println!("Failure: no 16-digit number in result {:?}", output);
            assert!(false);
        }
    }

    #[test]
    fn day09_q1_test3() {
        let new_program: Vec<i64> = "109,1,204,-1,1001,100,1,100,1008,100,16,101,1006,101,0,99".to_string().split(',').map(|s| s.parse().unwrap()).collect();

        let mut program = Program::new(new_program.clone(), 1, 1);
        let mut output = vec![];
        while let Some(result) = program.run_program().unwrap() {
            output.push(result);
        }

        assert_eq!(
            output,
            new_program
        )
    }
}
