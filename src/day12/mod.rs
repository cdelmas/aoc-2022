use anyhow::Result;
use petgraph::algo::dijkstra;
use petgraph::graph::NodeIndex;
use petgraph::Graph;

use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

const ROAD_START: char = 'S';
const ROAD_END: char = 'E';
const LOWEST_ELEVATION: char = 'a';

type Point = (usize, usize);

type RoadMap = Graph<Point, ()>;

type Location = NodeIndex;

#[derive(Debug)]
struct Journey {
    paths: RoadMap,
    possible_starts: Vec<Location>,
    end: Location,
}

impl Journey {
    fn new(paths: RoadMap, possible_starts: Vec<Location>, end: Location) -> Self {
        Self {
            paths,
            possible_starts,
            end,
        }
    }

    fn path_hops(&self) -> Option<usize> {
        self.possible_starts
            .iter()
            .filter_map(|start_node| {
                let distance_map = dijkstra(&self.paths, *start_node, Some(self.end), |_| 1);
                distance_map.get(&self.end).copied()
            })
            .min()
    }
}

#[derive(thiserror::Error, Debug)]
#[error("Path not found")]
enum Error {
    PathNotFound,
}

type Elevation = i32;

fn to_elevation(c: char) -> Elevation {
    match c {
        ROAD_START => b'a' as Elevation,
        ROAD_END => b'z' as Elevation,
        c if c.is_ascii_lowercase() => c as Elevation,
        _ => unreachable!(), // well, normally…
    }
}

fn build_journey(map: &Vec<Vec<char>>) -> Journey {
    let mut end_node = None;
    let mut possible_starts = vec![];
    let width = map[0].len();
    let mut graph = RoadMap::with_capacity(width * map.len(), width * map.len() / 2);

    for i in 0..map.len() {
        for j in 0..map[i].len() {
            let elevation = to_elevation(map[i][j]);
            let node = graph.add_node((i, j));
            if map[i][j] == ROAD_END {
                end_node = Some(node);
            } else if elevation == to_elevation(LOWEST_ELEVATION) {
                possible_starts.push(node);
            }
            if i > 0 {
                let neighbour_elevation = to_elevation(map[i - 1][j]);
                let neighbour = NodeIndex::new((i - 1) * width + j);
                if (neighbour_elevation - elevation) <= 1 {
                    graph.add_edge(node, neighbour, ());
                }
                if (elevation - neighbour_elevation) <= 1 {
                    graph.add_edge(neighbour, node, ());
                }
            }
            if j > 0 {
                let neighbour_elevation = to_elevation(map[i][j - 1]);
                let neighbour = NodeIndex::new(i * width + j - 1);
                if (neighbour_elevation - elevation) <= 1 {
                    graph.add_edge(node, neighbour, ());
                }
                if (elevation - neighbour_elevation) <= 1 {
                    graph.add_edge(neighbour, node, ());
                }
            }
        }
    }
    Journey::new(graph, possible_starts, end_node.unwrap()) // here we unwrap as we are sure (really??) that we find the start and end
}

pub fn great_journey(input: &PathBuf) -> Result<usize> {
    let file = File::open(input)?;
    let reader = BufReader::new(file);
    let map: Vec<Vec<char>> = reader
        .lines()
        .filter_map(std::io::Result::ok)
        .map(|v| v.chars().collect())
        .collect();

    let journey = build_journey(&map);

    journey
        .path_hops()
        .ok_or_else(|| Error::PathNotFound.into())
}

#[cfg(test)]
mod tests {

    use super::*;
    use spectral::prelude::*;

    #[test]
    fn path_length() {
        let map = vec![
            vec!['S', 'a', 'b', 'q', 'p', 'o', 'n', 'm'],
            vec!['a', 'b', 'c', 'r', 'y', 'x', 'x', 'l'],
            vec!['a', 'c', 'c', 's', 'z', 'E', 'x', 'k'],
            vec!['a', 'c', 'c', 't', 'u', 'v', 'w', 'j'],
            vec!['a', 'b', 'd', 'e', 'f', 'g', 'h', 'i'],
        ];

        let journey = build_journey(&map);

        let hops = journey.path_hops();

        assert_that!(hops).is_some().is_equal_to(29);
    }
}
