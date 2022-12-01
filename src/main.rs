mod day1;

use std::path::PathBuf;

fn main() {
    let calories = day1::calories_carried(&PathBuf::from("data/day_1_input.txt"));
    match calories {
        Ok(calories) => println!("{} calories brought by the most loaded elf", calories),
        Err(_) => eprintln!("Something went wrong…"),
    }
}
