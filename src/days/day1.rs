use crate::iterators::SplitIteratorExt;
use crate::parsing::{integer, line_separated};
use crate::Solution;
use itertools::Itertools;
use nom::combinator::{all_consuming, opt};

#[derive(Default)]
pub struct Day1 {}

impl Solution for Day1 {
    type Part1Result = u64;
    type Part2Result = Self::Part1Result;

    type Input = Vec<Vec<u64>>;

    fn parse<'a>(
        &mut self,
        input: &'a str,
    ) -> Result<Self::Input, nom::Err<nom::error::Error<&'a str>>> {
        let list = all_consuming(line_separated(opt(integer)))(input)?.1;

        Ok(list
            .into_iter()
            .split(None)
            .map(|i| i.map(|x| x.unwrap()).collect())
            .collect())
    }

    fn run_part_1(&mut self, data: &Self::Input) -> Self::Part1Result {
        data.iter().map(|xs| xs.iter().sum()).max().unwrap_or(0)
    }

    fn run_part_2(&mut self, data: &Self::Input) -> Self::Part2Result {
        data.iter()
            .map(|xs| xs.iter().sum::<u64>())
            .sorted()
            .rev()
            .take(3)
            .sum()
    }
}
