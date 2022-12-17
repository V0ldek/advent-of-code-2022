use crate::{
    parsing::{integer, line_separated},
    Solution,
};
use itertools::Itertools;
use nom::{
    bytes::complete::tag,
    character::complete::char,
    combinator::{all_consuming, map},
    multi::separated_list1,
    sequence::separated_pair,
    IResult,
};
use std::cmp;
use std::collections::HashMap;

#[derive(Default)]
pub struct Day14 {}

const START: Coords = Coords { x: 500, y: 0 };

impl Solution for Day14 {
    type Part1Result = usize;
    type Part2Result = Self::Part1Result;

    type Input = Vec<Path>;

    fn parse<'a>(
        &mut self,
        input: &'a str,
    ) -> Result<Self::Input, nom::Err<nom::error::Error<&'a str>>> {
        all_consuming(line_separated(path))(input).map(|x| x.1)
    }

    fn run_part_1(&mut self, data: &Self::Input) -> Self::Part1Result {
        let lines: Vec<_> = data.iter().flat_map(|p| p.to_lines()).collect();
        let stone_map = StoneMap::new(&lines);
        let mut sand = HashMap::new();
        stone_map.check_if_rests(START, &mut sand);

        sand.values().into_iter().filter(|&&x| x).count()
    }

    fn run_part_2(&mut self, data: &Self::Input) -> Self::Part2Result {
        let mut lines: Vec<_> = data.iter().flat_map(|p| p.to_lines()).collect();
        lines.push(HorizontalLine {
            x_start: 0,
            x_end: usize::MAX,
            y: lines.iter().map(|l| l.y).max().unwrap() + 2,
        });
        let stone_map = StoneMap::new(&lines);
        let mut sand = HashMap::new();
        stone_map.check_if_rests(START, &mut sand);

        sand.values().into_iter().filter(|&&x| x).count()
    }
}

pub struct StoneMap {
    horizontal_lines: HashMap<usize, Vec<Segment>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Segment(usize, usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct HorizontalLine {
    x_start: usize,
    x_end: usize,
    y: usize,
}

impl StoneMap {
    fn new(lines: &[HorizontalLine]) -> Self {
        let horizontal_lines = lines
            .iter()
            .map(|&l| (l.y, Segment(l.x_start, l.x_end)))
            .sorted()
            .group_by(|x| x.0)
            .into_iter()
            .map(|(y, lines)| (y, concat_lines(lines.map(|x| x.1))))
            .collect();

        Self { horizontal_lines }
    }

    fn check_if_rests(&self, coords: Coords, memo: &mut HashMap<Coords, bool>) -> bool {
        if self.is_stone(coords) || self.max_y() < coords.y {
            return false;
        }

        if let Some(result) = memo.get(&coords) {
            return *result;
        }

        let result = [coords.down(), coords.down_left(), coords.down_right()]
            .into_iter()
            .all(|c| self.is_stone(c) || self.check_if_rests(c, memo));

        memo.insert(coords, result);

        result
    }

    fn is_stone(&self, coords: Coords) -> bool {
        let lines = self.horizontal_lines.get(&coords.y);

        lines.map_or(false, |lines| {
            let closest = ok_or_err(lines.binary_search(&Segment(coords.x, coords.x)));

            let first = lines.get(closest);
            let second = if closest > 0 {
                lines.get(closest - 1)
            } else {
                None
            };
            let third = lines.get(closest + 1);

            first.map_or(false, |l| l.contains(coords.x))
                || second.map_or(false, |l| l.contains(coords.x))
                || third.map_or(false, |l| l.contains(coords.x))
        })
    }

    fn max_y(&self) -> usize {
        *self.horizontal_lines.keys().max().unwrap()
    }
}

impl Segment {
    fn contains(&self, point: usize) -> bool {
        self.0 <= point && self.1 >= point
    }
}

fn ok_or_err<T>(res: Result<T, T>) -> T {
    match res {
        Ok(x) | Err(x) => x,
    }
}

pub struct Path {
    points: Vec<Coords>,
}

impl Path {
    fn to_lines(&self) -> Vec<HorizontalLine> {
        let mut lines = Vec::with_capacity(self.points.len());

        for window in self.points.windows(2) {
            let start = window[0];
            let end = window[1];

            if start.y == end.y {
                lines.push(HorizontalLine {
                    x_start: cmp::min(start.x, end.x),
                    x_end: cmp::max(start.x, end.x),
                    y: start.y,
                })
            } else {
                let a = cmp::min(start.y, end.y);
                let b = cmp::max(start.y, end.y);

                for y in a..b {
                    lines.push(HorizontalLine {
                        y,
                        x_start: start.x,
                        x_end: start.x,
                    });
                }
            }
        }

        lines
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Coords {
    x: usize,
    y: usize,
}

impl Coords {
    fn down(&self) -> Self {
        Self {
            x: self.x,
            y: self.y + 1,
        }
    }

    fn down_left(&self) -> Self {
        Self {
            x: self.x.wrapping_sub(1),
            y: self.y + 1,
        }
    }

    fn down_right(&self) -> Self {
        Self {
            x: self.x + 1,
            y: self.y + 1,
        }
    }
}

fn concat_lines<I: Iterator<Item = Segment>>(mut iter: I) -> Vec<Segment> {
    let mut current = iter.next().unwrap();
    let mut result = vec![];

    for item in iter {
        if current.1 + 1 >= item.0 {
            current = Segment(current.0, cmp::max(current.1, item.1));
        } else {
            result.push(current);
            current = item;
        }
    }

    result.push(current);
    result
}

fn path(input: &str) -> IResult<&str, Path> {
    map(separated_list1(tag(" -> "), coords), |points| Path {
        points,
    })(input)
}

fn coords(input: &str) -> IResult<&str, Coords> {
    map(separated_pair(integer, char(','), integer), |(x, y)| {
        Coords { x, y }
    })(input)
}
