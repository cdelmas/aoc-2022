use anyhow::Result;
use itertools::Itertools;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

pub fn calories_carried(input: &PathBuf) -> Result<u32> {
    let file = File::open(input)?;
    let reader = BufReader::new(file);
    Ok(reader
        .lines()
        .filter_map(std::io::Result::ok)
        .collect::<Vec<String>>()
        .split(|s| s.is_empty())
        .map(|sl| sl.iter().filter_map(|e| e.parse::<u32>().ok()).sum::<u32>())
        .sorted()
        .rev()
        .take(3)
        .sum::<u32>())
}
