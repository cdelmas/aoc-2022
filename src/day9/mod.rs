use anyhow::Result;
use dendron::{traverse::DftEvent::Close, tree::HierarchyEditGrantError, tree_node, Node};
use itertools::Itertools;
use nom::{
    branch::alt,
    character::complete::{char, line_ending, u8},
    combinator::{eof, map},
    error::{ErrorKind, FromExternalError, ParseError},
    multi::many1,
    sequence::{separated_pair, terminated, tuple},
    IResult,
};
use std::collections::BTreeSet;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::fs::read_to_string;
use std::num::ParseIntError;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, PartialEq)]
enum Move {
    Up(u8),
    Down(u8),
    Right(u8),
    Left(u8),
}

impl Move {
    fn on_x(val: i16) -> Self {
        if val < 0 {
            Move::Left(val.abs() as u8)
        } else {
            Move::Right(val.abs() as u8)
        }
    }

    fn on_y(val: i16) -> Self {
        if val < 0 {
            Move::Down(val.abs() as u8)
        } else {
            Move::Up(val.abs() as u8)
        }
    }

    fn small_step(mv: &Self) -> Self {
        match mv {
            Move::Up(d) => Move::Up(if *d != 0 { 1 } else { 0 }),
            Move::Down(d) => Move::Down(if *d != 0 { 1 } else { 0 }),
            Move::Left(d) => Move::Left(if *d != 0 { 1 } else { 0 }),
            Move::Right(d) => Move::Right(if *d != 0 { 1 } else { 0 }),
        }
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Move::Up(d) => write!(f, "upwards by {}", d),
            Move::Down(d) => write!(f, "downwards by {}", d),
            Move::Left(d) => write!(f, "to the left by {}", d),
            Move::Right(d) => write!(f, "to the right by {}", d),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
struct Position((i16, i16));

impl Display for Position {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{:?}", self.0)
    }
}

impl Position {
    fn new(point: (i16, i16)) -> Self {
        Self(point)
    }

    fn x(self: &Self) -> i16 {
        self.0 .0
    }

    fn y(self: &Self) -> i16 {
        self.0 .1
    }

    fn move_to(self: &mut Self, mv: &Move) {
        match mv {
            Move::Up(d) => self.0 = (self.x(), self.y() + *d as i16),
            Move::Down(d) => self.0 = (self.x(), self.y() - *d as i16),
            Move::Left(d) => self.0 = (self.x() - *d as i16, self.y()),
            Move::Right(d) => self.0 = (self.x() + *d as i16, self.y()),
        }
    }

    fn move_next_to(self: &mut Self, target: &Self) -> Vec<Position> {
        let mut tracker = vec![];
        while !self.is_around(target) {
            let moves = self.path_to(target);
            moves.iter().for_each(|mv| {
                self.move_to(mv);
            });
            tracker.push(*self);
        }
        tracker
    }

    fn path_to(self: &Self, other: &Self) -> Vec<Move> {
        match (other.x() - self.x(), other.y() - self.y()) {
            (0, 0) => vec![],
            (0, y) => vec![Move::small_step(&Move::on_y(y))],
            (x, 0) => vec![Move::small_step(&Move::on_x(x))],
            (x, y) => vec![
                Move::small_step(&Move::on_x(x)),
                Move::small_step(&Move::on_y(y)),
            ],
        }
    }

    fn is_around(self: &Self, other: &Self) -> bool {
        (self.x() - other.x()).abs() <= 1 && (self.y() - other.y()).abs() <= 1
    }
}

fn move_statement<'a, E>(i: &'a str) -> IResult<&'a str, Move, E>
where
    E: ParseError<&'a str>,
{
    map(
        separated_pair(
            alt((char('R'), char('U'), char('D'), char('L'))),
            char(' '),
            u8,
        ),
        |(direction, distance)| match direction {
            'R' => Move::Right(distance),
            'D' => Move::Down(distance),
            'U' => Move::Up(distance),
            'L' => Move::Left(distance),
            _ => panic!("Should not happen as the parser only accept R,U,L,D"),
        },
    )(i)
}

fn moves<'a, E>(i: &'a str) -> IResult<&'a str, Vec<Move>, E>
where
    E: ParseError<&'a str>,
{
    many1(terminated(move_statement, alt((line_ending, eof))))(i)
}

const ROPE_SIZE: usize = 10;
const TAIL_INDEX: usize = ROPE_SIZE - 1;
const HEAD_INDEX: usize = 0;

fn move_rope(head_moves: &[Move]) -> usize {
    let mut tail_visits: BTreeSet<Position> = BTreeSet::new();
    let mut rope: Vec<Position> = vec![Position::default(); ROPE_SIZE];
    tail_visits.insert(rope[TAIL_INDEX]);
    for mv in head_moves {
        //println!("Moving head from {} {}", rope[HEAD_INDEX], mv);
        rope[HEAD_INDEX].move_to(mv);
        //println!("Now at {}", rope[HEAD_INDEX]);
        for i in 1..ROPE_SIZE {
            let mut local_head = rope[i - 1];
            let mut local_tail = rope[i];
            /*println!(
                "Moving part {} from {} next to {}",
                i, local_tail, local_head
            );*/
            let tracker = local_tail.move_next_to(&local_head);
            //println!("part {} moved to {}", i, local_tail);
            rope[i] = local_tail;
            if i == TAIL_INDEX {
                // track rope’s tail position
                tracker.into_iter().for_each(|p| {
                    //println!("Moving tail to {}", p);
                    tail_visits.insert(p);
                });
            }
        }
        //println!("Rope now at {:?}", rope);
    }
    tail_visits.len()
}

pub fn nb_tail_positions(input: &PathBuf) -> Result<usize> {
    let data = read_to_string(input)?;
    let (_, moves) = moves::<()>(&data)?;

    Ok(move_rope(&moves))
}

#[cfg(test)]
mod tests {

    use super::*;
    use parameterized::parameterized;

    #[test]
    fn parse_moves() -> Result<()> {
        let commands = "U 3\nR 1\nD 2\nL 4\n";

        let (_, moves) = moves::<()>(&commands)?;

        assert_eq!(
            moves,
            vec![Move::Up(3), Move::Right(1), Move::Down(2), Move::Left(4)]
        );
        Ok(())
    }

    #[parameterized(
        mv = {
            &Move::Up(3), &Move::Down(5), &Move::Left(10), &Move::Right(1)
        },
        expected_position = {
            &Position::new((0, 3)), &Position::new((0, -5)), &Position::new((-10, 0)), &Position::new((1, 0))
        })
    ]
    fn move_position(mv: &Move, expected_position: &Position) {
        let mut position = Position::default();

        position.move_to(&mv);

        assert_eq!(position, *expected_position);
    }

    #[parameterized(
        other = {
            &Position::new((0, 1)),
            &Position::new((1, 1)),
            &Position::new((1, 0)),
            &Position::new((0, 0)),
            &Position::new((0, -1)),
            &Position::new((-1, 0)),
            &Position::new((1, -1)),
            &Position::new((-1, 1)),
            &Position::new((-1, -1)),
        }
    )]
    fn are_around(other: &Position) {
        let position = Position::default();
        assert!(
            other.is_around(&position),
            "{:?} is not around {:?}",
            other,
            position
        );
    }

    #[parameterized(
        other = {
            &Position::new((0, 2)),
            &Position::new((5, 1)),
            &Position::new((1, 2)),
            &Position::new((6, 0)),
            &Position::new((1, -2)),
            &Position::new((-2, -2)),
            &Position::new((4, 0)),
            &Position::new((-2, 1)),
            &Position::new((-1, -2)),
        }
    )]
    fn are_not_around(other: &Position) {
        let position = Position::default();
        assert!(
            !other.is_around(&position),
            "{:?} is around {:?}",
            other,
            position
        );
    }

    #[parameterized(
        target = {
            &Position::new((2,0)), &Position::new((2,1)),  &Position::new((2,2)),  &Position::new((1,2)),
            &Position::new((0,2)), &Position::new((-1,2)), &Position::new((-2,2)), &Position::new((-2,1)),
            &Position::new((-2,0)),&Position::new((-2,-1)),&Position::new((-2,-2)),&Position::new((-1,-2)),
            &Position::new((0,-2)),&Position::new((1,-2)), &Position::new((2,-2)), &Position::new((2,-1)),
        },
        expected_position = {
            &Position::new((1,0)), &Position::new((1,1)),  &Position::new((1,1)),  &Position::new((1,1)),
            &Position::new((0,1)), &Position::new((-1,1)), &Position::new((-1,1)), &Position::new((-1,1)),
            &Position::new((-1,0)),&Position::new((-1,-1)),&Position::new((-1,-1)),&Position::new((-1,-1)),
            &Position::new((0,-1)),&Position::new((1,-1)), &Position::new((1,-1)), &Position::new((1,-1)),
        }
    )]
    fn should_move_next_to(target: &Position, expected_position: &Position) {
        let mut to_move = Position::default();

        to_move.move_next_to(&target);

        assert_eq!(to_move, *expected_position);
    }

    #[test]
    fn move_small_rope_test() {
        let moves = vec![
            Move::Right(4),
            Move::Up(4),
            Move::Left(3),
            Move::Down(1),
            Move::Right(4),
            Move::Down(1),
            Move::Left(5),
            Move::Right(2),
        ];

        assert_eq!(move_rope(&moves), 1);
    }

    #[test]
    fn move_big_rope_test() {
        let moves = vec![
            Move::Right(5),
            Move::Up(8),
            Move::Left(8),
            Move::Down(3),
            Move::Right(17),
            Move::Down(10),
            Move::Left(25),
            Move::Up(20),
        ];

        assert_eq!(move_rope(&moves), 36);
    }
}
