mod day1;
mod day2;
mod day3;

use std::path::PathBuf;

fn main() {
    // day1
    let calories = day1::calories_carried(&PathBuf::from("data/day_1_input.txt"));
    match calories {
        Ok(calories) => println!("{} calories brought by the most loaded elf", calories),
        Err(_) => eprintln!("Something went wrong…"),
    }

    // day2
    let score = day2::rock_paper_scissors(&PathBuf::from("data/day_2_input.txt"));
    match score {
        Ok(score) => println!("Rock Paper Scissors score={}", score),
        Err(_) => eprintln!("Something went wrong…"),
    }

    // day3
    let priorities = day3::priorities(&PathBuf::from("data/day_3_input.txt"));
    match priorities {
        Ok(priorities) => println!("total priorities: {}", priorities),
        Err(_) => eprintln!("Something went wrong…"),
    }

    let priorities_2 = day3::priorities_2(&PathBuf::from("data/day_3_part2_input.txt"));
    match priorities_2 {
        Ok(priorities_2) => println!("total priorities: {}", priorities_2),
        Err(_) => eprintln!("Something went wrong…"),
    }
}
