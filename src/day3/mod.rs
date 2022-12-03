use anyhow::Result;
use itertools::process_results;
use std::collections::BTreeSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

// parse as string
// chunk in 2 parts
// find union
// convert to priority (using a static map or a magic crate — to find)
// sum

pub fn priority(c: &char) -> u32 {
    let c = *c as u32;
    let p = c - 'A' as u32;
    if p > 27 {
        c - 'a' as u32 + 1
    } else {
        p + 27
    }
}

pub fn priorities(input: &PathBuf) -> Result<u32> {
    let file = File::open(input)?;
    let reader = BufReader::new(file);
    process_results(reader.lines(), |iter| {
        iter.map(|s| {
            let (part1, part2) = s.split_at(s.len() / 2);
            let part1 = part1.chars().collect::<BTreeSet<char>>();
            let part2 = part2.chars().collect::<BTreeSet<char>>();
            let common_item = *part1.intersection(&part2).collect::<Vec<&char>>()[0];
            priority(&common_item)
        })
        .sum::<u32>()
    })
    .map_err(|err| err.into())
}

pub fn priorities_2(input: &PathBuf) -> Result<u32> {
    let file = File::open(input)?;
    let reader = BufReader::new(file);
    process_results(reader.lines(), |iter| {
        iter.collect::<Vec<String>>()
            .chunks(3)
            .map(|s| {
                // arrays_chunks would be better but is nightly only for now
                if let [part1, part2, part3] = s {
                    let part1 = part1.chars().collect::<BTreeSet<char>>();
                    let part2 = part2.chars().collect::<BTreeSet<char>>();
                    let part3 = part3.chars().collect::<BTreeSet<char>>();
                    let common_items_1 = part1.intersection(&part2).collect::<BTreeSet<&char>>();
                    let common_items_2 = part2.intersection(&part3).collect::<BTreeSet<&char>>();
                    let common_item = common_items_1.intersection(&common_items_2).next().unwrap(); // we are sur we have a result, so unwrap is simple
                    priority(&common_item)
                } else {
                    0
                }
            })
            .sum::<u32>()
    })
    .map_err(|err| err.into())
}
