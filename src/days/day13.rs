use crate::{
    parsing::{integer, line_separated},
    Solution,
};
use nom::{
    branch::alt,
    character::complete::{char, newline},
    combinator::{all_consuming, map, opt},
    multi::separated_list0,
    sequence::{delimited, separated_pair, terminated},
    IResult,
};
use std::borrow::Borrow;
use std::cmp::{self, Ord, Ordering, PartialOrd};

#[derive(Default)]
pub struct Day13 {}

impl Solution for Day13 {
    type Part1Result = usize;
    type Part2Result = Self::Part1Result;

    type Input = Vec<ValuePair>;

    fn parse<'a>(
        &mut self,
        input: &'a str,
    ) -> Result<Self::Input, nom::Err<nom::error::Error<&'a str>>> {
        all_consuming(line_separated(terminated(value_pair, opt(newline))))(input).map(|x| x.1)
    }

    fn run_part_1(&mut self, data: &Self::Input) -> Self::Part1Result {
        data.iter()
            .enumerate()
            .filter(|&(_, p)| p.is_ordered())
            .map(|x| x.0 + 1)
            .sum()
    }

    fn run_part_2(&mut self, data: &Self::Input) -> Self::Part2Result {
        let divider1 = value("[[2]]").unwrap().1;
        let divider2 = value("[[6]]").unwrap().1;

        let mut data: Vec<&Value> = data.iter().flat_map(|vp| [&vp.0, &vp.1]).collect();
        data.push(&divider1);
        data.push(&divider2);
        data.sort();

        let index1 = data.binary_search(&&divider1).unwrap();
        let index2 = data.binary_search(&&divider2).unwrap();

        (index1 + 1) * (index2 + 1)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Value {
    List(Vec<Value>),
    Integer(u64),
}

#[derive(Debug)]
pub struct ValuePair(Value, Value);

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Value::Integer(x), Value::Integer(y)) => x.cmp(y),
            (Value::List(l), Value::Integer(_)) => cmp_list(l, &[other]),
            (Value::Integer(_), Value::List(l)) => cmp_list(&[self], l),
            (Value::List(l1), Value::List(l2)) => cmp_list(l1, l2),
        }
    }
}

fn cmp_list<T1: Borrow<Value>, T2: Borrow<Value>>(left: &[T1], right: &[T2]) -> std::cmp::Ordering {
    // Yoinked from https://doc.rust-lang.org/src/core/slice/cmp.rs.html#166
    let len = cmp::min(left.len(), right.len());
    let lhs = &left[..len];
    let rhs = &right[..len];

    for i in 0..len {
        match lhs[i].borrow().cmp(rhs[i].borrow()) {
            Ordering::Equal => (),
            non_eq => return non_eq,
        }
    }

    left.len().cmp(&right.len())
}

impl ValuePair {
    fn is_ordered(&self) -> bool {
        self.0.cmp(&self.1) == Ordering::Less
    }
}

fn value_pair(input: &str) -> IResult<&str, ValuePair> {
    map(separated_pair(value, newline, value), |(x, y)| {
        ValuePair(x, y)
    })(input)
}

fn value(input: &str) -> IResult<&str, Value> {
    alt((
        map(integer, Value::Integer),
        map(
            delimited(char('['), separated_list0(char(','), value), char(']')),
            Value::List,
        ),
    ))(input)
}
