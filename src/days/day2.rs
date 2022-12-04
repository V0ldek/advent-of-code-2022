use crate::parsing::line_separated;
use crate::Solution;
use nom::{
    branch::alt,
    character::complete::{char as nom_char, space1},
    combinator::{all_consuming, map},
    sequence::separated_pair,
    IResult,
};

#[derive(Default)]
pub struct Day2 {}

impl Solution for Day2 {
    type Part1Result = u32;
    type Part2Result = Self::Part1Result;

    type Input = Vec<Round>;

    fn parse<'a>(
        &mut self,
        input: &'a str,
    ) -> Result<Self::Input, nom::Err<nom::error::Error<&'a str>>> {
        all_consuming(line_separated(round))(input).map(|x| x.1)
    }

    fn run_part_1(&mut self, data: &Self::Input) -> Self::Part1Result {
        data.iter().map(|x| x.score_part_1()).sum()
    }

    fn run_part_2(&mut self, data: &Self::Input) -> Self::Part2Result {
        data.iter().map(|x| x.score_part_2()).sum()
    }
}

pub struct Round {
    opposing_shape: Shape,
    my_shape_or_outcome: ShapeOrOutcome,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Shape {
    Rock,
    Paper,
    Scissors,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ShapeOrOutcome {
    RockOrLoss,
    PaperOrDraw,
    ScissorsOrWin,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Outcome {
    Win,
    Draw,
    Loss,
}

impl Round {
    fn score_part_1(&self) -> u32 {
        self.my_shape_or_outcome.shape().score() + self.outcome().score()
    }

    fn score_part_2(&self) -> u32 {
        let my_shape = match self.my_shape_or_outcome.outcome() {
            Outcome::Win => self.opposing_shape.beaten_by_shape(),
            Outcome::Draw => self.opposing_shape,
            Outcome::Loss => self.opposing_shape.beats_shape(),
        };

        my_shape.score() + self.my_shape_or_outcome.outcome().score()
    }

    fn outcome(&self) -> Outcome {
        if self
            .my_shape_or_outcome
            .shape()
            .does_beat(self.opposing_shape)
        {
            Outcome::Win
        } else if self
            .opposing_shape
            .does_beat(self.my_shape_or_outcome.shape())
        {
            Outcome::Loss
        } else {
            Outcome::Draw
        }
    }
}

impl ShapeOrOutcome {
    fn shape(self) -> Shape {
        match self {
            ShapeOrOutcome::RockOrLoss => Shape::Rock,
            ShapeOrOutcome::PaperOrDraw => Shape::Paper,
            ShapeOrOutcome::ScissorsOrWin => Shape::Scissors,
        }
    }

    fn outcome(self) -> Outcome {
        match self {
            ShapeOrOutcome::RockOrLoss => Outcome::Loss,
            ShapeOrOutcome::PaperOrDraw => Outcome::Draw,
            ShapeOrOutcome::ScissorsOrWin => Outcome::Win,
        }
    }
}

impl Shape {
    fn score(self) -> u32 {
        match self {
            Shape::Rock => 1,
            Shape::Paper => 2,
            Shape::Scissors => 3,
        }
    }

    fn does_beat(self, other: Shape) -> bool {
        self.beats_shape() == other
    }

    fn beats_shape(self) -> Shape {
        match self {
            Shape::Rock => Shape::Scissors,
            Shape::Paper => Shape::Rock,
            Shape::Scissors => Shape::Paper,
        }
    }

    fn beaten_by_shape(self) -> Shape {
        match self {
            Shape::Scissors => Shape::Rock,
            Shape::Rock => Shape::Paper,
            Shape::Paper => Shape::Scissors,
        }
    }
}

impl Outcome {
    fn score(self) -> u32 {
        match self {
            Outcome::Win => 6,
            Outcome::Draw => 3,
            Outcome::Loss => 0,
        }
    }
}

fn round(input: &str) -> IResult<&str, Round> {
    map(
        separated_pair(opponent_shape, space1, shape_or_outcome),
        |x| Round {
            opposing_shape: x.0,
            my_shape_or_outcome: x.1,
        },
    )(input)
}

fn opponent_shape(input: &str) -> IResult<&str, Shape> {
    alt((
        map(nom_char('A'), |_| Shape::Rock),
        map(nom_char('B'), |_| Shape::Paper),
        map(nom_char('C'), |_| Shape::Scissors),
    ))(input)
}

fn shape_or_outcome(input: &str) -> IResult<&str, ShapeOrOutcome> {
    alt((
        map(nom_char('X'), |_| ShapeOrOutcome::RockOrLoss),
        map(nom_char('Y'), |_| ShapeOrOutcome::PaperOrDraw),
        map(nom_char('Z'), |_| ShapeOrOutcome::ScissorsOrWin),
    ))(input)
}
