use anyhow::Result;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{i32, line_ending},
    combinator::{eof, map},
    error::ParseError,
    multi::many1,
    sequence::{delimited, terminated},
    IResult,
};
use std::fs::read_to_string;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq)]
enum Cycle {
    Noop,
    Loading,
    Execution(i32),
}

fn noop_instruction<'a, E>(i: &'a str) -> IResult<&'a str, Vec<Cycle>, E>
where
    E: ParseError<&'a str>,
{
    map(terminated(tag("noop"), alt((line_ending, eof))), |_| {
        vec![Cycle::Noop]
    })(i)
}

fn addx_instruction<'a, E>(i: &'a str) -> IResult<&'a str, Vec<Cycle>, E>
where
    E: ParseError<&'a str>,
{
    map(
        delimited(tag("addx "), i32, alt((line_ending, eof))),
        |val| vec![Cycle::Loading, Cycle::Execution(val)],
    )(i)
}

fn cycles<'a, E>(i: &'a str) -> IResult<&'a str, Vec<Cycle>, E>
where
    E: ParseError<&'a str>,
{
    map(many1(alt((noop_instruction, addx_instruction))), |v| {
        v.into_iter().flatten().collect::<Vec<_>>()
    })(i)
}

const FIRST_SIGNAL_IDX: usize = 20;
const SIGNAL_PERIOD: usize = 40;

fn compute_signal_strength(cycles: &[Cycle]) -> i32 {
    cycles
        .iter()
        .enumerate()
        .fold((0, 1), |(signal_strength, current_x), (i, v)| {
            let x = match v {
                Cycle::Noop | Cycle::Loading => current_x,
                Cycle::Execution(x) => current_x + x,
            };
            if i + 1 == FIRST_SIGNAL_IDX
                || (i > FIRST_SIGNAL_IDX && (i + 1 - FIRST_SIGNAL_IDX) % 40 == 0)
            {
                (signal_strength + (current_x * (i + 1) as i32), x)
            } else {
                (signal_strength, x)
            }
        })
        .0
}

fn display_pixel(index: usize, register_x: i32) {
    let sprite_index: i32 = (index % SIGNAL_PERIOD) as i32;
    if sprite_index == 0 {
        println!();
    }
    if register_x == sprite_index - 1
        || register_x == sprite_index
        || register_x == sprite_index + 1
    {
        print!("#");
    } else {
        print!(".");
    }
}

fn crt_display(cycles: &[Cycle]) {
    let mut current_x = 1;
    for (i, cycle) in cycles.iter().enumerate() {
        display_pixel(i, current_x);
        current_x = match cycle {
            Cycle::Noop | Cycle::Loading => current_x,
            Cycle::Execution(x) => current_x + x,
        };
    }
}

pub fn sum_of_signal_strengths(input: &PathBuf) -> Result<i32> {
    let data = read_to_string(input)?;
    let (_, cycles) = cycles::<()>(&data)?;
    crt_display(&cycles);

    Ok(compute_signal_strength(&cycles))
}

#[cfg(test)]
mod tests {

    use super::*;
    use spectral::prelude::*;

    #[test]
    fn parse_noop() {
        let noop = noop_instruction::<()>("noop");

        assert_that!(noop)
            .is_ok()
            .is_equal_to(&("", vec![Cycle::Noop]));
    }

    #[test]
    fn parse_addx_positive() {
        let addx = addx_instruction::<()>("addx 12");

        assert_that!(addx)
            .is_ok()
            .is_equal_to(&("", vec![Cycle::Loading, Cycle::Execution(12)]));
    }

    #[test]
    fn parse_addx_negative() {
        let addx = addx_instruction::<()>("addx -42");

        assert_that!(addx)
            .is_ok()
            .is_equal_to(&("", vec![Cycle::Loading, Cycle::Execution(-42)]));
    }

    #[test]
    fn parse_cycles() {
        let data = "noop\naddx 3\nnoop\nnoop\naddx -3";
        let cycles = cycles::<()>(&data);

        assert_that!(cycles).is_ok().is_equal_to(&(
            "",
            vec![
                Cycle::Noop,
                Cycle::Loading,
                Cycle::Execution(3),
                Cycle::Noop,
                Cycle::Noop,
                Cycle::Loading,
                Cycle::Execution(-3),
            ],
        ));
    }

    #[test]
    fn test_compute_signal_strength() -> Result<()> {
        let data = r#"addx 15
addx -11
addx 6
addx -3
addx 5
addx -1
addx -8
addx 13
addx 4
noop
addx -1
addx 5
addx -1
addx 5
addx -1
addx 5
addx -1
addx 5
addx -1
addx -35
addx 1
addx 24
addx -19
addx 1
addx 16
addx -11
noop
noop
addx 21
addx -15
noop
noop
addx -3
addx 9
addx 1
addx -3
addx 8
addx 1
addx 5
noop
noop
noop
noop
noop
addx -36
noop
addx 1
addx 7
noop
noop
noop
addx 2
addx 6
noop
noop
noop
noop
noop
addx 1
noop
noop
addx 7
addx 1
noop
addx -13
addx 13
addx 7
noop
addx 1
addx -33
noop
noop
noop
addx 2
noop
noop
noop
addx 8
noop
addx -1
addx 2
addx 1
noop
addx 17
addx -9
addx 1
addx 1
addx -3
addx 11
noop
noop
addx 1
noop
addx 1
noop
noop
addx -13
addx -19
addx 1
addx 3
addx 26
addx -30
addx 12
addx -1
addx 3
addx 1
noop
noop
noop
addx -9
addx 18
addx 1
addx 2
noop
noop
addx 9
noop
noop
noop
addx -1
addx 2
addx -37
addx 1
addx 3
noop
addx 15
addx -21
addx 22
addx -6
addx 1
noop
addx 2
addx 1
noop
addx -10
noop
noop
addx 20
addx 1
addx 2
addx 2
addx -6
addx -11
noop
noop
noop"#;

        let (_, cycles) = cycles::<()>(&data)?;
        let res = compute_signal_strength(&cycles);

        assert_that!(res).is_equal_to(13140i32);
        Ok(())
    }
}
