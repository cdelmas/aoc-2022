use anyhow::Result;
use itertools::Itertools;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

fn count_visible_trees(data: &Vec<u8>, map_size: (usize, usize)) -> u16 {
    let (nb_rows, nb_columns) = map_size;
    let mut visible_trees = 0;
    for (i, c) in data.iter().enumerate() {
        let to_north = (i % nb_columns)..i;
        let to_south = (i + nb_columns)..((nb_rows * nb_columns) + (i % nb_columns));
        let to_west = (i - i % nb_columns)..i;
        let to_east = (i + 1..(i - (i % nb_columns) + nb_columns));
        if to_north
            .into_iter()
            .step_by(nb_columns)
            .all(|ix| data[ix] < *c)
            || to_south
                .into_iter()
                .step_by(nb_columns)
                .all(|ix| data[ix] < *c)
            || to_west.into_iter().all(|ix| data[ix] < *c)
            || to_east.into_iter().all(|ix| data[ix] < *c)
        {
            visible_trees += 1;
        }
    }

    visible_trees
}

fn find_best_spot(data: &Vec<u8>, map_size: (usize, usize)) -> u32 {
    let (nb_rows, nb_columns) = map_size;
    let mut visible_trees = 0;
    data.iter()
        .enumerate()
        .map(|(i, c)| {
            let to_north = (i % nb_columns)..i;
            let to_south = (i + nb_columns)..((nb_rows * nb_columns) + (i % nb_columns));
            let to_west = (i - i % nb_columns)..i;
            let to_east = (i + 1..(i - (i % nb_columns) + nb_columns));

            let north_score = to_north
                .into_iter()
                .step_by(nb_columns)
                .rev()
                .fold((0, true), |(count, counting), ix| {
                    if counting {
                        (count + 1, data[ix] < *c)
                    } else {
                        (count, false)
                    }
                })
                .0;
            let south_score = to_south
                .into_iter()
                .step_by(nb_columns)
                .fold((0, true), |(count, counting), ix| {
                    if counting {
                        (count + 1, data[ix] < *c)
                    } else {
                        (count, false)
                    }
                })
                .0;
            let west_score = to_west
                .into_iter()
                .rev()
                .fold((0, true), |(count, counting), ix| {
                    if counting {
                        (count + 1, data[ix] < *c)
                    } else {
                        (count, false)
                    }
                })
                .0;
            let east_score = to_east
                .into_iter()
                .fold((0, true), |(count, counting), ix| {
                    if counting {
                        (count + 1, data[ix] < *c)
                    } else {
                        (count, false)
                    }
                })
                .0;
            north_score * south_score * west_score * east_score
        })
        .max()
        .unwrap_or(0)
}

pub fn find_best_spot_for_tree_house(input: &PathBuf) -> Result<(u16, u32)> {
    let f = File::open(input)?;
    let mut reader = BufReader::new(f);
    let mut raw_data = vec![];
    let size = reader.read_to_end(&mut raw_data)?;
    let nb_rows: usize = raw_data.iter().filter(|c| **c == b'\n').count() + 1;
    let nb_columns: usize = size / nb_rows;

    let data = raw_data
        .into_iter()
        .filter(|c| *c != b'\n' && *c != b'\r')
        .collect::<Vec<_>>();

    let visible_trees = count_visible_trees(&data, (nb_rows, nb_columns));
    let best_spot = find_best_spot(&data, (nb_rows, nb_columns));

    Ok((visible_trees, best_spot))
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn best_spot() {
        let data = "3037325512653323354935390".bytes().collect::<Vec<u8>>();

        let score = find_best_spot(&data, (5, 5));

        assert_eq!(score, 8);
    }
}
