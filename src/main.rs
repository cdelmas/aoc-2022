mod day1;
mod day10;
mod day11;
mod day2;
mod day3;
mod day4;
mod day5;
mod day6;
mod day7;
mod day8;
mod day9;

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

    let ship_unload_overlaps = day4::ship_unload_overlaps(&PathBuf::from("data/day_4_input.txt"));
    match ship_unload_overlaps {
        Ok(ship_unload_overlaps) => println!("Overlaps: {}", ship_unload_overlaps),
        Err(_) => eprintln!("Something went wrong…"),
    }

    let top_crate_of_stacks = day5::top_crate_of_stacks(&PathBuf::from("data/day_5_input.txt"));
    match top_crate_of_stacks {
        Ok(top_crate_of_stacks) => println!("Top crates of stacks: {}", top_crate_of_stacks),
        Err(_) => eprintln!("Something went wrong…"),
    }

    let markers = day6::fix_device(&PathBuf::from("data/day_6_input.txt"));
    match markers {
        Ok((start_stream, start_message)) => println!(
            "Markers: start stream at {}, message at {}",
            start_stream, start_message
        ),
        Err(_) => eprintln!("Something went wrong…"),
    }

    let small_directories = day7::total_size_of_small_directories_and_smallest_to_delete(
        &PathBuf::from("data/day_7_input.txt"),
    );
    match small_directories {
        Ok((total_small_directories_size, smallest_to_delete_size)) => {
            println!(
                "Total size of small directories: {}; smallest to delete: {}",
                total_small_directories_size, smallest_to_delete_size
            )
        }
        Err(_) => eprintln!("Something went wrong…"),
    }

    let spot = day8::find_best_spot_for_tree_house(&PathBuf::from("data/day_8_input.txt"));
    match spot {
        Ok((visible_trees, best_spot)) => println!(
            "{} visible trees around, {} is the best spot",
            visible_trees, best_spot
        ),
        Err(_) => eprintln!("Something went wrong…"),
    }

    let nb_tail_positions = day9::nb_tail_positions(&PathBuf::from("data/day_9_input.txt"));
    match nb_tail_positions {
        Ok(nb_tail_positions) => {
            println!("Tail gone through {} positions buggy 🙈", nb_tail_positions)
        }
        Err(_) => eprintln!("Something went wrong…"),
    }

    let signal_strength = day10::sum_of_signal_strengths(&PathBuf::from("data/day_10_input.txt"));
    match signal_strength {
        Ok(signal_strength) => println!("Signal strength total: {}", signal_strength),
        Err(_) => eprintln!("Something went wrong…"),
    }
}
