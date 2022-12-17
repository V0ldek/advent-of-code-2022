use crate::{
    parsing::{integer, line_separated},
    Solution,
};
use itertools::Itertools;
use nom::{
    bytes::complete::tag,
    combinator::{all_consuming, map},
    sequence::{preceded, separated_pair},
    IResult,
};
use std::cmp;

#[derive(Default)]
pub struct Day15 {}

const TARGET_Y: i64 = 2_000_000;
const Y_THRESHOLD: i64 = 4_000_000;
const TUNING_CONSTANT: i64 = 4_000_000;

impl Solution for Day15 {
    type Part1Result = usize;
    type Part2Result = i64;

    type Input = Vec<Sensor>;

    fn parse<'a>(
        &mut self,
        input: &'a str,
    ) -> Result<Self::Input, nom::Err<nom::error::Error<&'a str>>> {
        all_consuming(line_separated(sensor))(input).map(|x| x.1)
    }

    fn run_part_1(&mut self, data: &Self::Input) -> Self::Part1Result {
        let coverage = get_covered_segments(data, TARGET_Y);
        let without_beacons = remove_beacons(data, TARGET_Y, coverage);
        without_beacons.into_iter().map(|s| s.len()).sum()
    }

    fn run_part_2(&mut self, data: &Self::Input) -> Self::Part2Result {
        for target_y in 0..=Y_THRESHOLD {
            let coverage = get_covered_segments(data, target_y);

            if coverage.len() == 2 {
                let x = coverage[0].end + 1;
                return Coords { x, y: target_y }.tuning_frequency();
            }
        }

        0
    }
}

fn get_covered_segments(sensors: &[Sensor], target_y: i64) -> Vec<Segment> {
    let raw_segments: Vec<_> = sensors
        .iter()
        .flat_map(|x| x.get_target_segment(target_y))
        .sorted()
        .collect();
    concat_lines(raw_segments.into_iter())
}

fn remove_beacons(sensors: &[Sensor], target_y: i64, segments: Vec<Segment>) -> Vec<Segment> {
    let beacons: Vec<_> = sensors
        .iter()
        .filter(|s| s.beacon.y == target_y)
        .map(|s| s.beacon.x)
        .collect();

    remove_points(segments.into_iter(), beacons.into_iter())
}

#[derive(Debug)]
pub struct Sensor {
    location: Coords,
    beacon: Coords,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Coords {
    x: i64,
    y: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Segment {
    start: i64,
    end: i64,
}

impl Segment {
    fn len(&self) -> usize {
        usize::try_from(self.end - self.start + 1).unwrap()
    }

    fn split_at(&self, point: i64) -> (Option<Segment>, Option<Segment>) {
        let left = Segment {
            start: self.start,
            end: point - 1,
        };
        let right = Segment {
            start: point + 1,
            end: self.end,
        };

        (
            if left.start <= left.end {
                Some(left)
            } else {
                None
            },
            if right.start <= right.end {
                Some(right)
            } else {
                None
            },
        )
    }
}

impl Sensor {
    fn get_target_segment(&self, target: i64) -> Option<Segment> {
        let distance = self.location.distance_to(&self.beacon);
        let source = self.location.project_on_target_y(target);
        let distance_to_source = self.location.distance_to(&source);

        if distance >= distance_to_source {
            let delta = distance - distance_to_source;
            Some(Segment {
                start: source.x - delta,
                end: source.x + delta,
            })
        } else {
            None
        }
    }
}

impl Coords {
    fn distance_to(&self, other: &Coords) -> i64 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }

    fn project_on_target_y(&self, y: i64) -> Coords {
        Self { x: self.x, y }
    }

    fn tuning_frequency(&self) -> i64 {
        self.x * TUNING_CONSTANT + self.y
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

fn remove_points<I1: Iterator<Item = Segment>, I2: Iterator<Item = i64>>(
    segments: I1,
    mut points: I2,
) -> Vec<Segment> {
    let mut point = points.next();
    let mut result = vec![];

    fn add_if_some(result: &mut Vec<Segment>, opt_segment: Option<Segment>) {
        if let Some(segment) = opt_segment {
            result.push(segment)
        }
    }

    for segment in segments {
        let mut remainder = Some(segment);

        while point.map_or(false, |p| segment.start > p) {
            point = points.next();
        }
        while remainder.is_some() && point.map_or(false, |p| segment.end >= p) {
            let (left, right) = remainder.unwrap().split_at(point.unwrap());
            remainder = right;

            add_if_some(&mut result, left);

            point = points.next();
        }

        add_if_some(&mut result, remainder);
    }

    result
}

fn sensor(input: &str) -> IResult<&str, Sensor> {
    map(
        preceded(
            tag("Sensor at "),
            separated_pair(coords, tag(": closest beacon is at "), coords),
        ),
        |(location, beacon)| Sensor { location, beacon },
    )(input)
}

fn coords(input: &str) -> IResult<&str, Coords> {
    map(
        separated_pair(
            preceded(tag("x="), integer),
            tag(", "),
            preceded(tag("y="), integer),
        ),
        |(x, y)| Coords { x, y },
    )(input)
}
