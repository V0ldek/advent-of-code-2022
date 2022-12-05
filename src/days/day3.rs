use std::collections::HashSet;

use crate::parsing::line_separated;
use crate::Solution;
use nom::{
    character::complete::satisfy,
    combinator::{all_consuming, map},
    multi::many1,
    IResult,
};

#[derive(Default)]
pub struct Day3 {}

impl Solution for Day3 {
    type Part1Result = u32;
    type Part2Result = Self::Part1Result;

    type Input = Vec<Rucksack>;

    fn parse<'a>(
        &mut self,
        input: &'a str,
    ) -> Result<Self::Input, nom::Err<nom::error::Error<&'a str>>> {
        all_consuming(line_separated(rucksack))(input).map(|x| x.1)
    }

    fn run_part_1(&mut self, data: &Self::Input) -> Self::Part1Result {
        data.iter().map(|r| r.common_item().priority()).sum()
    }

    fn run_part_2(&mut self, data: &Self::Input) -> Self::Part2Result {
        data.chunks(3)
            .map(|xs| common_item_in_rucksacks(xs).priority())
            .sum()
    }
}

#[derive(Debug)]
pub struct Rucksack {
    compartment_1: Compartment,
    compartment_2: Compartment,
}

#[derive(Debug)]
struct Compartment {
    items: HashSet<Item>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
struct Item {
    char: char,
}

impl Rucksack {
    fn new(items: &[Item]) -> Self {
        let compartment_1_items = &items[..items.len() / 2];
        let compartment_2_items = &items[items.len() / 2..];

        Self {
            compartment_1: Compartment::new(compartment_1_items),
            compartment_2: Compartment::new(compartment_2_items),
        }
    }

    fn common_item(&self) -> Item {
        let intersection = &self.compartment_1.items & &self.compartment_2.items;

        if intersection.len() != 1 {
            panic!("intersection of compartments is not of size 1: {self:?}");
        }

        intersection.into_iter().next().unwrap()
    }

    fn items(&self) -> HashSet<Item> {
        &self.compartment_1.items | &self.compartment_2.items
    }
}

impl Compartment {
    fn new(items: &[Item]) -> Self {
        Compartment {
            items: HashSet::from_iter(items.iter().copied()),
        }
    }
}

impl Item {
    fn priority(self) -> u32 {
        if self.char.is_lowercase() {
            (self.char as u32) - ('a' as u32) + 1
        } else {
            (self.char as u32) - ('A' as u32) + 27
        }
    }
}

fn rucksack(input: &str) -> IResult<&str, Rucksack> {
    map(many1(item), |x| Rucksack::new(&x))(input)
}

fn item(input: &str) -> IResult<&str, Item> {
    map(satisfy(|x| x.is_ascii_alphabetic()), |x| Item { char: x })(input)
}

fn common_item_in_rucksacks(sacks: &[Rucksack]) -> Item {
    assert!(!sacks.is_empty());

    let intersection = sacks
        .iter()
        .map(|r| r.items())
        .reduce(|a, x| &a & &x)
        .unwrap();

    if intersection.len() != 1 {
        panic!(
            "intersection of sacks is not of size 1 but {}: {sacks:?}",
            intersection.len()
        );
    }

    intersection.into_iter().next().unwrap()
}
