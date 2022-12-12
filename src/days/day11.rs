use crate::{
    parsing::{integer, line_separated},
    Solution,
};
use itertools::Itertools;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, newline, space0},
    combinator::{all_consuming, map, opt},
    multi::separated_list0,
    sequence::{delimited, preceded, separated_pair, tuple},
    IResult,
};

#[derive(Default)]
pub struct Day11 {}

impl Solution for Day11 {
    type Part1Result = usize;
    type Part2Result = usize;

    type Input = Vec<MonkeyDescription>;

    fn parse<'a>(
        &mut self,
        input: &'a str,
    ) -> Result<Self::Input, nom::Err<nom::error::Error<&'a str>>> {
        all_consuming(line_separated(monkey))(input).map(|x| x.1)
    }

    fn run_part_1(&mut self, data: &Self::Input) -> Self::Part1Result {
        let mut monkey_business = MonkeyBusiness::<ItemWithDivision>::new(data);
        monkey_business.run_for(20);
        monkey_business.value()
    }

    fn run_part_2(&mut self, data: &Self::Input) -> Self::Part2Result {
        let mut monkey_business = MonkeyBusiness::<SimpleItem>::new(data);
        monkey_business.run_for(10_000);
        monkey_business.value()
    }
}

type MonkeyId = usize;
type Int = u64;

#[derive(Debug)]
struct MonkeyBusiness<I: Item> {
    monkeys: Vec<Monkey<I>>,
}

#[derive(Debug)]
struct Monkey<I: Item> {
    items: Vec<I>,
    operation: Operation,
    test: Test,
    inspections: usize,
}

trait Item {
    fn execute_operation(&mut self, op: &Operation);

    fn test(&self, modulo: Int) -> bool;
}

#[derive(Debug)]
struct ItemWithDivision {
    worry: Int,
}

#[derive(Debug)]
struct SimpleItem {
    worry: Int,
    modulus: Int,
}

impl MonkeyBusiness<ItemWithDivision> {
    fn new(descriptions: &[MonkeyDescription]) -> Self {
        let monkeys = descriptions
            .iter()
            .map(|m| {
                let items = m
                    .starting_items
                    .iter()
                    .map(|&i| ItemWithDivision { worry: i })
                    .collect();
                Monkey {
                    items,
                    operation: m.operation,
                    test: m.test,
                    inspections: 0,
                }
            })
            .collect();

        MonkeyBusiness { monkeys }
    }
}

impl MonkeyBusiness<SimpleItem> {
    fn new(descriptions: &[MonkeyDescription]) -> Self {
        let modulos: Vec<Int> = descriptions.iter().map(|m| m.test.modulus).collect();
        let gcd = modulos.iter().copied().reduce(gcd).unwrap();
        let lcm = modulos
            .iter()
            .copied()
            .reduce(|x, y| (x * y) / gcd)
            .unwrap();

        let monkeys = descriptions
            .iter()
            .map(|m| {
                let items = m
                    .starting_items
                    .iter()
                    .map(|&i| SimpleItem::new(i, lcm))
                    .collect();
                Monkey {
                    items,
                    operation: m.operation,
                    test: m.test,
                    inspections: 0,
                }
            })
            .collect();

        MonkeyBusiness { monkeys }
    }
}

impl<I: Item> MonkeyBusiness<I> {
    fn run_round(&mut self) {
        for monkey_id in 0..self.monkeys.len() {
            let monkey = &mut self.monkeys[monkey_id];
            monkey.inspections += monkey.items.len();
            let operation = monkey.operation;
            let test = monkey.test;
            let mut items = vec![];
            std::mem::swap(&mut items, &mut monkey.items);

            for mut item in items {
                item.execute_operation(&operation);

                if item.test(test.modulus) {
                    self.monkeys[test.if_true].items.push(item);
                } else {
                    self.monkeys[test.if_false].items.push(item);
                }
            }
        }
    }

    fn run_for(&mut self, rounds: usize) {
        for _ in 0..rounds {
            self.run_round();
        }
    }

    fn value(&self) -> usize {
        self.monkeys
            .iter()
            .map(|m| m.inspections)
            .sorted_by(|x, y| y.cmp(x))
            .take(2)
            .product()
    }
}

impl Item for ItemWithDivision {
    fn execute_operation(&mut self, operation: &Operation) {
        let (op1, op2) = operation.operands();
        let val1 = self.op_val(&op1);
        let val2 = self.op_val(&op2);

        match operation {
            Operation::Add(_, _) => self.worry = val1 + val2,
            Operation::Multiply(_, _) => self.worry = val1 * val2,
        }

        self.worry /= 3;
    }

    fn test(&self, test: Int) -> bool {
        self.worry % test == 0
    }
}

impl ItemWithDivision {
    fn op_val(&self, op: &Operand) -> Int {
        match op {
            Operand::Old => self.worry,
            Operand::Constant(c) => *c,
        }
    }
}

impl Item for SimpleItem {
    fn execute_operation(&mut self, operation: &Operation) {
        let (op1, op2) = operation.operands();
        let val1 = self.op_val(&op1);
        let val2 = self.op_val(&op2);

        match operation {
            Operation::Add(_, _) => self.worry = (val1 + val2) % self.modulus,
            Operation::Multiply(_, _) => self.worry = (val1 * val2) % self.modulus,
        }
    }

    fn test(&self, test: Int) -> bool {
        self.worry % test == 0
    }
}

impl SimpleItem {
    fn new(value: Int, modulus: Int) -> Self {
        let worry = value % modulus;

        Self { worry, modulus }
    }

    fn op_val(&self, op: &Operand) -> Int {
        match op {
            Operand::Old => self.worry,
            Operand::Constant(c) => *c % self.modulus,
        }
    }
}

pub fn gcd(a: Int, b: Int) -> Int {
    let (mut a, mut b) = if a > b { (a, b) } else { (b, a) };

    while b != 0 {
        std::mem::swap(&mut a, &mut b);
        b %= a;
    }

    a
}

#[derive(Debug)]
pub struct MonkeyDescription {
    starting_items: Vec<Int>,
    operation: Operation,
    test: Test,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Operation {
    Add(Operand, Operand),
    Multiply(Operand, Operand),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Operand {
    Old,
    Constant(Int),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Test {
    modulus: Int,
    if_true: MonkeyId,
    if_false: MonkeyId,
}

impl Operation {
    fn operands(&self) -> (Operand, Operand) {
        match self {
            Operation::Add(op1, op2) | Operation::Multiply(op1, op2) => (*op1, *op2),
        }
    }
}

fn monkey(input: &str) -> IResult<&str, MonkeyDescription> {
    map(
        tuple((
            delimited(
                tag("Monkey "),
                integer::<usize>,
                tuple((char(':'), newline)),
            ),
            delimited(space0, starting, newline),
            delimited(space0, operation, newline),
            delimited(space0, test, opt(newline)),
        )),
        |(_, starting_items, operation, test)| MonkeyDescription {
            starting_items,
            operation,
            test,
        },
    )(input)
}

fn starting(input: &str) -> IResult<&str, Vec<Int>> {
    preceded(
        space0,
        preceded(tag("Starting items: "), separated_list0(tag(", "), integer)),
    )(input)
}

fn operation(input: &str) -> IResult<&str, Operation> {
    preceded(
        space0,
        preceded(tag("Operation: new = "), alt((add, multiply))),
    )(input)
}

fn add(input: &str) -> IResult<&str, Operation> {
    map(separated_pair(operand, tag(" + "), operand), |(x, y)| {
        Operation::Add(x, y)
    })(input)
}

fn multiply(input: &str) -> IResult<&str, Operation> {
    map(separated_pair(operand, tag(" * "), operand), |(x, y)| {
        Operation::Multiply(x, y)
    })(input)
}

fn operand(input: &str) -> IResult<&str, Operand> {
    alt((
        map(tag("old"), |_| Operand::Old),
        map(integer, Operand::Constant),
    ))(input)
}

fn test(input: &str) -> IResult<&str, Test> {
    map(
        preceded(
            space0,
            tuple((
                preceded(tag("Test: divisible by "), integer),
                preceded(newline, condition("true")),
                preceded(newline, condition("false")),
            )),
        ),
        |(modulus, if_true, if_false)| Test {
            modulus,
            if_true,
            if_false,
        },
    )(input)
}

fn condition<'a>(if_str: &'static str) -> impl FnMut(&'a str) -> IResult<&'a str, MonkeyId> {
    preceded(
        tuple((space0, tag("If "), tag(if_str), tag(": throw to monkey "))),
        integer,
    )
}
