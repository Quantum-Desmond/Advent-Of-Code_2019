use std::error::Error;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::result;

use itertools::Itertools;

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

struct Signal {
    numbers: Vec<i32>
}

impl Signal {
    fn new(numbers: Vec<i32>) -> Signal {
        Signal {
            numbers
        }
    }

    fn fft_iterate(&mut self) -> Result<()> {
        let result = (1..=self.numbers.len()).map(|n| fft_step_for(&self.numbers, n)).collect();
        self.numbers = result;

        Ok(())
    }

    fn shorter_fft_iterate(&mut self, target: usize) -> Result<()> {
        let mut current_idx = self.numbers.len() - 1;

        let mut result = 0;
        while current_idx >= target {
            result += self.numbers[current_idx];
            result = result % 10;

            self.numbers[current_idx] = result;

            current_idx -= 1;
        }

        Ok(())
    }
}

fn fft_pattern(step: usize, size: usize) -> Vec<i32> {
    let mut result: Vec<i32> = vec![];

    'main: loop {
        for digit in [0, 1, 0, -1].iter() {
            for _ in 0..step {
                result.push(*digit);
                if result.len() >= size + 1 {
                    break 'main;
                }
            }
        }
    }

    result.remove(0);
    result
}

fn fft_step_for(numbers: &Vec<i32>, n: usize) -> i32 {
    let pattern = fft_pattern(n, numbers.len());

    let sum_number: i32 = numbers.iter().zip(pattern.iter()).map(|(x, y)| x * y).sum();
    sum_number.abs() % 10
}

pub fn q1(fname: String) -> String {
    let mut f = File::open(fname).expect("File not found");
    let mut f_contents = String::new();

    f.read_to_string(&mut f_contents).expect("Couldn't find file");

    let number_list: Vec<i32> = f_contents.trim().chars().map(|s| s.to_digit(10).unwrap() as i32).collect();

    _q1(number_list).unwrap()
}

fn _q1(numbers: Vec<i32>) -> Result<String> {
    let mut signal = Signal::new(numbers);

    for _ in 0..100 {
        signal.fft_iterate()?;
    }

    Ok(
        signal.numbers[..8].iter().join("")
    )
}

pub fn q2(fname: String) -> String {
    let mut f = File::open(fname).expect("File not found");
    let mut f_contents = String::new();

    f.read_to_string(&mut f_contents).expect("Couldn't find file");

    let number_list: Vec<i32> = f_contents.trim().chars().map(|s| s.to_digit(10).unwrap() as i32).collect();

    _q2(number_list).unwrap()
}


fn _q2(number_list: Vec<i32>) -> Result<String> {
    let offset: usize = number_list[..7].iter().join("").parse()?;

    let mut extended_list: Vec<i32> = vec![];
    for _ in 0..10_000 {
        extended_list.extend(&number_list);
    }

    println!("Length of number list = {}", extended_list.len());
    println!("Fraction through list = {}", (offset as f32 / extended_list.len() as f32));

    let mut signal = Signal::new(extended_list);

    for iteration in 0..100 {
        signal.shorter_fft_iterate(offset)?;
        println!("Completed {} iterations", iteration);
    }

    Ok(
        signal.numbers[offset..offset+8].iter().join("")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day16_q1_test1() {
        let input: Vec<_> = "80871224585914546619083218645595".trim().chars().map(|c| c.to_digit(10).unwrap() as i32).collect();

        assert_eq!(
            _q1(input).unwrap(),
            "24176176".to_string()
        )
    }

    #[test]
    fn day16_q1_test2() {
        let input: Vec<_> = "19617804207202209144916044189917".trim().chars().map(|c| c.to_digit(10).unwrap() as i32).collect();

        assert_eq!(
            _q1(input).unwrap(),
            "73745418".to_string()
        )
    }

    #[test]
    fn day16_q1_test3() {
        let input: Vec<_> = "69317163492948606335995924319873".trim().chars().map(|c| c.to_digit(10).unwrap() as i32).collect();

        assert_eq!(
            _q1(input).unwrap(),
            "52432133".to_string()
        )
    }

    #[test]
    fn day16_q2_test1() {
        let input: Vec<_> = "03036732577212944063491565474664".trim().chars().map(|c| c.to_digit(10).unwrap() as i32).collect();

        assert_eq!(
            _q2(input).unwrap(),
            "84462026".to_string()
        )
    }

    #[test]
    fn day16_q2_test2() {
        let input: Vec<_> = "02935109699940807407585447034323".trim().chars().map(|c| c.to_digit(10).unwrap() as i32).collect();

        assert_eq!(
            _q2(input).unwrap(),
            "78725270".to_string()
        )
    }
    #[test]
    fn day16_q2_test3() {
        let input: Vec<_> = "03081770884921959731165446850517".trim().chars().map(|c| c.to_digit(10).unwrap() as i32).collect();

        assert_eq!(
            _q2(input).unwrap(),
            "53553731".to_string()
        )
    }
}
