use itertools::{process_results, Itertools};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Play {
    Rock,
    Paper,
    Scissors,
}

#[derive(Error, Debug)]
#[error("cannot parse")]
struct ParseError;

impl FromStr for Play {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Play, Self::Err> {
        match s {
            "A" => Ok(Play::Rock),
            "B" => Ok(Play::Paper),
            "C" => Ok(Play::Scissors),
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

type Hint = (Play, Should);

type Game = (Play, Play);

fn what_to_play(hint: &Hint) -> Play {
    match hint {
        (Play::Rock, Should::Lose) => Play::Scissors,
        (Play::Rock, Should::Win) => Play::Paper,
        (Play::Paper, Should::Lose) => Play::Rock,
        (Play::Paper, Should::Win) => Play::Scissors,
        (Play::Scissors, Should::Lose) => Play::Paper,
        (Play::Scissors, Should::Win) => Play::Rock,
        (p, Should::Draw) => *p,
    }
}

fn parse_game(s: &str) -> anyhow::Result<Game, ParseError> {
    let parts: Vec<&str> = s.split(' ').collect();
    if parts.len() != 2 {
        Err(ParseError {})
    } else {
        let opponent_play = parts[0].parse::<Play>()?;
        let hint = parts[1].parse::<Should>()?;
        Ok((opponent_play, what_to_play(&(opponent_play, hint))))
    }
}

fn parse_game_old(s: &str) -> anyhow::Result<Game, ParseError> {
    process_results(s.split(' ').map(Play::from_str), |iter| {
        iter.collect_tuple().unwrap_or((Play::Rock, Play::Rock))
    })
}

fn run_game(game: &Game) -> GameResult {
    match game {
        (Play::Rock, Play::Paper) => GameResult::Won,
        (Play::Rock, Play::Scissors) => GameResult::Loss,
        (Play::Paper, Play::Rock) => GameResult::Loss,
        (Play::Paper, Play::Scissors) => GameResult::Won,
        (Play::Scissors, Play::Rock) => GameResult::Won,
        (Play::Scissors, Play::Paper) => GameResult::Loss,
        _ => GameResult::Draw,
    }
}

fn shape_score(game: &Game) -> u32 {
    match game {
        (_, Play::Rock) => 1,
        (_, Play::Paper) => 2,
        (_, Play::Scissors) => 3,
    }
}

fn score(game: &Game) -> u32 {
    let shape_score = shape_score(&game);
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
