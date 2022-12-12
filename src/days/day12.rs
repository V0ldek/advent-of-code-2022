use std::{
    collections::{BinaryHeap, HashSet},
    ops::Index,
};

use crate::{parsing::line_separated, Solution};
use nom::{
    branch::alt,
    character::complete::{char, satisfy},
    combinator::{all_consuming, map},
    multi::many1,
    IResult,
};

#[derive(Default)]
pub struct Day12 {}

impl Solution for Day12 {
    type Part1Result = usize;
    type Part2Result = Self::Part1Result;

    type Input = Layout;

    fn parse<'a>(
        &mut self,
        input: &'a str,
    ) -> Result<Self::Input, nom::Err<nom::error::Error<&'a str>>> {
        all_consuming(layout)(input).map(|x| x.1)
    }

    fn run_part_1(&mut self, data: &Self::Input) -> Self::Part1Result {
        find_shortest_path(&[data.start], data.end, &data.grid)
    }

    fn run_part_2(&mut self, data: &Self::Input) -> Self::Part2Result {
        let starts: Vec<Coords> = data
            .grid
            .iter()
            .filter(|&(_, h)| h == 0)
            .map(|x| x.0)
            .collect();

        find_shortest_path(&starts, data.end, &data.grid)
    }
}

fn find_shortest_path(from: &[Coords], to: Coords, grid: &Grid) -> usize {
    let mut queue: BinaryHeap<_> = from.iter().copied().map(|x| QueueEntry(0, x)).collect();
    let mut visited = HashSet::new();

    while let Some(entry) = queue.pop() {
        if entry.1 == to {
            return entry.0;
        }

        let height = grid[&entry.1];

        for direction in Direction::ALL {
            let new_coords = entry.1.mov(direction);

            if new_coords.within(grid) {
                let new_height = grid[&new_coords];

                if new_height <= height + 1 && visited.insert(new_coords) {
                    queue.push(QueueEntry(entry.0 + 1, new_coords));
                }
            }
        }
    }

    panic!("could not reach the end");
}

type Height = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct QueueEntry(usize, Coords);

impl PartialOrd for QueueEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0).map(|c| c.reverse())
    }
}

impl Ord for QueueEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0).reverse()
    }
}

#[derive(Debug)]
pub struct Layout {
    grid: Grid,
    start: Coords,
    end: Coords,
}

#[derive(Debug)]
struct Grid {
    heights: Vec<Vec<Height>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Coords {
    x: usize,
    y: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Direction {
    Up,
    Left,
    Down,
    Right,
}

impl Grid {
    fn row_count(&self) -> usize {
        self.heights.len()
    }

    fn column_count(&self) -> usize {
        self.heights[0].len()
    }

    fn iter(&self) -> GridIterator {
        GridIterator {
            grid: self,
            coords: Coords { x: 0, y: 0 },
        }
    }
}

impl Direction {
    const ALL: [Direction; 4] = [
        Direction::Up,
        Direction::Left,
        Direction::Down,
        Direction::Right,
    ];
}

impl Index<&Coords> for Grid {
    type Output = Height;

    fn index(&self, index: &Coords) -> &Self::Output {
        &self.heights[index.y][index.x]
    }
}

impl Coords {
    fn mov(&self, direction: Direction) -> Self {
        match direction {
            Direction::Up => Self {
                x: self.x,
                y: self.y.wrapping_sub(1),
            },
            Direction::Left => Self {
                x: self.x.wrapping_sub(1),
                y: self.y,
            },
            Direction::Down => Self {
                x: self.x,
                y: self.y + 1,
            },
            Direction::Right => Self {
                x: self.x + 1,
                y: self.y,
            },
        }
    }

    fn within(&self, grid: &Grid) -> bool {
        self.y < grid.row_count() && self.x < grid.column_count()
    }
}

struct GridIterator<'g> {
    grid: &'g Grid,
    coords: Coords,
}

impl<'g> Iterator for GridIterator<'g> {
    type Item = (Coords, Height);

    fn next(&mut self) -> Option<Self::Item> {
        if !self.coords.within(self.grid) {
            None
        } else {
            let result = Some((self.coords, self.grid[&self.coords]));
            self.coords = self.coords.mov(Direction::Right);

            if !self.coords.within(self.grid) {
                self.coords.x = 0;
                self.coords = self.coords.mov(Direction::Down);
            }

            result
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum InputTile {
    Elevation(char),
    Start,
    End,
}

fn layout(input: &str) -> IResult<&str, Layout> {
    map(tiles, |tiles| {
        let mut start = None;
        let mut end = None;

        let grid = tiles
            .into_iter()
            .enumerate()
            .map(|(y, row)| {
                row.into_iter()
                    .enumerate()
                    .map(|(x, tile)| match tile {
                        InputTile::Elevation(c) => c as Height - 'a' as Height,
                        InputTile::Start => {
                            start = Some(Coords { x, y });
                            0
                        }
                        InputTile::End => {
                            end = Some(Coords { x, y });
                            'z' as Height - 'a' as Height
                        }
                    })
                    .collect()
            })
            .collect();

        Layout {
            grid: Grid { heights: grid },
            start: start.unwrap(),
            end: end.unwrap(),
        }
    })(input)
}

fn tiles(input: &str) -> IResult<&str, Vec<Vec<InputTile>>> {
    line_separated(many1(alt((
        map(char('S'), |_| InputTile::Start),
        map(char('E'), |_| InputTile::End),
        map(satisfy(|c| c.is_ascii_lowercase()), InputTile::Elevation),
    ))))(input)
}
