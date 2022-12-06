use nom::{
    branch::alt,
    bytes::complete::{tag, take_while_m_n},
    character::complete::{char, digit1, line_ending, satisfy},
    combinator::{map_res, success},
    error::{ErrorKind, FromExternalError, ParseError},
    multi::{many1, separated_list1},
    sequence::{delimited, terminated, tuple},
    IResult,
};

use anyhow::Result;
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::fs::read_to_string;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Clone, Debug, PartialEq)]
struct Crate(char);

impl Display for Crate {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "[{}]", self.0)
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct StackId(char);

impl Display for StackId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, PartialEq)]
struct StacksSpecification {
    slot_lines: Vec<Vec<Option<Crate>>>,
    stack_ids: Vec<StackId>,
}

impl StacksSpecification {
    fn new(crates: Vec<Vec<Option<Crate>>>, stack_ids: Vec<StackId>) -> Self {
        Self {
            slot_lines: crates,
            stack_ids,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Stack {
    id: StackId,
    crates: Vec<Crate>,
}

impl Stack {
    fn new(id: StackId, crates: Vec<Crate>) -> Self {
        Self { id, crates }
    }
}

impl Display for Stack {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{}({})\t:{}",
            self.id,
            self.crates.len(),
            self.crates
                .iter()
                .map(|c| c.to_string())
                .collect::<String>()
        )
    }
}

fn rearrange(stacks: &Vec<Stack>, moves: &[Move]) -> Vec<Stack> {
    let mut stacks: BTreeMap<StackId, Stack> = stacks
        .into_iter()
        .map(|e| (e.id.clone(), e.clone()))
        .collect();
    for m in moves.iter() {
        let mut swap = Vec::with_capacity(m.num.into());
        stacks.entry(m.from.clone()).and_modify(|c| {
            for _ in 0..m.num {
                if let Some(c) = c.crates.pop() {
                    swap.push(c);
                }
            }
        });

        stacks.entry(m.to.clone()).and_modify(|e| {
            e.crates.extend(swap);
        });
    }
    stacks.values().cloned().collect()
}

fn rearrange_part_2(stacks: &Vec<Stack>, moves: &[Move]) -> Vec<Stack> {
    let mut stacks: BTreeMap<StackId, Stack> = stacks
        .into_iter()
        .map(|e| (e.id.clone(), e.clone()))
        .collect();
    for m in moves.iter() {
        let mut swap = Vec::with_capacity(m.num.into());
        stacks.entry(m.from.clone()).and_modify(|c| {
            for _ in 0..m.num {
                if let Some(c) = c.crates.pop() {
                    swap.push(c);
                }
            }
        });

        stacks.entry(m.to.clone()).and_modify(|e| {
            e.crates.extend(swap.into_iter().rev());
        });
    }
    stacks.values().cloned().collect()
}

#[derive(Debug, PartialEq)]
struct Move {
    num: u16,
    from: StackId,
    to: StackId,
}

impl Move {
    fn new(num: u16, from: StackId, to: StackId) -> Self {
        Move { num, from, to }
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{} --{}--> {}", self.from, self.num, self.to)
    }
}

#[derive(Error, Debug)]
#[error("cannot parse")]
struct ElvesParseError;

impl<'a> FromExternalError<&'a str, ElvesParseError> for ElvesParseError {
    fn from_external_error(_input: &'a str, _kind: ErrorKind, e: ElvesParseError) -> Self {
        e
    }
}

fn parse_stack_id<'a, E>(i: &'a str) -> IResult<&'a str, StackId, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, ElvesParseError>,
{
    map_res(digit1, |s: &str| {
        Ok::<StackId, ElvesParseError>(StackId(s.chars().next().unwrap_or('0')))
    })(i)
}

fn parse_crate<'a, E>(i: &'a str) -> IResult<&'a str, Option<Crate>, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, ElvesParseError>,
{
    alt((
        map_res(
            delimited(char('['), satisfy(|c| c.is_ascii_uppercase()), char(']')),
            |id: char| Ok::<Option<Crate>, ElvesParseError>(Some(Crate(id))),
        ),
        map_res(take_while_m_n(3, 3, char::is_whitespace), |_| {
            Ok::<Option<Crate>, ElvesParseError>(None)
        }),
    ))(i)
}

fn parse_stack_def_line<'a, E>(i: &'a str) -> IResult<&'a str, Vec<Option<Crate>>, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, ElvesParseError>,
{
    separated_list1(char(' '), parse_crate)(i)
}

fn parse_stack_def_line_nl<'a, E>(i: &'a str) -> IResult<&'a str, Vec<Option<Crate>>, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, ElvesParseError>,
{
    terminated(parse_stack_def_line, line_ending)(&i)
}

fn parse_stack_id_line<'a, E>(i: &'a str) -> IResult<&'a str, Vec<StackId>, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, ElvesParseError>,
{
    separated_list1(char(' '), delimited(char(' '), parse_stack_id, char(' ')))(i)
}

fn parse_stack_id_line_nl<'a, E>(i: &'a str) -> IResult<&'a str, Vec<StackId>, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, ElvesParseError>,
{
    terminated(parse_stack_id_line, line_ending)(&i)
}

fn parse_move<'a, E>(i: &'a str) -> IResult<&'a str, Move, E>
where
    E: ParseError<&'a str>
        + FromExternalError<&'a str, ElvesParseError>
        + FromExternalError<&'a str, std::num::ParseIntError>,
{
    map_res(
        tuple((
            tag("move "),
            digit1,
            tag(" from "),
            parse_stack_id,
            tag(" to "),
            parse_stack_id,
        )),
        |(_, num, _, from, _, to)| {
            let num = u16::from_str_radix(num, 10).map_err(|_| ElvesParseError {})?; // TODO: implement From<ParseIntError> for ElvesParseError to remove map_err
            Ok::<Move, ElvesParseError>(Move::new(num, from, to))
        },
    )(i)
}

fn parse_move_nl<'a, E>(i: &'a str) -> IResult<&'a str, Move, E>
where
    E: ParseError<&'a str>
        + FromExternalError<&'a str, ElvesParseError>
        + FromExternalError<&'a str, std::num::ParseIntError>,
{
    terminated(parse_move, line_ending)(&i)
}

fn parse_moves<'a, E>(i: &'a str) -> IResult<&'a str, Vec<Move>, E>
where
    E: ParseError<&'a str>
        + FromExternalError<&'a str, ElvesParseError>
        + FromExternalError<&'a str, std::num::ParseIntError>,
{
    many1(parse_move_nl)(i)
}

fn empty_line<'a, E>(i: &'a str) -> IResult<&'a str, &'a str, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, ElvesParseError>,
{
    line_ending(i)
}

fn parse_stacks_specifications<'a, E>(i: &'a str) -> IResult<&'a str, StacksSpecification, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, ElvesParseError>,
{
    let (rest, stack_def_lines) = many1(parse_stack_def_line_nl)(i)?;
    let (rest, stack_ids) = parse_stack_id_line_nl(&rest)?;
    success(StacksSpecification::new(stack_def_lines, stack_ids))(rest)
}

fn create_stacks(stacks_specs: StacksSpecification) -> Vec<Stack> {
    let lines = stacks_specs
        .slot_lines
        .iter()
        .rev()
        .collect::<Vec<&Vec<Option<Crate>>>>();
    let ids = stacks_specs.stack_ids;
    let stack_number = ids.len();
    let mut stacks = Vec::with_capacity(stack_number);
    for (i, id) in ids.iter().enumerate() {
        let mut stack = vec![];
        for l in &lines {
            if let Some(c) = &l[i] {
                stack.push(c.clone());
            }
        }
        stacks.push(Stack::new(id.clone(), stack));
    }
    stacks
}

fn code(stacks: &Vec<Stack>) -> String {
    stacks
        .iter()
        .map(|s| s.crates.last().map(|e| e.0).unwrap_or(' '))
        .collect::<String>()
}

pub fn top_crate_of_stacks(input: &PathBuf) -> Result<String> {
    let content = read_to_string(input)?;
    let (rest, stacks_specs) = parse_stacks_specifications::<()>(&content)?;
    let (rest, _) = empty_line::<()>(rest)?;
    let (_, moves) = parse_moves::<()>(rest)?;

    let stacks = create_stacks(stacks_specs);
    let stacks = rearrange_part_2(&stacks, &moves);
    let res = code(&stacks);

    Ok(res)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn rearrange_complex() {
        let stacks = vec![
            Stack::new(StackId('1'), vec![Crate('Z'), Crate('N')]),
            Stack::new(StackId('2'), vec![Crate('M'), Crate('C'), Crate('D')]),
            Stack::new(StackId('3'), vec![Crate('P')]),
        ];
        let moves = vec![
            Move::new(1, StackId('2'), StackId('1')),
            Move::new(3, StackId('1'), StackId('3')),
            Move::new(2, StackId('2'), StackId('1')),
            Move::new(1, StackId('1'), StackId('2')),
        ];

        let res = rearrange(&stacks, &moves);

        assert_eq!(
            res,
            vec![
                Stack::new(StackId('1'), vec![Crate('C')]),
                Stack::new(StackId('2'), vec![Crate('M')]),
                Stack::new(
                    StackId('3'),
                    vec![Crate('P'), Crate('D'), Crate('N'), Crate('Z')]
                )
            ]
        );
    }

    #[test]
    fn rearrange_simple() {
        let stacks = vec![
            Stack::new(StackId('1'), vec![]),
            Stack::new(StackId('2'), vec![Crate('A'), Crate('B')]),
        ];
        let moves = vec![Move::new(2, StackId('2'), StackId('1'))];

        let res = rearrange(&stacks, &moves);

        assert_eq!(
            res,
            vec![
                Stack::new(StackId('1'), vec![Crate('B'), Crate('A')]),
                Stack::new(StackId('2'), vec![]),
            ]
        );
    }

    #[test]
    fn create_stacks_valid() {
        let spec: StacksSpecification = StacksSpecification::new(
            vec![
                vec![
                    None,
                    Some(Crate('V')),
                    Some(Crate('G')),
                    None,
                    None,
                    None,
                    Some(Crate('H')),
                    None,
                    None,
                ],
                vec![
                    Some(Crate('Z')),
                    Some(Crate('H')),
                    Some(Crate('Z')),
                    None,
                    None,
                    Some(Crate('T')),
                    Some(Crate('S')),
                    None,
                    None,
                ],
                vec![
                    Some(Crate('P')),
                    Some(Crate('D')),
                    Some(Crate('F')),
                    None,
                    None,
                    Some(Crate('B')),
                    Some(Crate('V')),
                    Some(Crate('Q')),
                    None,
                ],
                vec![
                    Some(Crate('B')),
                    Some(Crate('M')),
                    Some(Crate('V')),
                    Some(Crate('N')),
                    None,
                    Some(Crate('F')),
                    Some(Crate('D')),
                    Some(Crate('N')),
                    None,
                ],
                vec![
                    Some(Crate('Q')),
                    Some(Crate('Q')),
                    Some(Crate('D')),
                    Some(Crate('F')),
                    None,
                    Some(Crate('Z')),
                    Some(Crate('Z')),
                    Some(Crate('P')),
                    Some(Crate('M')),
                ],
                vec![
                    Some(Crate('M')),
                    Some(Crate('Z')),
                    Some(Crate('R')),
                    Some(Crate('D')),
                    Some(Crate('Q')),
                    Some(Crate('V')),
                    Some(Crate('T')),
                    Some(Crate('F')),
                    Some(Crate('R')),
                ],
                vec![
                    Some(Crate('D')),
                    Some(Crate('L')),
                    Some(Crate('H')),
                    Some(Crate('G')),
                    Some(Crate('F')),
                    Some(Crate('Q')),
                    Some(Crate('M')),
                    Some(Crate('G')),
                    Some(Crate('W')),
                ],
                vec![
                    Some(Crate('N')),
                    Some(Crate('C')),
                    Some(Crate('Q')),
                    Some(Crate('H')),
                    Some(Crate('N')),
                    Some(Crate('D')),
                    Some(Crate('Q')),
                    Some(Crate('M')),
                    Some(Crate('B')),
                ],
            ],
            vec![
                StackId('1'),
                StackId('2'),
                StackId('3'),
                StackId('4'),
                StackId('5'),
                StackId('6'),
                StackId('7'),
                StackId('8'),
                StackId('9'),
            ],
        );

        let stacks = create_stacks(spec);

        assert_eq!(
            stacks,
            vec![
                Stack::new(
                    StackId('1'),
                    vec![
                        Crate('N'),
                        Crate('D'),
                        Crate('M'),
                        Crate('Q'),
                        Crate('B'),
                        Crate('P'),
                        Crate('Z')
                    ]
                ),
                Stack::new(
                    StackId('2'),
                    vec![
                        Crate('C'),
                        Crate('L'),
                        Crate('Z'),
                        Crate('Q'),
                        Crate('M'),
                        Crate('D'),
                        Crate('H'),
                        Crate('V')
                    ]
                ),
                Stack::new(
                    StackId('3'),
                    vec![
                        Crate('Q'),
                        Crate('H'),
                        Crate('R'),
                        Crate('D'),
                        Crate('V'),
                        Crate('F'),
                        Crate('Z'),
                        Crate('G')
                    ]
                ),
                Stack::new(
                    StackId('4'),
                    vec![Crate('H'), Crate('G'), Crate('D'), Crate('F'), Crate('N')]
                ),
                Stack::new(StackId('5'), vec![Crate('N'), Crate('F'), Crate('Q')]),
                Stack::new(
                    StackId('6'),
                    vec![
                        Crate('D'),
                        Crate('Q'),
                        Crate('V'),
                        Crate('Z'),
                        Crate('F'),
                        Crate('B'),
                        Crate('T')
                    ]
                ),
                Stack::new(
                    StackId('7'),
                    vec![
                        Crate('Q'),
                        Crate('M'),
                        Crate('T'),
                        Crate('Z'),
                        Crate('D'),
                        Crate('V'),
                        Crate('S'),
                        Crate('H')
                    ]
                ),
                Stack::new(
                    StackId('8'),
                    vec![
                        Crate('M'),
                        Crate('G'),
                        Crate('F'),
                        Crate('P'),
                        Crate('N'),
                        Crate('Q')
                    ]
                ),
                Stack::new(
                    StackId('9'),
                    vec![Crate('B'), Crate('W'), Crate('R'), Crate('M')]
                ),
            ]
        );
    }

    #[test]
    fn parse_stack_def_line_nl_valid() {
        let ins = String::from("[A] [B]     [C]\n");
        let fff = parse_stack_def_line_nl::<()>(&ins);
        assert_eq!(
            fff,
            Ok((
                "",
                vec![Some(Crate('A')), Some(Crate('B')), None, Some(Crate('C'))]
            ))
        )
    }

    #[test]
    fn parse_crate_with_spaces_leads_to_none() {
        let crate_s = parse_crate::<()>("   ");
        assert_eq!(crate_s, Ok(("", None)));
    }

    #[test]
    fn parse_crate_with_crate_id() {
        let crate_s = parse_crate::<()>("[C]");
        assert_eq!(crate_s, Ok(("", Some(Crate('C')))));
    }

    #[test]
    fn parse_stack_def_line_valid() {
        let stack_def_line = parse_stack_def_line::<()>("[Z] [A]     [X] [K]");
        assert_eq!(
            stack_def_line,
            Ok((
                "",
                vec![
                    Some(Crate('Z')),
                    Some(Crate('A')),
                    None,
                    Some(Crate('X')),
                    Some(Crate('K'))
                ]
            ))
        );
    }

    #[test]
    fn parse_stack_id_line_valid() {
        let stack_id_line = parse_stack_id_line::<()>(" 1   2   3   4   5   6   7   8   9 ");
        assert_eq!(
            stack_id_line,
            Ok((
                "",
                vec![
                    StackId('1'),
                    StackId('2'),
                    StackId('3'),
                    StackId('4'),
                    StackId('5'),
                    StackId('6'),
                    StackId('7'),
                    StackId('8'),
                    StackId('9'),
                ]
            ))
        );
    }

    #[test]
    fn parse_move_valid() {
        let move_s = parse_move::<()>("move 42 from 1 to 4");
        assert_eq!(move_s, Ok(("", Move::new(42, StackId('1'), StackId('4')))));
    }
}
