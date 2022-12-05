use anyhow::Result;
use itertools::process_results;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::RangeInclusive;
use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("cannot parse")]
struct ParseError;

fn parse_tokens<T>(s: &str, sep: char) -> Result<Vec<T>, ParseError>
where
    T: FromStr,
{
    process_results(
        s.split(sep)
            .map(|x| x.parse::<T>().map_err(|_| ParseError {})),
        |iter| iter.collect::<Vec<T>>(),
    )
}

fn parse_range<T>(s: &str) -> Result<RangeInclusive<T>, ParseError>
where
    T: Copy + PartialOrd<T> + FromStr,
{
    let bounds = parse_tokens(s, '-');
    match bounds {
        Ok(bounds) => Ok(bounds[0]..=bounds[1]),
        Err(x) => Err(x),
    }
}

fn parse_line<T>(s: &str) -> Result<(RangeInclusive<T>, RangeInclusive<T>), ParseError>
where
    T: Copy + PartialOrd<T> + FromStr,
{
    process_results(s.split(',').map(|e| parse_range::<T>(e)), |iter| {
        let ranges = iter.collect::<Vec<RangeInclusive<T>>>();
        (ranges[0].clone(), ranges[1].clone())
    })
}

fn range_included<T>(range: &RangeInclusive<T>, candidate: &RangeInclusive<T>) -> bool
where
    T: PartialOrd<T>,
{
    range.contains(&candidate.start()) && range.contains(&candidate.end())
}

fn range_overlaps<T>(range: &RangeInclusive<T>, candidate: &RangeInclusive<T>) -> bool
where
    T: PartialOrd<T>,
{
    range.contains(&candidate.start()) || range.contains(&candidate.end())
}

pub fn ship_unload_overlaps(input: &PathBuf) -> Result<u32> {
    let file = File::open(input)?;
    let reader = BufReader::new(file);
    process_results(reader.lines(), |iter| {
        iter.map(|line| parse_line::<u32>(&line).unwrap_or((0..=0, 1..=1)))
            .filter(|(r0, r1)| range_overlaps(r0, r1) || range_overlaps(r1, r0))
            .count() as u32
    })
    .map_err(|err| err.into())
}
