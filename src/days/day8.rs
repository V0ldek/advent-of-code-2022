use crate::{parsing::line_separated, Solution};
use itertools::{FoldWhile, Itertools};
use nom::{
    character::complete::satisfy,
    combinator::{all_consuming, map},
    multi::many1,
};
use std::fmt::Display;
use std::ops::{Index, IndexMut};

#[derive(Default)]
pub struct Day8 {}

impl Solution for Day8 {
    type Part1Result = usize;
    type Part2Result = u64;

    type Input = Grid<Height>;

    fn parse<'a>(
        &mut self,
        input: &'a str,
    ) -> Result<Self::Input, nom::Err<nom::error::Error<&'a str>>> {
        map(
            all_consuming(line_separated(many1(map(
                satisfy(|x| x.is_ascii_digit()),
                |x| x.to_digit(10).unwrap().try_into().unwrap(),
            )))),
            |g| Grid { rows: g },
        )(input)
        .map(|x| x.1)
    }

    fn run_part_1(&mut self, data: &Self::Input) -> Self::Part1Result {
        data.iter()
            .filter(|&(coords, height)| {
                DIRECTIONS.iter().any(|&direction| {
                    let maximum = data.directional_iter(coords, direction).max();
                    maximum.map_or(true, |h| h < height)
                })
            })
            .count()
    }

    fn run_part_2(&mut self, data: &Self::Input) -> Self::Part2Result {
        data.iter()
            .map(|(coords, height)| {
                DIRECTIONS
                    .iter()
                    .map(|&direction| {
                        data.directional_iter(coords, direction)
                            .fold_while(0_u64, |acc, h| {
                                if h < height {
                                    FoldWhile::Continue(acc + 1)
                                } else {
                                    FoldWhile::Done(acc + 1)
                                }
                            })
                            .into_inner()
                    })
                    .product()
            })
            .max()
            .unwrap()
    }
}

type Height = usize;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
struct Coordinates {
    row: usize,
    column: usize,
}

pub struct Grid<T> {
    rows: Vec<Vec<T>>,
}

struct DirectionalGridIterator<'g, T> {
    grid: &'g Grid<T>,
    direction: Direction,
    coords: Coordinates,
}

struct WholeGridIterator<'g, T> {
    grid: &'g Grid<T>,
    coords: Coordinates,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Direction {
    Up,
    Left,
    Down,
    Right,
}

const DIRECTIONS: [Direction; 4] = [
    Direction::Up,
    Direction::Left,
    Direction::Down,
    Direction::Right,
];

impl Coordinates {
    fn within_grid<T>(&self, grid: &Grid<T>) -> bool {
        self.row < grid.row_len() && self.column < grid.column_len()
    }

    fn mov(&self, direction: Direction) -> Self {
        match direction {
            Direction::Up => self.up(),
            Direction::Left => self.left(),
            Direction::Right => self.right(),
            Direction::Down => self.down(),
        }
    }

    fn up(&self) -> Self {
        Coordinates {
            row: self.row.wrapping_sub(1),
            column: self.column,
        }
    }

    fn down(&self) -> Self {
        Coordinates {
            row: self.row + 1,
            column: self.column,
        }
    }

    fn left(&self) -> Self {
        Coordinates {
            row: self.row,
            column: self.column.wrapping_sub(1),
        }
    }

    fn right(&self) -> Self {
        Coordinates {
            row: self.row,
            column: self.column + 1,
        }
    }
}

impl<T> Grid<T> {
    fn row_len(&self) -> usize {
        self.rows.len()
    }

    fn column_len(&self) -> usize {
        self.rows[0].len()
    }

    fn iter(&self) -> WholeGridIterator<T> {
        WholeGridIterator::new(self)
    }

    fn directional_iter(
        &self,
        origin: Coordinates,
        direction: Direction,
    ) -> DirectionalGridIterator<T> {
        DirectionalGridIterator::new(self, origin, direction)
    }
}

impl<'g, T> WholeGridIterator<'g, T> {
    fn new(grid: &'g Grid<T>) -> Self {
        Self {
            grid,
            coords: Coordinates { row: 0, column: 0 },
        }
    }

    fn advance(&mut self) {
        if self.coords.column < self.grid.column_len() - 1 {
            self.coords.column += 1;
        } else {
            self.coords.row += 1;
            self.coords.column = 0;
        }
    }
}

impl<'g, T> Iterator for WholeGridIterator<'g, T> {
    type Item = (Coordinates, &'g T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.coords.within_grid(self.grid) {
            let res = Some((self.coords, &self.grid[&self.coords]));
            self.advance();
            res
        } else {
            None
        }
    }
}

impl<'g, T> DirectionalGridIterator<'g, T> {
    fn new(grid: &'g Grid<T>, origin: Coordinates, direction: Direction) -> Self {
        let coords = origin.mov(direction);
        Self {
            grid,
            coords,
            direction,
        }
    }
}

impl<'g, T> Iterator for DirectionalGridIterator<'g, T> {
    type Item = &'g T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.coords.within_grid(self.grid) {
            let res = Some(&self.grid[&self.coords]);
            self.coords = self.coords.mov(self.direction);
            res
        } else {
            None
        }
    }
}

impl<T> Index<&Coordinates> for Grid<T> {
    type Output = T;

    fn index(&self, index: &Coordinates) -> &Self::Output {
        &self.rows[index.row][index.column]
    }
}

impl<T> IndexMut<&Coordinates> for Grid<T> {
    fn index_mut(&mut self, index: &Coordinates) -> &mut Self::Output {
        &mut self.rows[index.row][index.column]
    }
}

impl<T: Display> Display for Grid<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in self.rows.iter() {
            for column in row.iter() {
                write!(f, "{column}")?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}
