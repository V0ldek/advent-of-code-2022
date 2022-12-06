use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{anychar, char as nom_char, multispace0, space0},
    combinator::{all_consuming, map},
    multi::separated_list1,
    sequence::{delimited, preceded, tuple},
    IResult,
};

use crate::{
    parsing::{integer, line_separated},
    Solution,
};

#[derive(Default)]
pub struct Day5 {}

impl Solution for Day5 {
    type Part1Result = String;
    type Part2Result = Self::Part1Result;

    type Input = (Storage, Vec<Command>);

    fn parse<'a>(
        &mut self,
        input: &'a str,
    ) -> Result<Self::Input, nom::Err<nom::error::Error<&'a str>>> {
        let (rem, mut rows) =
            line_separated(separated_list1(nom_char(' '), parse_empty_or_crate))(input)?;
        let (rem, mut stacks) =
            separated_list1(space0, map(integer::<usize>, |_| Stack::new()))(rem)?;
        let commands = all_consuming(preceded(multispace0, line_separated(parse_command)))(rem)
            .map(|x| x.1)?;

        rows.reverse();
        for row in rows {
            for (i, maybe_crate) in row.into_iter().enumerate() {
                if let Some(cr) = maybe_crate {
                    stacks[i].push(cr);
                }
            }
        }

        Ok((Storage { stacks }, commands))
    }

    fn run_part_1(&mut self, data: &Self::Input) -> Self::Part1Result {
        let mut storage = data.0.clone();

        for command in data.1.iter() {
            storage.execute(command, Order::Reverse);
        }

        storage.signature()
    }

    fn run_part_2(&mut self, data: &Self::Input) -> Self::Part2Result {
        let mut storage = data.0.clone();

        for command in data.1.iter() {
            storage.execute(command, Order::Retain);
        }

        storage.signature()
    }
}

#[derive(Debug, Clone)]
pub struct Storage {
    stacks: Vec<Stack>,
}

#[derive(Debug, Clone)]
struct Stack {
    crates: Vec<Crate>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Crate {
    id: char,
}

#[derive(Debug)]
pub struct Command {
    count: usize,
    from: usize,
    to: usize,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum Order {
    Retain,
    Reverse
}

impl Storage {
    fn execute(&mut self, cmd: &Command, order: Order) {
        let mut buffer = Vec::with_capacity(cmd.count);

        for _ in 0..cmd.count {
            let cr = self.stacks[cmd.from - 1].pop();
            buffer.push(cr);
        }

        if order == Order::Retain {
            buffer.reverse();
        }

        for cr in buffer {
            self.stacks[cmd.to - 1].push(cr);
        }
    }

    fn signature(&self) -> String {
        let mut sig = String::with_capacity(self.stacks.len());

        for stack in self.stacks.iter() {
            sig.push(stack.top().id);
        }

        sig
    }
}

impl Stack {
    fn new() -> Self {
        Self { crates: vec![] }
    }

    fn push(&mut self, cr: Crate) {
        self.crates.push(cr);
    }

    fn pop(&mut self) -> Crate {
        self.crates.pop().unwrap()
    }

    fn top(&self) -> Crate {
        *self.crates.last().unwrap()
    }
}

fn parse_command(input: &str) -> IResult<&str, Command> {
    map(
        tuple((
            preceded(tag("move "), integer),
            preceded(tag(" from "), integer),
            preceded(tag(" to "), integer),
        )),
        |(count, from, to)| Command { count, from, to },
    )(input)
}

fn parse_empty_or_crate(input: &str) -> IResult<&str, Option<Crate>> {
    alt((map(parse_crate, Some), map(tag("   "), |_| None)))(input)
}

fn parse_crate(input: &str) -> IResult<&str, Crate> {
    map(delimited(nom_char('['), anychar, nom_char(']')), |x| {
        Crate { id: x }
    })(input)
}
