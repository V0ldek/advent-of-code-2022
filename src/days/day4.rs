use crate::{
    parsing::{integer, line_separated},
    Solution,
};
use nom::{
    character::complete::char as nom_char,
    combinator::{all_consuming, map},
    sequence::separated_pair,
    IResult,
};

#[derive(Default)]
pub struct Day4 {}

impl Solution for Day4 {
    type Part1Result = usize;
    type Part2Result = Self::Part1Result;

    type Input = Vec<ElfPair>;

    fn parse<'a>(
        &mut self,
        input: &'a str,
    ) -> Result<Self::Input, nom::Err<nom::error::Error<&'a str>>> {
        all_consuming(line_separated(elf_pair))(input).map(|x| x.1)
    }

    fn run_part_1(&mut self, data: &Self::Input) -> Self::Part1Result {
        data.iter().filter(|x| x.has_containing_ranges()).count()
    }

    fn run_part_2(&mut self, data: &Self::Input) -> Self::Part2Result {
        data.iter().filter(|x| x.has_overlapping_ranges()).count()
    }
}

type Section = u64;

#[derive(Debug)]
struct Range {
    from: Section,
    to: Section,
}

#[derive(Debug)]
pub struct ElfPair {
    elf_1: Range,
    elf_2: Range,
}

impl ElfPair {
    fn has_containing_ranges(&self) -> bool {
        self.elf_1.contains_range(&self.elf_2) || self.elf_2.contains_range(&self.elf_1)
    }

    fn has_overlapping_ranges(&self) -> bool {
        self.elf_1.overlaps(&self.elf_2) || self.elf_2.overlaps(&self.elf_1)
    }
}

impl Range {
    fn contains_range(&self, other: &Range) -> bool {
        self.contains_section(other.from) && self.contains_section(other.to)
    }

    fn contains_section(&self, section: Section) -> bool {
        self.from <= section && self.to >= section
    }

    fn overlaps(&self, other: &Range) -> bool {
        self.contains_section(other.from) || self.contains_section(other.to)
    }
}

fn elf_pair(input: &str) -> IResult<&str, ElfPair> {
    map(separated_pair(range, nom_char(','), range), |(x, y)| {
        ElfPair { elf_1: x, elf_2: y }
    })(input)
}

fn range(input: &str) -> IResult<&str, Range> {
    map(separated_pair(integer, nom_char('-'), integer), |(x, y)| {
        Range { from: x, to: y }
    })(input)
}
