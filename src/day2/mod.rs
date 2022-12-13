use itertools::{process_results, Itertools};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Shape {
    Rock,
    Paper,
    Scissors,
}

#[derive(Error, Debug)]
#[error("cannot parse")]
struct ParseError;

impl FromStr for Shape {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Shape, Self::Err> {
        match s {
            "A" => Ok(Shape::Rock),
            "B" => Ok(Shape::Paper),
            "C" => Ok(Shape::Scissors),
            _ => Err(ParseError {}),
        }
    }
}

#[derive(Debug)]
enum GameResult {
    Loss,
    Draw,
    Won,
}

#[derive(Debug, PartialEq)]
enum Should {
    Lose,
    Draw,
    Win,
}

impl FromStr for Should {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Should, Self::Err> {
        match s {
            "X" => Ok(Should::Lose),
            "Y" => Ok(Should::Draw),
            "Z" => Ok(Should::Win),
            _ => Err(ParseError {}),
        }
    }
}

type Hint = (Shape, Should);

type Game = (Shape, Shape);

fn what_to_play(hint: &Hint) -> Shape {
    match hint {
        (Shape::Rock, Should::Lose) => Shape::Scissors,
        (Shape::Rock, Should::Win) => Shape::Paper,
        (Shape::Paper, Should::Lose) => Shape::Rock,
        (Shape::Paper, Should::Win) => Shape::Scissors,
        (Shape::Scissors, Should::Lose) => Shape::Paper,
        (Shape::Scissors, Should::Win) => Shape::Rock,
        (p, Should::Draw) => *p,
    }
}

fn parse_game(s: &str) -> anyhow::Result<Game, ParseError> {
    let parts: Vec<&str> = s.split(' ').collect();
    if parts.len() != 2 {
        Err(ParseError {})
    } else {
        let opponent_play = parts[0].parse::<Shape>()?;
        let hint = parts[1].parse::<Should>()?;
        Ok((opponent_play, what_to_play(&(opponent_play, hint))))
    }
}

fn _parse_game_old(s: &str) -> anyhow::Result<Game, ParseError> {
    process_results(s.split(' ').map(Shape::from_str), |iter| {
        iter.collect_tuple().unwrap_or((Shape::Rock, Shape::Rock))
    })
}

fn run_game(game: &Game) -> GameResult {
    match game {
        (Shape::Rock, Shape::Paper) => GameResult::Won,
        (Shape::Rock, Shape::Scissors) => GameResult::Loss,
        (Shape::Paper, Shape::Rock) => GameResult::Loss,
        (Shape::Paper, Shape::Scissors) => GameResult::Won,
        (Shape::Scissors, Shape::Rock) => GameResult::Won,
        (Shape::Scissors, Shape::Paper) => GameResult::Loss,
        _ => GameResult::Draw,
    }
}

fn shape_score(game: &Game) -> u32 {
    match game {
        (_, Shape::Rock) => 1,
        (_, Shape::Paper) => 2,
        (_, Shape::Scissors) => 3,
    }
}

fn score(game: &Game) -> u32 {
    let shape_score = shape_score(game);
    let outcome_score = match run_game(game) {
        GameResult::Won => 6,
        GameResult::Draw => 3,
        GameResult::Loss => 0,
    };
    shape_score + outcome_score
}

pub fn rock_paper_scissors(input: &PathBuf) -> anyhow::Result<u32> {
    let file = File::open(input)?;
    let reader = BufReader::new(file);
    let mut my_score = 0;
    for line in reader.lines() {
        let line = line?;
        let game = parse_game(&line)?;
        let game_score = score(&game);
        my_score += game_score;
    }

    Ok(my_score)
}
