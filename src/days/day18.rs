use crate::{
    parsing::{integer, line_separated},
    Solution,
};
use itertools::Itertools;
use nom::{
    character::complete::char,
    combinator::{all_consuming, map},
    sequence::separated_pair,
    IResult,
};
use std::{cmp, collections::HashMap};

#[derive(Default)]
pub struct Day18 {}

impl Solution for Day18 {
    type Part1Result = usize;
    type Part2Result = Self::Part1Result;

    type Input = Vec<Coords>;

    fn parse<'a>(
        &mut self,
        input: &'a str,
    ) -> Result<Self::Input, nom::Err<nom::error::Error<&'a str>>> {
        all_consuming(line_separated(cube))(input).map(|x| x.1)
    }

    fn run_part_1(&mut self, data: &Self::Input) -> Self::Part1Result {
        let group_by_xy = group_dimension(data, |c| ((c.x, c.y), c.z));
        let group_by_xz = group_dimension(data, |c| ((c.x, c.z), c.y));
        let group_by_yz = group_dimension(data, |c| ((c.y, c.z), c.x));

        let result_xy: usize = group_by_xy
            .into_iter()
            .map(|line| concat_lines(line.into_iter()).len() * 2)
            .sum();
        let result_xz: usize = group_by_xz
            .into_iter()
            .map(|line| concat_lines(line.into_iter()).len() * 2)
            .sum();
        let result_yz: usize = group_by_yz
            .into_iter()
            .map(|line| concat_lines(line.into_iter()).len() * 2)
            .sum();

        result_xy + result_xz + result_yz
    }

    fn run_part_2(&mut self, data: &Self::Input) -> Self::Part2Result {
        let max_coords = data
            .iter()
            .fold(Coords::default(), |a, c| Coords {
                x: cmp::max(a.x, c.x),
                y: cmp::max(a.y, c.y),
                z: cmp::max(a.z, c.z),
            })
            .offset(1, 1, 1);
        let mut map: HashMap<_, _> = data.iter().map(|&c| (c, CubeType::Lava)).collect();

        mark_exterior(max_coords, &mut map, &max_coords);

        data.iter()
            .map(|c| {
                c.neighbors()
                    .into_iter()
                    .filter(|n| {
                        n.x < 0
                            || n.y < 0
                            || n.z < 0
                            || map.get(n).is_some_and(|&x| x == CubeType::Exterior)
                    })
                    .count()
            })
            .sum()
    }
}

fn group_dimension<F>(cubes: &[Coords], proj: F) -> Vec<Vec<Segment>>
where
    F: Fn(&Coords) -> ((isize, isize), isize),
{
    cubes
        .iter()
        .map(proj)
        .sorted()
        .group_by(|x| x.0)
        .into_iter()
        .map(|(_, g)| {
            g.map(|c| Segment {
                start: c.1,
                end: c.1,
            })
            .collect()
        })
        .collect()
}

fn mark_exterior(coords: Coords, map: &mut HashMap<Coords, CubeType>, limit: &Coords) {
    map.insert(coords, CubeType::Exterior);
    let mut stack = vec![coords];

    while let Some(coords) = stack.pop() {
        for neigh in coords.neighbors() {
            if neigh.x <= limit.x
                && neigh.y <= limit.y
                && neigh.z <= limit.z
                && neigh.x >= 0
                && neigh.y >= 0
                && neigh.z >= 0
                && !map.contains_key(&neigh)
            {
                map.insert(neigh, CubeType::Exterior);
                stack.push(neigh);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum CubeType {
    Lava,
    Exterior,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Coords {
    x: isize,
    y: isize,
    z: isize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Segment {
    start: isize,
    end: isize,
}

impl Coords {
    fn offset(&self, x: isize, y: isize, z: isize) -> Coords {
        Coords {
            x: self.x + x,
            y: self.y + y,
            z: self.z + z,
        }
    }

    fn neighbors(&self) -> [Coords; 6] {
        [
            Coords {
                x: self.x + 1,
                y: self.y,
                z: self.z,
            },
            Coords {
                x: self.x.wrapping_sub(1),
                y: self.y,
                z: self.z,
            },
            Coords {
                x: self.x,
                y: self.y + 1,
                z: self.z,
            },
            Coords {
                x: self.x,
                y: self.y.wrapping_sub(1),
                z: self.z,
            },
            Coords {
                x: self.x,
                y: self.y,
                z: self.z + 1,
            },
            Coords {
                x: self.x,
                y: self.y,
                z: self.z.wrapping_sub(1),
            },
        ]
    }
}

fn concat_lines<I: Iterator<Item = Segment>>(mut iter: I) -> Vec<Segment> {
    let mut current = match iter.next() {
        Some(segment) => segment,
        None => return vec![],
    };
    let mut result = vec![];

    for item in iter {
        if current.end + 1 >= item.start {
            current = Segment {
                start: current.start,
                end: cmp::max(current.end, item.end),
            };
        } else {
            result.push(current);
            current = item;
        }
    }

    result.push(current);
    result
}

fn cube(input: &str) -> IResult<&str, Coords> {
    map(
        separated_pair(
            integer,
            char(','),
            separated_pair(integer, char(','), integer),
        ),
        |(x, (y, z))| Coords { x, y, z },
    )(input)
}
