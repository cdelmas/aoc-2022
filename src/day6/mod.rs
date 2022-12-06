use anyhow::Result;
use std::collections::BTreeSet;
use std::fs::read_to_string;
use std::path::PathBuf;
use thiserror::Error;

const START_MARKER_SIZE: usize = 4;
const MESSAGE_MARKER_SIZE: usize = 14;

#[derive(Error, Debug)]
#[error("Could not find the marker")]
struct NotFoundError;

fn find_marker(buffer: &Vec<char>, marker_len: usize) -> Result<usize> {
    buffer
        .windows(marker_len)
        .position(|s| s.iter().collect::<BTreeSet<&char>>().len() == s.len())
        .map(|i| i + marker_len)
        .ok_or(NotFoundError.into())
}

pub fn fix_device(input: &PathBuf) -> Result<(usize, usize)> {
    let content = read_to_string(input)?;
    let buffer = content.chars().collect::<Vec<_>>();
    let start_stream = find_marker(&buffer, START_MARKER_SIZE)?;
    let start_message = find_marker(&buffer, MESSAGE_MARKER_SIZE)?;
    Ok((start_stream, start_message))
}

#[cfg(test)]
mod tests {

    use super::*;
    use parameterized::parameterized;

    #[parameterized(
        input = {
            "bvwbjplbgvbhsrlpgdmjqwftvncz", "nppdvjthqldpwncqszvftbrmjlhg", "nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg", "zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw",
            "mjqjpqmgbljsphdztnvjfqwrcgsmlb", "bvwbjplbgvbhsrlpgdmjqwftvncz", "nppdvjthqldpwncqszvftbrmjlhg", "nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg",
            "zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw"
        },
        marker_size = {
            4,4,4,4,14,14,14,14,14
        },
        index = {
            5,6,10,11,19,23,23,29,26
        })
    ]
    fn marker_size_tests(input: &str, marker_size: usize, index: usize) {
        let v: Vec<_> = String::from(input).chars().collect();

        let res = find_marker(&v, marker_size).unwrap();

        assert_eq!(res, index);
    }
}
