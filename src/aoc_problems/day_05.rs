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

fn get_param(parameter_form: Parameter, val: i32, program: &Vec<i32>) -> i32 {
    use self::Parameter::*;

    match parameter_form {
        Position => program[val as usize],
        Immediate => val
    }
}

fn run_program(mut program: Vec<i32>, input_value: i32) -> Result<()> {

    let mut pointer_idx = 0;
    loop {
        let current_instruction = Instruction::new(program[pointer_idx] as usize)?;
        match current_instruction.opcode {
            1 => {
                let input_1 = get_param(
                    current_instruction.parameters[0],
                    program[pointer_idx+1],
                    &program
                );
                let input_2 = get_param(
                    current_instruction.parameters[1],
                    program[pointer_idx+2],
                    &program
                );
                let output_idx = program[pointer_idx+3] as usize;
                program[output_idx] = input_1 + input_2;

                pointer_idx += 4;
            },
            2 => {
                let input_1 = get_param(
                    current_instruction.parameters[0],
                    program[pointer_idx+1],
                    &program
                );
                let input_2 = get_param(
                    current_instruction.parameters[1],
                    program[pointer_idx+2],
                    &program
                );
                let output_idx = program[pointer_idx+3] as usize;
                program[output_idx] = input_1 * input_2;

                pointer_idx += 4;
            },
            3 => {
                let output_idx = program[pointer_idx+1] as usize;
                program[output_idx] = input_value;

                pointer_idx += 2;
            },
            4 => {
                let input_idx = program[pointer_idx+1] as usize;
                println!("Output value from instruction {:?}: {}", current_instruction, program[input_idx]);

                pointer_idx += 2;
            },
            5 => {
                let input_1 = get_param(
                    current_instruction.parameters[0],
                    program[pointer_idx+1],
                    &program
                );
                let input_2 = get_param(
                    current_instruction.parameters[1],
                    program[pointer_idx+2],
                    &program
                );
                if input_1 != 0 {
                    pointer_idx = input_2 as usize;
                } else {
                    pointer_idx += 3;
                }
            },
            6 => {
                let input_1 = get_param(
                    current_instruction.parameters[0],
                    program[pointer_idx+1],
                    &program
                );
                let input_2 = get_param(
                    current_instruction.parameters[1],
                    program[pointer_idx+2],
                    &program
                );
                if input_1 == 0 {
                    pointer_idx = input_2 as usize;
                } else {
                    pointer_idx += 3;
                }
            },
            7 => {
                let input_1 = get_param(
                    current_instruction.parameters[0],
                    program[pointer_idx+1],
                    &program
                );
                let input_2 = get_param(
                    current_instruction.parameters[1],
                    program[pointer_idx+2],
                    &program
                );
                let output_idx = program[pointer_idx+3] as usize;
                program[output_idx] = if input_1 < input_2 {
                    1
                } else {
                    0
                };

                pointer_idx += 4;
            },
            8 => {
                let input_1 = get_param(
                    current_instruction.parameters[0],
                    program[pointer_idx+1],
                    &program
                );
                let input_2 = get_param(
                    current_instruction.parameters[1],
                    program[pointer_idx+2],
                    &program
                );
                let output_idx = program[pointer_idx+3] as usize;
                program[output_idx] = if input_1 == input_2 {
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
    Ok(())
}

pub fn q1(fname: String) -> usize {
    let mut f = File::open(fname).expect("File not found");
    let mut f_contents = String::new();

    f.read_to_string(&mut f_contents).expect("Couldn't find file");
    let test_programs: Vec<String> = f_contents.trim().lines().map(|x: &str| {
        x.to_string()
    }).collect();

    _q1(test_programs).unwrap()
}

fn _q1(test_programs: Vec<String>) -> Result<usize> {
    let programs: Vec<Vec<i32>> = test_programs.into_iter().map(|s| {
        s.split(',').filter_map(|ss| ss.parse::<i32>().ok()).collect()
    }).collect();

    for program in programs {
        println!("New program");
        run_program(program, 1)?;
    }

    unimplemented!();
}

pub fn q2(fname: String) -> usize {
    let mut f = File::open(fname).expect("File not found");
    let mut f_contents = String::new();

    f.read_to_string(&mut f_contents).expect("Couldn't find file");
    let test_programs: Vec<String> = f_contents.trim().lines().map(|x: &str| {
        x.to_string()

    }).collect();

    _q2(test_programs).unwrap()
}

fn _q2(test_programs: Vec<String>) -> Result<usize> {
    let programs: Vec<Vec<i32>> = test_programs.into_iter().map(|s| {
        s.split(',').filter_map(|ss| ss.parse::<i32>().ok()).collect()
    }).collect();

    for program in programs {
        println!("New program");
        run_program(program, 5)?;
    }
    unimplemented!();
}
