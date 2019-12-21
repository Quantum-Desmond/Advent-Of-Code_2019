use std::error::Error;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::result;
use std::str::FromStr;

use std::collections::HashMap;

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

#[derive(Clone, Default, Debug, Eq, PartialEq, Hash)]
struct Material {
    chemical: String,
    amount: usize
}

impl Material {
    fn new(chemical: String, amount: usize) -> Material {
        Material {
            chemical, amount
        }
    }
}

impl FromStr for Material {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self> {
        lazy_static! {
            static ref COORD_RE: Regex = Regex::new(
                r"(?P<count>\d+) (?P<chemical>\w+)"
            ).unwrap();
        }

        if !s.is_ascii() {
            return err!("area must be in ASCII");
        }

        if s.lines().count() != 1 {
            return err!("Only accepts 1 line");
        }

        if let Some(caps) = COORD_RE.captures(s) {
            return Ok(
                Material::new(
                    caps["chemical"].to_string(),
                    caps["count"].parse()?,
                )
            );
        }

        err!("Cannot parse material input: {}", s)
    }
}

#[derive(Clone, Default, Debug, Eq, PartialEq, Hash)]
struct RecipeRequirements {
    output: Material,
    inputs: Vec<Material>
}

#[derive(Clone, Default, Debug, Eq, PartialEq)]
struct Nanofactory {
    recipes: HashMap<String, RecipeRequirements>,
    material_counts: HashMap<String, usize>,
    ore_usage: usize,
    fuel_count: usize
}

impl Nanofactory {
    fn new(recipes: Vec<String>) -> Result<Nanofactory> {
        lazy_static! {
            static ref RECIPE_RE: Regex = Regex::new(
                r"(?P<inputs>[a-zA-Z0-9, ]+) => (?P<output_material>[a-zA-Z0-9 ]+)"
            ).unwrap();
        }

        let mut recipe_map: HashMap<String, RecipeRequirements> = HashMap::new();
        for recipe in recipes {
            if let Some(caps) = RECIPE_RE.captures(&recipe) {
                let output_material: Material = caps["output_material"].parse().expect("Invalid output material");
                let input_materials: Vec<Material> = caps["inputs"]
                    .split(", ")
                    .map(|s| s.parse().expect("Invalid material input"))
                    .collect();

                recipe_map.insert(
                    output_material.chemical.clone(),
                    RecipeRequirements {
                        output: output_material,
                        inputs: input_materials
                    }
                );
            } else {
                return err!("Cannot parse recipe {}", recipe);
            }
        }

        let material_counts: HashMap<String, usize> = recipe_map
            .keys()
            .map(|chemical| (chemical.clone(), 0))
            .collect();

        Ok(
            Nanofactory {
                recipes: recipe_map,
                material_counts,
                ore_usage: 0,
                fuel_count: 0
            }
        )
    }

    fn _create(&mut self, chemical_name: String, minimum_amount: usize) -> Result<()> {
        let chemical_rqmts = self.recipes.get(&chemical_name)
            .ok_or(format!("Cannot find recipe for chemical {}", chemical_name))?
            .clone();

        let complete_sets_needed: usize = (minimum_amount as f64 / chemical_rqmts.output.amount as f64).ceil() as usize;

        for input_material in &chemical_rqmts.inputs {
            if &input_material.chemical == "ORE" {
                self.ore_usage += complete_sets_needed*input_material.amount;
                if self.ore_usage > 1_000_000_000_000 {
                    return err!("Run out of ore");
                }
                continue;
            }

            let current_amount: usize = self.material_counts[&input_material.chemical];

            if current_amount < input_material.amount * complete_sets_needed {
                self._create(input_material.chemical.clone(), input_material.amount*complete_sets_needed - current_amount)?;
            }

            // while self.material_counts[&input_material.chemical] < bulk_amount * input_material.amount {
            //     self._create(input_material.chemical.clone(), bulk_amount)?;
            // }

            let input_count = self.material_counts.get_mut(&input_material.chemical)
                .ok_or(format!("Cannot find recipe for chemical {}", input_material.chemical))?;
            *input_count -= complete_sets_needed * input_material.amount;
        }

        // increase chemical amount
        let chemical_count = self.material_counts.get_mut(&chemical_name)
            .ok_or(format!("Cannot find recipe for chemical {}", chemical_name))?;
        *chemical_count += complete_sets_needed * chemical_rqmts.output.amount;

        Ok(())
    }

    // fn _create(&mut self, chemical_name: String, amount_to_make: usize) -> Result<()> {
    //     let chemical_rqmts = self.recipes.get(&chemical_name)
    //         .ok_or(format!("Cannot find recipe for chemical {}", chemical_name))?
    //         .clone();

    //     println!("Making {} of {}", amount_to_make, chemical_name);
    //     println!("Recipe to follow = {:?}", chemical_rqmts);

    //     for input_material in &chemical_rqmts.inputs {
    //         if &input_material.chemical == "ORE" {
    //             println!("Using {} ore", amount_to_make*input_material.amount);
    //             self.ore_usage += amount_to_make*input_material.amount;
    //             if self.ore_usage > 1_000_000_000_000 {
    //                 return err!("Run out of ore");
    //             }
    //             continue;
    //         }

    //         let current_amount: usize = self.material_counts[&input_material.chemical];

    //         if current_amount < input_material.amount * amount_to_make {
    //             self._create(input_material.chemical.clone(), input_material.amount*amount_to_make - current_amount)?;
    //         }

    //         // while self.material_counts[&input_material.chemical] < bulk_amount * input_material.amount {
    //         //     self._create(input_material.chemical.clone(), bulk_amount)?;
    //         // }

    //         let input_count = self.material_counts.get_mut(&input_material.chemical)
    //             .ok_or(format!("Cannot find recipe for chemical {}", input_material.chemical))?;
    //         *input_count -= amount_to_make * input_material.amount;
    //     }

    //     // increase chemical amount
    //     let chemical_count = self.material_counts.get_mut(&chemical_name)
    //         .ok_or(format!("Cannot find recipe for chemical {}", chemical_name))?;
    //     *chemical_count += amount_to_make * chemical_rqmts.output.amount;

    //     Ok(())
    // }

    fn produce_one_fuel(&mut self) -> Result<()> {
        self._create("FUEL".to_string(), 1)?;

        Ok(())
    }

    fn find_cyclic_usage(&mut self) -> Result<(usize, usize)> {
        let mut fuel_produced = 0;

        loop {
            self._create("FUEL".to_string(), 1)?;
            fuel_produced += 1;

            if self.material_counts.iter().filter(|(k, _)| *k != "FUEL").all(|(_, &v)| v == 0) {
                break;
            }
        }

        let total_ore_used = self.ore_usage;

        self.ore_usage = 0;

        Ok((fuel_produced, total_ore_used))
    }

    fn wipe_everything(&mut self) {
        self.material_counts = self.material_counts.keys().map(|k| (k.clone(), 0)).collect();
        self.ore_usage = 0;
    }

    fn max_fuel_output(&mut self, lower_limit: usize) -> Result<usize> {
        // let mut total_fuel_produced = 0;
        // answer in [955_000, 955_000 + 1_048_576]
        // binary search

        // let upper_limit = lower_limit + 1_048_576;

        let mut current_guess = lower_limit;
        let mut jump = 1048576/2;

        let mut result = 0;

        loop {
            self.wipe_everything();

            if self._create("FUEL".to_string(), current_guess).is_ok() {
                println!("{} fuel needed {} ore", current_guess, self.ore_usage);
                result = current_guess;
                current_guess += jump;
            } else {
                println!("Cannot make {} fuel", current_guess);
                current_guess -= jump;
            }

            if jump == 0 {
                return Ok(result);
            }

            jump /= 2;
        }
    }
}

pub fn q1(fname: String) -> usize {
    let mut f = File::open(fname).expect("File not found");
    let mut f_contents = String::new();

    f.read_to_string(&mut f_contents).expect("Couldn't find file");

    let recipes: Vec<String> = f_contents.trim().lines().map(|s| s.trim().to_string()).collect();

    _q1(recipes).unwrap()
}


fn _q1(recipes: Vec<String>) -> Result<usize> {
    let mut nanofactory = Nanofactory::new(recipes)?;

    nanofactory.produce_one_fuel()?;

    Ok(nanofactory.ore_usage)
}


pub fn q2(fname: String) -> usize {
    let mut f = File::open(fname).expect("File not found");
    let mut f_contents = String::new();

    f.read_to_string(&mut f_contents).expect("Couldn't find file");

    let recipes: Vec<String> = f_contents.trim().lines().map(|s| s.trim().to_string()).collect();

    _q2(recipes).unwrap()
}


fn _q2(recipes: Vec<String>) -> Result<usize> {
    let mut nanofactory = Nanofactory::new(recipes)?;

    // first, get the general lower bound for what to guess
    // 1 trillion / amount to make 1 fuel
    // using this, x - (x % 10000)
    nanofactory.produce_one_fuel()?;
    println!("Ore usage for one fuel is {}", nanofactory.ore_usage);
    let lower_bound = 1_000_000_000_000 / nanofactory.ore_usage;
    println!("Initial lower bound is {}", lower_bound);
    nanofactory.wipe_everything();
    let lower_bound = lower_bound - (lower_bound % 10_000);

    println!("Initial lower bound is now {}", lower_bound);

    let max_fuel = nanofactory.max_fuel_output(lower_bound)?;

    Ok(max_fuel)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day14_q1_test1() {
        let recipe_list : Vec<String> = "
            10 ORE => 10 A
            1 ORE => 1 B
            7 A, 1 B => 1 C
            7 A, 1 C => 1 D
            7 A, 1 D => 1 E
            7 A, 1 E => 1 FUEL
        ".trim().lines().map(|l| l.trim().to_string()).collect();

        assert_eq!(
            _q1(recipe_list).unwrap(),
            31
        )
    }

    #[test]
    fn day14_q1_test2() {
        let recipe_list : Vec<String> = "
            9 ORE => 2 A
            8 ORE => 3 B
            7 ORE => 5 C
            3 A, 4 B => 1 AB
            5 B, 7 C => 1 BC
            4 C, 1 A => 1 CA
            2 AB, 3 BC, 4 CA => 1 FUEL
        ".trim().lines().map(|l| l.trim().to_string()).collect();

        assert_eq!(
            _q1(recipe_list).unwrap(),
            165
        )
    }

    #[test]
    fn day14_q1_test3() {
        let recipe_list : Vec<String> = "
            157 ORE => 5 NZVS
            165 ORE => 6 DCFZ
            44 XJWVT, 5 KHKGT, 1 QDVJ, 29 NZVS, 9 GPVTF, 48 HKGWZ => 1 FUEL
            12 HKGWZ, 1 GPVTF, 8 PSHF => 9 QDVJ
            179 ORE => 7 PSHF
            177 ORE => 5 HKGWZ
            7 DCFZ, 7 PSHF => 2 XJWVT
            165 ORE => 2 GPVTF
            3 DCFZ, 7 NZVS, 5 HKGWZ, 10 PSHF => 8 KHKGT
        ".trim().lines().map(|l| l.trim().to_string()).collect();

        assert_eq!(
            _q1(recipe_list).unwrap(),
            13312
        )
    }

    #[test]
    fn day14_q1_test4() {
        let recipe_list : Vec<String> = "
            2 VPVL, 7 FWMGM, 2 CXFTF, 11 MNCFX => 1 STKFG
            17 NVRVD, 3 JNWZP => 8 VPVL
            53 STKFG, 6 MNCFX, 46 VJHF, 81 HVMC, 68 CXFTF, 25 GNMV => 1 FUEL
            22 VJHF, 37 MNCFX => 5 FWMGM
            139 ORE => 4 NVRVD
            144 ORE => 7 JNWZP
            5 MNCFX, 7 RFSQX, 2 FWMGM, 2 VPVL, 19 CXFTF => 3 HVMC
            5 VJHF, 7 MNCFX, 9 VPVL, 37 CXFTF => 6 GNMV
            145 ORE => 6 MNCFX
            1 NVRVD => 8 CXFTF
            1 VJHF, 6 MNCFX => 4 RFSQX
            176 ORE => 6 VJHF
        ".trim().lines().map(|l| l.trim().to_string()).collect();

        assert_eq!(
            _q1(recipe_list).unwrap(),
            180697
        )
    }

    #[test]
    fn day14_q1_test5() {
        let recipe_list : Vec<String> = "
            171 ORE => 8 CNZTR
            7 ZLQW, 3 BMBT, 9 XCVML, 26 XMNCP, 1 WPTQ, 2 MZWV, 1 RJRHP => 4 PLWSL
            114 ORE => 4 BHXH
            14 VRPVC => 6 BMBT
            6 BHXH, 18 KTJDG, 12 WPTQ, 7 PLWSL, 31 FHTLT, 37 ZDVW => 1 FUEL
            6 WPTQ, 2 BMBT, 8 ZLQW, 18 KTJDG, 1 XMNCP, 6 MZWV, 1 RJRHP => 6 FHTLT
            15 XDBXC, 2 LTCX, 1 VRPVC => 6 ZLQW
            13 WPTQ, 10 LTCX, 3 RJRHP, 14 XMNCP, 2 MZWV, 1 ZLQW => 1 ZDVW
            5 BMBT => 4 WPTQ
            189 ORE => 9 KTJDG
            1 MZWV, 17 XDBXC, 3 XCVML => 2 XMNCP
            12 VRPVC, 27 CNZTR => 2 XDBXC
            15 KTJDG, 12 BHXH => 5 XCVML
            3 BHXH, 2 VRPVC => 7 MZWV
            121 ORE => 7 VRPVC
            7 XCVML => 6 RJRHP
            5 BHXH, 4 VRPVC => 5 LTCX
        ".trim().lines().map(|l| l.trim().to_string()).collect();

        assert_eq!(
            _q1(recipe_list).unwrap(),
            2210736
        )
    }

    #[test]
    fn day14_q2_test1() {
        let recipe_list : Vec<String> = "
            157 ORE => 5 NZVS
            165 ORE => 6 DCFZ
            44 XJWVT, 5 KHKGT, 1 QDVJ, 29 NZVS, 9 GPVTF, 48 HKGWZ => 1 FUEL
            12 HKGWZ, 1 GPVTF, 8 PSHF => 9 QDVJ
            179 ORE => 7 PSHF
            177 ORE => 5 HKGWZ
            7 DCFZ, 7 PSHF => 2 XJWVT
            165 ORE => 2 GPVTF
            3 DCFZ, 7 NZVS, 5 HKGWZ, 10 PSHF => 8 KHKGT
        ".trim().lines().map(|l| l.trim().to_string()).collect();

        assert_eq!(
            _q2(recipe_list).unwrap(),
            82892753
        )
    }

    #[test]
    fn day14_q2_test2() {
        let recipe_list : Vec<String> = "
            2 VPVL, 7 FWMGM, 2 CXFTF, 11 MNCFX => 1 STKFG
            17 NVRVD, 3 JNWZP => 8 VPVL
            53 STKFG, 6 MNCFX, 46 VJHF, 81 HVMC, 68 CXFTF, 25 GNMV => 1 FUEL
            22 VJHF, 37 MNCFX => 5 FWMGM
            139 ORE => 4 NVRVD
            144 ORE => 7 JNWZP
            5 MNCFX, 7 RFSQX, 2 FWMGM, 2 VPVL, 19 CXFTF => 3 HVMC
            5 VJHF, 7 MNCFX, 9 VPVL, 37 CXFTF => 6 GNMV
            145 ORE => 6 MNCFX
            1 NVRVD => 8 CXFTF
            1 VJHF, 6 MNCFX => 4 RFSQX
            176 ORE => 6 VJHF
        ".trim().lines().map(|l| l.trim().to_string()).collect();

        assert_eq!(
            _q2(recipe_list).unwrap(),
            5586022
        )
    }

    #[test]
    fn day14_q2_test3() {
        let recipe_list : Vec<String> = "
            171 ORE => 8 CNZTR
            7 ZLQW, 3 BMBT, 9 XCVML, 26 XMNCP, 1 WPTQ, 2 MZWV, 1 RJRHP => 4 PLWSL
            114 ORE => 4 BHXH
            14 VRPVC => 6 BMBT
            6 BHXH, 18 KTJDG, 12 WPTQ, 7 PLWSL, 31 FHTLT, 37 ZDVW => 1 FUEL
            6 WPTQ, 2 BMBT, 8 ZLQW, 18 KTJDG, 1 XMNCP, 6 MZWV, 1 RJRHP => 6 FHTLT
            15 XDBXC, 2 LTCX, 1 VRPVC => 6 ZLQW
            13 WPTQ, 10 LTCX, 3 RJRHP, 14 XMNCP, 2 MZWV, 1 ZLQW => 1 ZDVW
            5 BMBT => 4 WPTQ
            189 ORE => 9 KTJDG
            1 MZWV, 17 XDBXC, 3 XCVML => 2 XMNCP
            12 VRPVC, 27 CNZTR => 2 XDBXC
            15 KTJDG, 12 BHXH => 5 XCVML
            3 BHXH, 2 VRPVC => 7 MZWV
            121 ORE => 7 VRPVC
            7 XCVML => 6 RJRHP
            5 BHXH, 4 VRPVC => 5 LTCX
        ".trim().lines().map(|l| l.trim().to_string()).collect();

        assert_eq!(
            _q2(recipe_list).unwrap(),
            460664
        )
    }
}
