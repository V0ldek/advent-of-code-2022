use crate::Solution;

#[derive(Default)]
pub struct Day6 {

}

impl Solution for Day6 {
    type Part1Result = String;
    type Part2Result = Self::Part1Result;

    type Input = ();

    fn parse<'a>(&mut self, _input: &'a str) -> Result<Self::Input, nom::Err<nom::error::Error<&'a str>>> {
        todo!()
    }

    fn run_part_1(&mut self, _data: &Self::Input) -> Self::Part1Result {
        todo!()
    }

    fn run_part_2(&mut self, _data: &Self::Input) -> Self::Part2Result {
        todo!()
    }
}