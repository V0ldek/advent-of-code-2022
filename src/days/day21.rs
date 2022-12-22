use std::{collections::HashMap, fmt::Display};

use crate::{
    parsing::{integer, line_separated},
    Solution,
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::alpha0,
    combinator::{all_consuming, map},
    sequence::{separated_pair, tuple},
    IResult,
};

#[derive(Default)]
pub struct Day21 {}

impl Solution for Day21 {
    type Part1Result = NodeRef;
    type Part2Result = Int;

    type Input = Vec<Monkey>;

    fn parse<'a>(
        &mut self,
        input: &'a str,
    ) -> Result<Self::Input, nom::Err<nom::error::Error<&'a str>>> {
        all_consuming(line_separated(monkey))(input).map(|x| x.1)
    }

    fn run_part_1(&mut self, data: &Self::Input) -> Self::Part1Result {
        let monkeys = data.iter().map(|m| (&m.id as &str, &m.job)).collect();
        let mut cache = HashMap::new();

        build_tree("root", &monkeys, &mut cache)
    }

    fn run_part_2(&mut self, data: &Self::Input) -> Self::Part2Result {
        let mut monkeys: HashMap<&str, &Job> =
            data.iter().map(|m| (&m.id as &str, &m.job)).collect();
        let root_job = monkeys.get_mut("root").expect("no root job");
        let fixed_root_job = root_fixup(root_job);
        *root_job = &fixed_root_job;

        let mut cache = HashMap::new();
        cache.insert("humn", NodeRef(Box::new(Node::Variable)));

        let tree = build_tree("root", &monkeys, &mut cache);
        println!("initial: {tree}");
        let unknown = equate_with_zero(tree);

        solve_equation(unknown, 0)
    }
}

fn build_tree<'a>(
    id: &'a str,
    jobs: &'a HashMap<&str, &Job>,
    cache: &mut HashMap<&'a str, NodeRef>,
) -> NodeRef {
    if let Some(val) = cache.get(id) {
        val.clone()
    } else {
        let node = match jobs[id] {
            Job::Constant(x) => Node::Value(*x),
            Job::Operation(m1, op, m2) => {
                let left = build_tree(m1, jobs, cache);
                let right = build_tree(m2, jobs, cache);

                match (left.try_value(), right.try_value()) {
                    (Some(v1), Some(v2)) if *op != Operation::Eq => {
                        let value = match op {
                            Operation::Add => v1 + v2,
                            Operation::Sub => v1 - v2,
                            Operation::Mul => v1 * v2,
                            Operation::Div => v1 / v2,
                            _ => unreachable!(),
                        };
                        Node::Value(value)
                    }
                    _ => Node::Operation(left, *op, right),
                }
            }
        };

        let result = NodeRef(Box::new(node));
        cache.insert(id, result.clone());
        result
    }
}

fn root_fixup(job: &Job) -> Job {
    match job {
        Job::Constant(v) => Job::Constant(*v),
        Job::Operation(l, _, r) => Job::Operation(l.clone(), Operation::Eq, r.clone()),
    }
}

fn equate_with_zero(node: NodeRef) -> NodeRef {
    match *node.0 {
        Node::Operation(l, Operation::Eq, r) => {
            let new_equation = Node::Operation(l, Operation::Sub, r);
            NodeRef(Box::new(new_equation))
        }
        _ => panic!("not an equation"),
    }
}

fn solve_equation(equation: NodeRef, solution: Int) -> Int {
    match *equation.0 {
        Node::Value(_) => panic!("no variable"),
        Node::Variable => solution,
        Node::Operation(l, op, r) => {
            let (rem, val, reverse) = if let Some(val) = l.try_value() {
                (r, val, true)
            } else if let Some(val) = r.try_value() {
                (l, val, false)
            } else {
                panic!("not a simple equation")
            };

            let solution = match op {
                Operation::Add => solution - val,
                Operation::Sub if reverse => -(solution - val),
                Operation::Sub => solution + val,
                Operation::Mul => solution / val,
                Operation::Div if reverse => val / solution,
                Operation::Div => solution * val,
                Operation::Eq => panic!("eq inside equation body"),
            };

            solve_equation(rem, solution)
        }
    }
}

type Int = i64;
type MonkeyId = String;

#[derive(Debug, Clone)]
pub struct NodeRef(Box<Node>);

#[derive(Debug, Clone)]
enum Node {
    Value(Int),
    Variable,
    Operation(NodeRef, Operation, NodeRef),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Monkey {
    id: MonkeyId,
    job: Job,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Job {
    Constant(Int),
    Operation(MonkeyId, Operation, MonkeyId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Operation {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
}

impl NodeRef {
    fn try_value(&self) -> Option<Int> {
        match self.0.as_ref() {
            Node::Value(val) => Some(*val),
            Node::Variable => None,
            Node::Operation(_, _, _) => None,
        }
    }
}

impl Display for NodeRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0.as_ref() {
            Node::Value(v) => write!(f, "{v}"),
            Node::Variable => write!(f, "x"),
            Node::Operation(l, op, r) => write!(f, "({l} {op} {r})"),
        }
    }
}

impl Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operation::Add => write!(f, "+"),
            Operation::Sub => write!(f, "-"),
            Operation::Mul => write!(f, "*"),
            Operation::Div => write!(f, "/"),
            Operation::Eq => write!(f, "="),
        }
    }
}

fn monkey(input: &str) -> IResult<&str, Monkey> {
    map(separated_pair(alpha0, tag(": "), job), |(id, job)| Monkey {
        id: id.to_owned(),
        job,
    })(input)
}

fn job(input: &str) -> IResult<&str, Job> {
    alt((
        map(integer, Job::Constant),
        map(tuple((alpha0, operation, alpha0)), |(id1, op, id2)| {
            Job::Operation(id1.to_owned(), op, id2.to_owned())
        }),
    ))(input)
}

fn operation(input: &str) -> IResult<&str, Operation> {
    alt((
        map(tag(" + "), |_| Operation::Add),
        map(tag(" - "), |_| Operation::Sub),
        map(tag(" * "), |_| Operation::Mul),
        map(tag(" / "), |_| Operation::Div),
    ))(input)
}
