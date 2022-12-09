use std::{
    collections::HashSet,
    fmt::Display,
    ops::{Add, Sub},
};

use nom::{
    branch::alt,
    character::complete::{char, space0},
    combinator::{all_consuming, map},
    sequence::separated_pair,
    IResult,
};

use crate::{
    parsing::{integer, line_separated},
    Solution,
};

#[derive(Default)]
pub struct Day9 {}

impl Solution for Day9 {
    type Part1Result = usize;
    type Part2Result = Self::Part1Result;

    type Input = Vec<Move>;

    fn parse<'a>(
        &mut self,
        input: &'a str,
    ) -> Result<Self::Input, nom::Err<nom::error::Error<&'a str>>> {
        all_consuming(line_separated(parse_move))(input).map(|x| x.1)
    }

    fn run_part_1(&mut self, data: &Self::Input) -> Self::Part1Result {
        let rope = Rope::new(2);
        track_tail(rope, data)
    }

    fn run_part_2(&mut self, data: &Self::Input) -> Self::Part2Result {
        let rope = Rope::new(10);
        track_tail(rope, data)
    }
}

fn track_tail(mut rope: Rope, moves: &[Move]) -> usize {
    let mut visited = HashSet::new();

    for mov in moves {
        for _ in 0..mov.count {
            rope.mov(mov.direction);
            visited.insert(rope.tail());
        }
    }

    visited.len()
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Direction {
    Up,
    Left,
    Down,
    Right,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Move {
    direction: Direction,
    count: usize,
}

#[derive(Debug)]
struct Rope {
    parts: Vec<Coordinates>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct Coordinates {
    x: isize,
    y: isize,
}

impl Rope {
    fn new(size: usize) -> Self {
        Self {
            parts: std::iter::repeat(Coordinates { x: 0, y: 0 })
                .take(size)
                .collect(),
        }
    }

    fn mov(&mut self, direction: Direction) {
        self.parts[0] = self.parts[0].mov(direction);
        let mut prev = self.parts[0];

        for part in self.parts.iter_mut().skip(1) {
            Self::mov_part(part, prev);
            prev = *part;
        }
    }

    fn mov_part(part: &mut Coordinates, prev: Coordinates) {
        let delta = prev - *part;

        if delta.x.abs() > 1 || delta.y.abs() > 1 {
            match delta.x.cmp(&0) {
                std::cmp::Ordering::Less => *part = part.mov(Direction::Left),
                std::cmp::Ordering::Equal => (),
                std::cmp::Ordering::Greater => *part = part.mov(Direction::Right),
            }

            match delta.y.cmp(&0) {
                std::cmp::Ordering::Less => *part = part.mov(Direction::Down),
                std::cmp::Ordering::Equal => (),
                std::cmp::Ordering::Greater => *part = part.mov(Direction::Up),
            }
        }
    }

    fn tail(&self) -> Coordinates {
        *self.parts.last().unwrap()
    }
}

impl Coordinates {
    fn unit(direction: Direction) -> Self {
        match direction {
            Direction::Up => Coordinates { x: 0, y: 1 },
            Direction::Left => Coordinates { x: -1, y: 0 },
            Direction::Down => Coordinates { x: 0, y: -1 },
            Direction::Right => Coordinates { x: 1, y: 0 },
        }
    }

    fn mov(&self, direction: Direction) -> Self {
        *self + Self::unit(direction)
    }
}

impl Add<Coordinates> for Coordinates {
    type Output = Coordinates;

    fn add(self, rhs: Coordinates) -> Self::Output {
        Coordinates {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub<Coordinates> for Coordinates {
    type Output = Coordinates;

    fn sub(self, rhs: Coordinates) -> Self::Output {
        Coordinates {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Display for Rope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "H: {}", self.parts[0])?;

        for (i, part) in self.parts.iter().enumerate().skip(1) {
            write!(f, ", {i}: {part}")?;
        }

        Ok(())
    }
}

impl Display for Coordinates {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

fn parse_move(input: &str) -> IResult<&str, Move> {
    map(
        separated_pair(parse_direction, space0, integer),
        |(direction, count)| Move { direction, count },
    )(input)
}

fn parse_direction(input: &str) -> IResult<&str, Direction> {
    alt((
        map(char('U'), |_| Direction::Up),
        map(char('L'), |_| Direction::Left),
        map(char('D'), |_| Direction::Down),
        map(char('R'), |_| Direction::Right),
    ))(input)
}
