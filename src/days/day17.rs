use crate::Solution;
use nom::{
    branch::alt,
    character::complete::char,
    combinator::{all_consuming, map},
    multi::many1,
    IResult,
};
use std::{cmp, collections::HashSet};

const PART_1_PIECE_LIMIT: usize = 2022;
const PART_2_PIECE_LIMIT: usize = 1_000_000_000_000;
const PIECE_SEQUENCE: [PieceType; 5] = [
    PieceType::LongHorizontal,
    PieceType::Plus,
    PieceType::ReverseL,
    PieceType::LongVertical,
    PieceType::Square,
];

#[derive(Default)]
pub struct Day17 {}

impl Solution for Day17 {
    type Part1Result = usize;
    type Part2Result = Self::Part1Result;

    type Input = Vec<Move>;

    fn parse<'a>(
        &mut self,
        input: &'a str,
    ) -> Result<Self::Input, nom::Err<nom::error::Error<&'a str>>> {
        all_consuming(many1(mov))(input).map(|x| x.1)
    }

    fn run_part_1(&mut self, data: &Self::Input) -> Self::Part1Result {
        let piece_types = std::iter::repeat(PIECE_SEQUENCE)
            .flatten()
            .take(PART_1_PIECE_LIMIT);
        let mut moves = std::iter::repeat(data.iter().copied().enumerate()).flatten();
        let mut playground = Playground::new();

        for piece_type in piece_types {
            dispatch_piece(&mut playground, &mut moves, piece_type);
        }

        playground.watermark()
    }

    fn run_part_2(&mut self, data: &Self::Input) -> Self::Part2Result {
        let mut piece_types =
            std::iter::repeat(PIECE_SEQUENCE.iter().copied().enumerate()).flatten();
        let mut moves = std::iter::repeat(data.iter().copied().enumerate()).flatten();
        let mut playground = Playground::new();
        let mut visited = HashSet::new();
        let cycle_state;
        let mut pieces_count = 0;

        loop {
            pieces_count += 1;
            let (piece_idx, piece_type) = piece_types.next().unwrap();
            let last_move_idx = dispatch_piece(&mut playground, &mut moves, piece_type);
            if !visited.insert((piece_idx, last_move_idx)) {
                cycle_state = (piece_idx, last_move_idx);
                break;
            }
        }

        let constant_factor = playground.watermark();
        let mut cycle_length = 0;

        loop {
            cycle_length += 1;
            pieces_count += 1;
            let (piece_idx, piece_type) = piece_types.next().unwrap();
            let last_move_idx = dispatch_piece(&mut playground, &mut moves, piece_type);
            if (piece_idx, last_move_idx) == cycle_state {
                break;
            }
        }

        let cycle_factor = playground.watermark() - constant_factor;
        let remainder = (PART_2_PIECE_LIMIT - pieces_count) % cycle_length;

        for _ in 0..remainder {
            let (_, piece_type) = piece_types.next().unwrap();
            dispatch_piece(&mut playground, &mut moves, piece_type);
            pieces_count += 1;
        }

        let remaining_cycles = (PART_2_PIECE_LIMIT - pieces_count) / cycle_length;
        let remaining_factor = remaining_cycles * cycle_factor;

        playground.watermark() + remaining_factor
    }
}

fn dispatch_piece<I: Iterator<Item = (usize, Move)>>(
    playground: &mut Playground,
    moves: &mut I,
    piece_type: PieceType,
) -> usize {
    match piece_type {
        PieceType::LongHorizontal => run_piece::<LongHorizontal, _>(playground, moves),
        PieceType::Plus => run_piece::<Plus, _>(playground, moves),
        PieceType::ReverseL => run_piece::<ReverseL, _>(playground, moves),
        PieceType::LongVertical => run_piece::<LongVertical, _>(playground, moves),
        PieceType::Square => run_piece::<Square, _>(playground, moves),
    }
}

fn run_piece<P: Piece, I: Iterator<Item = (usize, Move)>>(
    playground: &mut Playground,
    moves: &mut I,
) -> usize {
    let mut piece = playground.spawn::<P>();

    loop {
        let (i, mov) = moves.next().unwrap();
        playground.mov(&mut piece, mov);

        if !playground.mov(&mut piece, Move::Down) {
            playground.emplace(piece);
            return i;
        }
    }
}

struct Playground {
    lines: Vec<Line>,
    watermark: usize,
}

struct Line {
    occupation: [bool; Playground::WIDTH + 2],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Atom(Coords);

trait Piece {
    type IntoAtomIter: Iterator<Item = Atom>;

    fn new(left_corner_coords: Coords) -> Self;

    fn height(&self) -> usize;

    fn atoms(&self) -> &[Atom];

    fn atoms_mut(&mut self) -> &mut [Atom];

    fn into_atoms(self) -> Self::IntoAtomIter;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Coords {
    x: usize,
    y: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    Left,
    Right,
    Down,
}

impl Atom {
    fn x(&self) -> usize {
        self.0.x
    }

    fn y(&self) -> usize {
        self.0.y
    }

    fn coords(&self) -> Coords {
        self.0
    }

    fn shifted(&self, mov: Move) -> Self {
        let coords = match mov {
            Move::Left => Coords {
                x: self.0.x - 1,
                y: self.0.y,
            },
            Move::Right => Coords {
                x: self.0.x + 1,
                y: self.0.y,
            },
            Move::Down => Coords {
                x: self.0.x,
                y: self.0.y - 1,
            },
        };

        Self(coords)
    }
}

impl Playground {
    const WIDTH: usize = 7;
    const EXTENSION_STEP: usize = 16;
    const VERTICAL_CLEARANCE: usize = 3;
    const LEFT_SIDE_CLEARANCE: usize = 2;
}

impl Playground {
    fn new() -> Self {
        let mut selff = Self {
            lines: vec![Line::full()],
            watermark: 0,
        };
        selff.extend();

        selff
    }

    fn spawn<P: Piece>(&mut self) -> P {
        let x = Playground::LEFT_SIDE_CLEARANCE + 1;
        let y = self.watermark + Self::VERTICAL_CLEARANCE + 1;
        let piece = P::new(Coords { x, y });

        if self.lines.len() <= y + piece.height() {
            self.extend();
        }

        piece
    }

    fn mov<P: Piece>(&mut self, piece: &mut P, mov: Move) -> bool {
        if piece
            .atoms()
            .iter()
            .all(|a| !self.is_occupied(a.shifted(mov).coords()))
        {
            for atom in piece.atoms_mut().iter_mut() {
                *atom = atom.shifted(mov);
            }

            true
        } else {
            false
        }
    }

    fn emplace<P: Piece>(&mut self, piece: P) {
        for atom in piece.into_atoms() {
            self.lines[atom.y()].set_occupied(atom.x());
            self.watermark = cmp::max(self.watermark, atom.y());
        }
    }

    fn extend(&mut self) {
        for _ in 0..Playground::EXTENSION_STEP {
            self.lines.push(Line::empty());
        }
    }

    fn watermark(&self) -> usize {
        self.watermark
    }

    fn is_occupied(&self, coords: Coords) -> bool {
        self.lines[coords.y].is_occupied(coords.x)
    }
}

impl Line {
    fn empty() -> Self {
        Self {
            occupation: [true, false, false, false, false, false, false, false, true],
        }
    }

    fn full() -> Self {
        Self {
            occupation: [true; Playground::WIDTH + 2],
        }
    }

    fn is_occupied(&self, x: usize) -> bool {
        self.occupation[x]
    }

    fn set_occupied(&mut self, x: usize) {
        self.occupation[x] = true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PieceType {
    LongHorizontal,
    Plus,
    ReverseL,
    LongVertical,
    Square,
}

struct LongHorizontal {
    atoms: [Atom; 4],
}

struct Plus {
    atoms: [Atom; 5],
}

struct ReverseL {
    atoms: [Atom; 5],
}

struct LongVertical {
    atoms: [Atom; 4],
}

struct Square {
    atoms: [Atom; 4],
}

macro_rules! array_piece_impl {
    ($size:expr) => {
        type IntoAtomIter = std::array::IntoIter<Atom, $size>;

        fn atoms(&self) -> &[Atom] {
            &self.atoms
        }

        fn atoms_mut(&mut self) -> &mut [Atom] {
            &mut self.atoms
        }

        fn into_atoms(self) -> Self::IntoAtomIter {
            self.atoms.into_iter()
        }
    };
}

impl Piece for LongHorizontal {
    fn new(left_corner_coords: Coords) -> Self {
        let x = left_corner_coords.x;
        let y = left_corner_coords.y;
        Self {
            atoms: [
                Atom(Coords { x, y }),
                Atom(Coords { x: x + 1, y }),
                Atom(Coords { x: x + 2, y }),
                Atom(Coords { x: x + 3, y }),
            ],
        }
    }

    fn height(&self) -> usize {
        1
    }

    array_piece_impl!(4);
}

impl Piece for Plus {
    fn new(left_corner_coords: Coords) -> Self {
        let x = left_corner_coords.x;
        let y = left_corner_coords.y;
        Self {
            atoms: [
                Atom(Coords { x: x + 1, y }),
                Atom(Coords { x, y: y + 1 }),
                Atom(Coords { x: x + 1, y: y + 1 }),
                Atom(Coords { x: x + 2, y: y + 1 }),
                Atom(Coords { x: x + 1, y: y + 2 }),
            ],
        }
    }

    fn height(&self) -> usize {
        3
    }

    array_piece_impl!(5);
}

impl Piece for ReverseL {
    fn new(left_corner_coords: Coords) -> Self {
        let x = left_corner_coords.x;
        let y = left_corner_coords.y;
        Self {
            atoms: [
                Atom(Coords { x, y }),
                Atom(Coords { x: x + 1, y }),
                Atom(Coords { x: x + 2, y }),
                Atom(Coords { x: x + 2, y: y + 1 }),
                Atom(Coords { x: x + 2, y: y + 2 }),
            ],
        }
    }

    fn height(&self) -> usize {
        3
    }

    array_piece_impl!(5);
}

impl Piece for LongVertical {
    fn new(left_corner_coords: Coords) -> Self {
        let x = left_corner_coords.x;
        let y = left_corner_coords.y;
        Self {
            atoms: [
                Atom(Coords { x, y }),
                Atom(Coords { x, y: y + 1 }),
                Atom(Coords { x, y: y + 2 }),
                Atom(Coords { x, y: y + 3 }),
            ],
        }
    }

    fn height(&self) -> usize {
        4
    }

    array_piece_impl!(4);
}

impl Piece for Square {
    fn new(left_corner_coords: Coords) -> Self {
        let x = left_corner_coords.x;
        let y = left_corner_coords.y;
        Self {
            atoms: [
                Atom(Coords { x, y }),
                Atom(Coords { x: x + 1, y }),
                Atom(Coords { x, y: y + 1 }),
                Atom(Coords { x: x + 1, y: y + 1 }),
            ],
        }
    }

    fn height(&self) -> usize {
        2
    }

    array_piece_impl!(4);
}

fn mov(input: &str) -> IResult<&str, Move> {
    alt((
        map(char('<'), |_| Move::Left),
        map(char('>'), |_| Move::Right),
    ))(input)
}
