use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, line_ending, one_of, space1, u64, u8},
    combinator::{eof, map, value},
    error::ParseError,
    multi::separated_list1,
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};
use nom_locate::LocatedSpan;

pub type Span<'a> = LocatedSpan<&'a str>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Operation {
    Add(Term, Term),
    Mul(Term, Term),
}

impl Operation {
    pub fn eval(self, old: u64) -> u64 {
        match self {
            Operation::Add(l, r) => l.eval(old) + r.eval(old),
            Operation::Mul(l, r) => l.eval(old) * r.eval(old),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Term {
    Old,
    Constant(u64),
}

impl Term {
    pub fn eval(self, old: u64) -> u64 {
        match self {
            Term::Old => old,
            Term::Constant(c) => c,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Item(u64);
#[derive(Clone, Copy, Debug, PartialEq)]
struct MonkeyId(u8);

#[derive(Debug, PartialEq)]
pub struct Monkey {
    id: MonkeyId,
    items: Vec<Item>,
    operation: Operation,
    throw_decision: ThrowDecision,
}

impl Monkey {
    fn new(
        id: MonkeyId,
        items: &[Item],
        operation: Operation,
        throw_decision: ThrowDecision,
    ) -> Self {
        Monkey {
            id,
            items: items.to_vec(),
            operation,
            throw_decision,
        }
    }

    fn inspect_item(&self, item: Item, md: u64) -> (MonkeyId, Item) {
        let new_worry_value = self.operation.eval(item.0 % md);

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

fn monkey_id<'a, E>(i: Span<'a>) -> IResult<Span<'a>, MonkeyId, E>
where
    E: ParseError<Span<'a>>,
{
    map(
        delimited(tag("Monkey "), u8, terminated(char(':'), line_ending)),
        MonkeyId,
    )(i)
}

fn items<'a, E>(i: Span<'a>) -> IResult<Span<'a>, Vec<Item>, E>
where
    E: ParseError<Span<'a>>,
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

fn term<'a, E>(i: Span<'a>) -> IResult<Span<'a>, Term, E>
where
    E: ParseError<Span<'a>>,
{
    alt((value(Term::Old, tag("old")), map(u64, Term::Constant)))(i)
}

fn operation<'a, E>(i: Span<'a>) -> IResult<Span<'a>, Operation, E>
where
    E: ParseError<Span<'a>>,
{
    let (i, (l, op, r)) = delimited(
        preceded(space1, tag("Operation: new = ")),
        tuple((term, preceded(space1, one_of("*+")), preceded(space1, term))),
        line_ending,
    )(i)?;
    let op = match op {
        '*' => Operation::Mul(l, r),
        '+' => Operation::Add(l, r),
        _ => unreachable!(),
    };
    Ok((i, op))
}

fn throw_decision<'a, E>(i: Span<'a>) -> IResult<Span<'a>, ThrowDecision, E>
where
    E: ParseError<Span<'a>>,
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

fn monkey<'a, E>(i: Span<'a>) -> IResult<Span<'a>, Monkey, E>
where
    E: ParseError<Span<'a>>,
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

pub fn monkeys<'a, E>(i: Span<'a>) -> IResult<Span<'a>, Vec<Monkey>, E>
where
    E: ParseError<Span<'a>>,
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

pub fn compute_score(monkeys: &[Monkey]) -> u64 {
    let mut inspections = rounds(monkeys, 10000);
    inspections.sort_by_key(|&e| std::cmp::Reverse(e));
    inspections.iter().take(2).product()
}

#[cfg(test)]
mod tests {

    use super::*;
    use spectral::prelude::*;

    #[test]
    fn parse_monkey() {
        let data = Span::new(
            r#"Monkey 0:
  Starting items: 79, 98
  Operation: new = old * 19
  Test: divisible by 23
    If true: throw to monkey 2
    If false: throw to monkey 3"#,
        );

        let monkey = monkey::<nom::error::Error<Span>>(data);

        assert_that!(monkey).is_ok();
        let monkey = monkey.unwrap().1;
        assert_that!(monkey).is_equal_to(&Monkey::new(
            MonkeyId(0),
            &[Item(79), Item(98)],
            Operation::Mul(Term::Old, Term::Constant(19)),
            ThrowDecision::new(23, MonkeyId(2), MonkeyId(3)),
        ));
    }

    #[test]
    fn parse_monkey_square() {
        let data = Span::new(
            r#"Monkey 0:
  Starting items: 79, 98
  Operation: new = old * old
  Test: divisible by 23
    If true: throw to monkey 2
    If false: throw to monkey 3"#,
        );

        let monkey = monkey::<nom::error::Error<Span>>(data);

        assert_that!(monkey).is_ok();
        let monkey = monkey.unwrap().1;
        assert_that!(monkey).is_equal_to(&Monkey::new(
            MonkeyId(0),
            &[Item(79), Item(98)],
            Operation::Mul(Term::Old, Term::Old),
            ThrowDecision::new(23, MonkeyId(2), MonkeyId(3)),
        ));
    }

    #[test]
    fn parse_monkeys() {
        let data = Span::new(
            r#"Monkey 0:
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
    If false: throw to monkey 0"#,
        );

        let monkeys = monkeys::<nom::error::Error<Span>>(data);

        assert_that!(monkeys).is_ok();
        let monkeys = monkeys.unwrap().1;
        assert_that!(monkeys).is_equal_to(&vec![
            Monkey::new(
                MonkeyId(0),
                &[Item(79), Item(98)],
                Operation::Mul(Term::Old, Term::Constant(19)),
                ThrowDecision::new(23, MonkeyId(2), MonkeyId(3)),
            ),
            Monkey::new(
                MonkeyId(1),
                &[Item(54), Item(65), Item(75), Item(74)],
                Operation::Add(Term::Old, Term::Constant(6)),
                ThrowDecision::new(19, MonkeyId(2), MonkeyId(0)),
            ),
        ]);
    }

    #[test]
    fn items_inspected_after_twenty_rounds() {
        let monkeys = vec![
            Monkey::new(
                MonkeyId(0),
                &[Item(79), Item(98)],
                Operation::Mul(Term::Old, Term::Constant(19)),
                ThrowDecision::new(23, MonkeyId(2), MonkeyId(3)),
            ),
            Monkey::new(
                MonkeyId(1),
                &[Item(54), Item(65), Item(75), Item(74)],
                Operation::Add(Term::Old, Term::Constant(6)),
                ThrowDecision::new(19, MonkeyId(2), MonkeyId(0)),
            ),
            Monkey::new(
                MonkeyId(2),
                &[Item(79), Item(60), Item(97)],
                Operation::Mul(Term::Old, Term::Old),
                ThrowDecision::new(13, MonkeyId(1), MonkeyId(3)),
            ),
            Monkey::new(
                MonkeyId(3),
                &[Item(74)],
                Operation::Add(Term::Old, Term::Constant(3)),
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
                Operation::Mul(Term::Old, Term::Constant(19)),
                ThrowDecision::new(23, MonkeyId(2), MonkeyId(3)),
            ),
            Monkey::new(
                MonkeyId(1),
                &[Item(54), Item(65), Item(75), Item(74)],
                Operation::Add(Term::Old, Term::Constant(6)),
                ThrowDecision::new(19, MonkeyId(2), MonkeyId(0)),
            ),
            Monkey::new(
                MonkeyId(2),
                &[Item(79), Item(60), Item(97)],
                Operation::Mul(Term::Old, Term::Old),
                ThrowDecision::new(13, MonkeyId(1), MonkeyId(3)),
            ),
            Monkey::new(
                MonkeyId(3),
                &[Item(74)],
                Operation::Add(Term::Old, Term::Constant(3)),
                ThrowDecision::new(17, MonkeyId(0), MonkeyId(1)),
            ),
        ];

        let result = compute_score(&monkeys);

        assert_that!(result).is_equal_to(2713310158);
    }
}
