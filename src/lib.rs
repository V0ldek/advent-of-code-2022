#![feature(is_some_and)]
mod days;
pub(crate) mod iterators;
pub(crate) mod parsing;
use std::fmt::Display;

pub use days::*;

pub trait Solution: Default {
    type Input;
    type Part1Result: Display;
    type Part2Result: Display;

    fn parse<'a>(&mut self, input: &'a str) -> Result<Self::Input, nom::Err<nom::error::Error<&'a str>>>;

    fn run_part_1(&mut self, data: &Self::Input) -> Self::Part1Result;

    fn run_part_2(&mut self, data: &Self::Input) -> Self::Part2Result;
}
