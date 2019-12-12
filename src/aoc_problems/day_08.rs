use std::cmp;
use std::error::Error;
use std::fmt;
use std::fs::File;
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
    x: u32,
    y: u32
}

impl Coordinate {
    fn new(x: u32, y: u32) -> Coordinate {
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

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
enum PixelType {
    Black,
    White,
    Transparent
}

impl PixelType {
    fn new(num: u32) -> PixelType {
        use self::PixelType::*;
        match num {
            0 => Black,
            1 => White,
            2 => Transparent,
            n => panic!("Cannot decipher pixel value: {}", n)
        }
    }
}

impl fmt::Display for PixelType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::PixelType::*;
        match self {
            Black => write!(f, " "),
            White => write!(f, "█"),
            Transparent => write!(f, "O"),
        }
    }
}

struct Picture {
    layers: Vec<BTreeMap<Coordinate, PixelType>>,
    width: u32,
    height: u32,
    layer_count: usize
}

impl Picture {
    fn new(mut pixels: Vec<u32>, width: u32, height: u32) -> Result<Picture> {
        let layer_count = pixels.len() / (width * height) as usize;
        println!("Layer count is {}", layer_count);

        let mut layers: Vec<_> = (0..layer_count).map(|_| BTreeMap::new()).collect();

        for layer_num in 0..layer_count {
            for y in 0..height {
                for x in 0..width {
                    layers[layer_num].insert(Coordinate::new(x, y), PixelType::new(pixels.pop().unwrap()));
                }
            }
        }

        Ok(
            Picture {
                layers, width, height, layer_count
            }
        )
    }

    fn first_opaque_pixel(&self, x: u32, y: u32) -> PixelType {
        use self::PixelType::*;
        for layer in 0..self.layer_count {
            if let Some(pixel_type) = self.layers[layer].get(&Coordinate::new(x, y)) {
                let pixel = match *pixel_type {
                    Transparent => continue,
                    pixel => pixel
                };

                return pixel;
            }
        }
        panic!("Cannot find pixel");
    }
}

impl fmt::Debug for Picture {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for layer in 0..self.layer_count {
            writeln!(f, "Layer {}", layer+1)?;
            let mut current_y = 0;
            for (coord, pixel) in self.layers[layer].iter() {
                if coord.y != current_y {
                    write!(f, "{}", '\n')?;
                    current_y = coord.y;
                }
                write!(f, "{}", pixel)?;
            }
            write!(f, "{}", '\n')?;
            // for y in 0..self.height {
            //     for x in 0..self.width {
            //         write!(f, "{}", self.layers[layer].get(&Coordinate::new(x, y)).unwrap())?;
            //     }
            //     write!(f, "{}", '\n')?;
            // }
        }
        Ok(())
    }
}

impl fmt::Display for Picture {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for y in 0..self.height {
            for x in 0..self.width {
                write!(f, "{}", self.first_opaque_pixel(x, y))?;
            }
            write!(f, "{}", '\n')?;
        }
        Ok(())
    }
}

pub fn q1(fname: String) -> usize {
    let mut f = File::open(fname).expect("File not found");
    let mut f_contents = String::new();

    f.read_to_string(&mut f_contents).expect("Couldn't find file");
    let pixel_data: Vec<u32> = f_contents.trim().chars().map(|x| {
        x.to_digit(10).unwrap()
    }).collect();

    _q1(pixel_data).unwrap()
}

fn _q1(mut pixels: Vec<u32>) -> Result<usize> {
    // reversing so pop pulls off the right order in Picture constructor
    pixels.reverse();
    let picture = Picture::new(pixels, 25, 6)?;

    let zero_layer_min = (0..picture.layer_count).map(|idx| {
            (idx, picture.layers[idx].values().filter(|&&pixel| pixel == PixelType::Black).count())
        })
        .min_by_key(|t| t.1)
        .ok_or("No elements!")?
        .0;

    println!("Layer with least zeroes = {}", zero_layer_min);

    Ok(
        picture.layers[zero_layer_min].values().filter(|&&pixel| pixel == PixelType::White).count()
        * picture.layers[zero_layer_min].values().filter(|&&pixel| pixel == PixelType::Transparent).count()
    )
}

pub fn q2(fname: String) -> usize {
    let mut f = File::open(fname).expect("File not found");
    let mut f_contents = String::new();

    f.read_to_string(&mut f_contents).expect("Couldn't find file");
    let pixel_data: Vec<u32> = f_contents.trim().chars().map(|x| {
        x.to_digit(10).unwrap()
    }).collect();

    _q2(pixel_data).unwrap()
}

fn _q2(mut pixels: Vec<u32>) -> Result<usize> {
    pixels.reverse();
    let picture = Picture::new(pixels, 25, 6)?;

    print!("{}", picture);

    unimplemented!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day08_q2_tests() {
        let mut pixels = vec![0,2,2,2,1,1,2,2,2,2,1,2,0,0,0,0];
        pixels.reverse();
        let picture = Picture::new(pixels, 2, 2).unwrap();
        print!("{:?}", picture);
        print!("{}", picture);
    }
}
