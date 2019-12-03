use std::error::Error;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::result;

type Result<T> = result::Result<T, Box<dyn Error>>;

fn pause() {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    // We want the cursor to stay at the end of the line, so we print without a newline and flush manually.
    write!(stdout, "Press any key to continue...").unwrap();
    stdout.flush().unwrap();

    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
}

fn output_value(mut numbers: Vec<usize>, noun: usize, verb: usize) -> usize {
    numbers[1] = noun;
    numbers[2] = verb;

    let mut pointer_idx: usize = 0;
    loop {
        match numbers[pointer_idx] {
            1 => {
                let input_1 = numbers[numbers[pointer_idx+1]];
                let input_2 = numbers[numbers[pointer_idx+2]];
                let output_idx = numbers[pointer_idx+3];
                numbers[output_idx] = input_1 + input_2;

                pointer_idx += 4;
            },
            2 => {
                let input_1 = numbers[numbers[pointer_idx+1]];
                let input_2 = numbers[numbers[pointer_idx+2]];
                let output_idx = numbers[pointer_idx+3];
                numbers[output_idx] = input_1 * input_2;

                pointer_idx += 4;
            },
            99 => break,
            x => panic!("Incorrect intcode: {}", x)
        }
    }

    numbers[0]
}

pub fn q1(fname: String) -> usize {
    let mut f = File::open(fname).expect("File not found");
    let mut f_contents = String::new();

    f.read_to_string(&mut f_contents).expect("Couldn't find file");
    let numbers: Vec<usize> = f_contents.trim().split(',').map(|x: &str| {
        x.parse().unwrap()
    }).collect();

    output_value(numbers, 12, 2)
}

pub fn q2(fname: String) -> usize {
    let mut f = File::open(fname).expect("File not found");
    let mut f_contents = String::new();

    f.read_to_string(&mut f_contents).expect("Couldn't find file");
    let numbers: Vec<usize> = f_contents.trim().split(',').map(|x: &str| {
        x.parse().unwrap()
    }).collect();

    for noun in 0..100 {
        for verb in 0..100 {
            if output_value(numbers.clone(), noun, verb) == 19690720 {
                return 100 * noun + verb;
            }
        }
    }

    panic!("Shouldn't get here");
}
