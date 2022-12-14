use anyhow::Result;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, digit1, line_ending, space1, u64, u8},
    combinator::{eof, map},
    error::ParseError,
    multi::separated_list1,
    sequence::{delimited, preceded, separated_pair, terminated},
    IResult,
};
use std::fs::read_to_string;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Op {
    Add(u64),
    Mult(u64),
    Square,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Item(u64);
#[derive(Clone, Copy, Debug, PartialEq)]
struct MonkeyId(u8);

#[derive(Debug, PartialEq)]
struct Monkey {
    id: MonkeyId,
    items: Vec<Item>,
    operation: Op,
    throw_decision: ThrowDecision,
}

impl Monkey {
    fn new(id: MonkeyId, items: &[Item], operation: Op, throw_decision: ThrowDecision) -> Self {
        Monkey {
            id,
            items: items.to_vec(),
            operation,
            throw_decision,
        }
    }

    fn inspect_item(&self, item: Item, md: u64) -> (MonkeyId, Item) {
        let new_worry_value = match self.operation {
            Op::Add(x) => (item.0 % md) + x,
            Op::Mult(x) => (item.0 % md) * x,
            Op::Square => (item.0 % md) * (item.0 % md),
        };

        (
            self.throw_decision.take_decision(new_worry_value),
            Item(new_worry_value),
        )
    }
}

#[derive(Debug, PartialEq)]
struct ThrowDecision {
    modulus: u64,
    if_true: MonkeyId,
    if_false: MonkeyId,
}

impl ThrowDecision {
    fn new(modulus: u64, if_true: MonkeyId, if_false: MonkeyId) -> Self {
        Self {
            modulus,
            if_true,
            if_false,
        }
    }

    fn take_decision(&self, item_value: u64) -> MonkeyId {
        if item_value % self.modulus == 0 {
            self.if_true
        } else {
            self.if_false
        }
    }
}

fn monkey_id<'a, E>(i: &'a str) -> IResult<&'a str, MonkeyId, E>
where
    E: ParseError<&'a str>,
{
    map(
        delimited(tag("Monkey "), u8, terminated(char(':'), line_ending)),
        MonkeyId,
    )(i)
}

fn items<'a, E>(i: &'a str) -> IResult<&'a str, Vec<Item>, E>
where
    E: ParseError<&'a str>,
{
    map(
        delimited(
            preceded(space1, tag("Starting items: ")),
            separated_list1(tag(", "), u64),
            line_ending,
        ),
        |v| v.into_iter().map(Item).collect::<Vec<_>>(),
    )(i)
}

fn operation<'a, E>(i: &'a str) -> IResult<&'a str, Op, E>
where
    E: ParseError<&'a str>,
{
    map(
        delimited(
            preceded(space1, tag("Operation: new = old ")),
            alt((
                separated_pair(alt((char('+'), char('*'))), char(' '), digit1),
                separated_pair(char('*'), char(' '), tag("old")),
            )),
            line_ending,
        ),
        |(op, val): (char, &str)| {
            if val == "old" {
                Op::Square
            } else {
                let val = val.parse::<u64>().unwrap(); // is unwrap dangerous here?
                match op {
                    '+' => Op::Add(val),
                    '*' => Op::Mult(val),
                    _ => Op::Add(0), // cannot happen, the parser will fail
                }
            }
        },
    )(i)
}

fn throw_decision<'a, E>(i: &'a str) -> IResult<&'a str, ThrowDecision, E>
where
    E: ParseError<&'a str>,
{
    let (rest, modulus) = delimited(
        preceded(space1, tag("Test: divisible by ")),
        u64,
        line_ending,
    )(i)?;
    let (rest, if_true) = delimited(
        preceded(space1, tag("If true: throw to monkey ")),
        u8,
        line_ending,
    )(rest)?;
    let (rest, if_false) = delimited(
        preceded(space1, tag("If false: throw to monkey ")),
        u8,
        alt((line_ending, eof)),
    )(rest)?;
    Ok((
        rest,
        ThrowDecision::new(modulus, MonkeyId(if_true), MonkeyId(if_false)),
    ))
}

fn monkey<'a, E>(i: &'a str) -> IResult<&'a str, Monkey, E>
where
    E: ParseError<&'a str>,
{
    let (rest, monkey_id) = monkey_id(i)?;
    let (rest, items) = items(rest)?;
    let (rest, operation) = operation(rest)?;
    let (rest, throw_decision) = throw_decision(rest)?;

    Ok((
        rest,
        Monkey::new(monkey_id, &items, operation, throw_decision),
    ))
}

fn monkeys<'a, E>(i: &'a str) -> IResult<&'a str, Vec<Monkey>, E>
where
    E: ParseError<&'a str>,
{
    separated_list1(line_ending, monkey)(i)
}

fn rounds(monkeys: &[Monkey], n: u16) -> Vec<u64> {
    // NOTE: we can probably do it better
    let mut items = monkeys.iter().map(|m| m.items.clone()).collect::<Vec<_>>();
    let md = monkeys.iter().map(|m| m.throw_decision.modulus).product();
    let mut nb_item_inspections = vec![0u64; monkeys.len()];

    for _ in 0..n {
        let mut round_items: Vec<Vec<Item>> = vec![vec![]; items.len()];

        for (mk, monkey_items) in items.iter().enumerate() {
            let mut items_to_inspect = monkey_items.clone();
            let part2 = &round_items[mk];
            items_to_inspect.extend(part2);
            round_items[mk] = vec![];
            nb_item_inspections[mk] += items_to_inspect.len() as u64;
            for item in items_to_inspect {
                let (throw_to, item) = monkeys[mk].inspect_item(item, md);
                round_items[throw_to.0 as usize].push(item);
            }
        }
        items = round_items;
    }
    nb_item_inspections
}

fn compute_score(monkeys: &[Monkey]) -> u64 {
    let mut inspections = rounds(monkeys, 10000);
    inspections.sort_by_key(|&e| std::cmp::Reverse(e));
    inspections.iter().take(2).product()
}

pub fn most_active_monkeys_score(input: &PathBuf) -> Result<u64> {
    let data = read_to_string(input)?;
    let (_, monkeys) = monkeys::<()>(&data)?;
    Ok(compute_score(&monkeys))
}

#[cfg(test)]
mod tests {

    use super::*;
    use spectral::prelude::*;

    #[test]
    fn parse_monkey() {
        let data = r#"Monkey 0:
  Starting items: 79, 98
  Operation: new = old * 19
  Test: divisible by 23
    If true: throw to monkey 2
    If false: throw to monkey 3"#;

        let monkey = monkey::<nom::error::Error<&str>>(&data);

        assert_that!(monkey).is_ok().is_equal_to(&(
            "",
            Monkey::new(
                MonkeyId(0),
                &[Item(79), Item(98)],
                Op::Mult(19),
                ThrowDecision::new(23, MonkeyId(2), MonkeyId(3)),
            ),
        ));
    }

    #[test]
    fn parse_monkey_square() {
        let data = r#"Monkey 0:
  Starting items: 79, 98
  Operation: new = old * old
  Test: divisible by 23
    If true: throw to monkey 2
    If false: throw to monkey 3"#;

        let monkey = monkey::<nom::error::Error<&str>>(&data);

        assert_that!(monkey).is_ok().is_equal_to(&(
            "",
            Monkey::new(
                MonkeyId(0),
                &[Item(79), Item(98)],
                Op::Square,
                ThrowDecision::new(23, MonkeyId(2), MonkeyId(3)),
            ),
        ));
    }

    #[test]
    fn parse_monkeys() {
        let data = r#"Monkey 0:
  Starting items: 79, 98
  Operation: new = old * 19
  Test: divisible by 23
    If true: throw to monkey 2
    If false: throw to monkey 3

Monkey 1:
  Starting items: 54, 65, 75, 74
  Operation: new = old + 6
  Test: divisible by 19
    If true: throw to monkey 2
    If false: throw to monkey 0"#;

        let monkeys = monkeys::<nom::error::Error<&str>>(&data);

        assert_that!(monkeys).is_ok().is_equal_to(&(
            "",
            vec![
                Monkey::new(
                    MonkeyId(0),
                    &[Item(79), Item(98)],
                    Op::Mult(19),
                    ThrowDecision::new(23, MonkeyId(2), MonkeyId(3)),
                ),
                Monkey::new(
                    MonkeyId(1),
                    &[Item(54), Item(65), Item(75), Item(74)],
                    Op::Add(6),
                    ThrowDecision::new(19, MonkeyId(2), MonkeyId(0)),
                ),
            ],
        ));
    }

    #[test]
    fn items_inspected_after_twenty_rounds() {
        let monkeys = vec![
            Monkey::new(
                MonkeyId(0),
                &[Item(79), Item(98)],
                Op::Mult(19),
                ThrowDecision::new(23, MonkeyId(2), MonkeyId(3)),
            ),
            Monkey::new(
                MonkeyId(1),
                &[Item(54), Item(65), Item(75), Item(74)],
                Op::Add(6),
                ThrowDecision::new(19, MonkeyId(2), MonkeyId(0)),
            ),
            Monkey::new(
                MonkeyId(2),
                &[Item(79), Item(60), Item(97)],
                Op::Square,
                ThrowDecision::new(13, MonkeyId(1), MonkeyId(3)),
            ),
            Monkey::new(
                MonkeyId(3),
                &[Item(74)],
                Op::Add(3),
                ThrowDecision::new(17, MonkeyId(0), MonkeyId(1)),
            ),
        ];

        let res = rounds(&monkeys, 20);

        assert_that!(res).is_equal_to(&vec![99, 97, 8, 103]);
    }

    #[test]
    fn test_compute_score() {
        let monkeys = vec![
            Monkey::new(
                MonkeyId(0),
                &[Item(79), Item(98)],
                Op::Mult(19),
                ThrowDecision::new(23, MonkeyId(2), MonkeyId(3)),
            ),
            Monkey::new(
                MonkeyId(1),
                &[Item(54), Item(65), Item(75), Item(74)],
                Op::Add(6),
                ThrowDecision::new(19, MonkeyId(2), MonkeyId(0)),
            ),
            Monkey::new(
                MonkeyId(2),
                &[Item(79), Item(60), Item(97)],
                Op::Square,
                ThrowDecision::new(13, MonkeyId(1), MonkeyId(3)),
            ),
            Monkey::new(
                MonkeyId(3),
                &[Item(74)],
                Op::Add(3),
                ThrowDecision::new(17, MonkeyId(0), MonkeyId(1)),
            ),
        ];

        let result = compute_score(&monkeys);

        assert_that!(result).is_equal_to(2713310158);
    }

    /*    #[test]
        fn twenty_rounds() {
            let mut monkeys = vec![
                Monkey::new(
                    MonkeyId(0),
                    &[Item(79), Item(98)],
                    Op::Mult(19),
                    ThrowDecision::new(Test::DivisibleBy(23), MonkeyId(2), MonkeyId(3)),
                ),
                Monkey::new(
                    MonkeyId(1),
                    &[Item(54), Item(65), Item(75), Item(74)],
                    Op::Add(6),
                    ThrowDecision::new(Test::DivisibleBy(19), MonkeyId(2), MonkeyId(0)),
                ),
                Monkey::new(
                    MonkeyId(2),
                    &[Item(79), Item(60), Item(97)],
                    Op::Square,
                    ThrowDecision::new(Test::DivisibleBy(13), MonkeyId(1), MonkeyId(3)),
                ),
                Monkey::new(
                    MonkeyId(3),
                    &[Item(74)],
                    Op::Add(3),
                    ThrowDecision::new(Test::DivisibleBy(17), MonkeyId(0), MonkeyId(1)),
                ),
            ];

            let state = rounds(&monkeys, 20);

            assert_that!(state[0]).is_equal_to(&vec![Item(10), Item(12), Item(14), Item(26), Item(34)]);
            assert_that!(state[1]).is_equal_to(&vec![
                Item(245),
                Item(93),
                Item(53),
                Item(199),
                Item(115),
            ]);
            assert_that!(state[2]).is_empty();
            assert_that!(state[3]).is_empty();
        }
    */
}
