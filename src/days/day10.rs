use std::fmt::Display;

use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{all_consuming, map},
    sequence::preceded,
    IResult,
};

use crate::{
    parsing::{integer, line_separated},
    Solution,
};

#[derive(Default)]
pub struct Day10 {}

impl Solution for Day10 {
    type Part1Result = i64;
    type Part2Result = Crt;

    type Input = Vec<Command>;

    fn parse<'a>(
        &mut self,
        input: &'a str,
    ) -> Result<Self::Input, nom::Err<nom::error::Error<&'a str>>> {
        all_consuming(line_separated(parse_command))(input).map(|x| x.1)
    }

    fn run_part_1(&mut self, data: &Self::Input) -> Self::Part1Result {
        let cmd_details = data.iter().scan((1, 0), |(x, t), cmd| {
            let t_start = *t;
            let x_start = *x;

            *t += cmd.duration();
            if let Command::AddX(n) = cmd {
                *x += n;
            }

            Some(CmdDetails {
                t_start,
                t_end: *t,
                x_start,
            })
        });

        let mut signal = SignalStrength::new();

        for cmd in cmd_details {
            if let Some(cycle) = signal.next_interesting_cycle() {
                if cmd.executes_during(cycle) {
                    signal.record_next(cmd.x_start);
                }
            } else {
                break;
            }
        }

        signal.total
    }

    fn run_part_2(&mut self, data: &Self::Input) -> Self::Part2Result {
        let mut crt = Crt::new(3);
        let mut x = 0;

        for cmd in data {
            crt.move_cursor_by(cmd.duration());

            match cmd {
                Command::AddX(n) => {
                    x += n;
                    crt.move_sprite(x);
                }
                Command::Noop => (),
            }
        }

        crt
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Command {
    AddX(i64),
    Noop,
}

#[derive(Debug)]
struct CmdDetails {
    t_start: usize,
    t_end: usize,
    x_start: i64,
}

struct SignalStrength {
    cycles: Vec<usize>,
    cycles_idx: usize,
    total: i64,
}

pub struct Crt {
    buffer: [char; Self::HEIGHT * Self::WIDTH],
    cursor_idx: usize,
    sprite_idx: i64,
    sprite_len: usize,
}

impl Crt {
    const HEIGHT: usize = 6;
    const WIDTH: usize = 40;
    const EMPTY_PIXEL: char = '.';
    const LIT_PIXEL: char = '#';

    fn new(sprite_len: usize) -> Self {
        Self {
            buffer: [Self::EMPTY_PIXEL; Self::HEIGHT * Self::WIDTH],
            cursor_idx: 0,
            sprite_idx: 0,
            sprite_len,
        }
    }

    fn move_sprite(&mut self, start: i64) {
        self.sprite_idx = start;
    }

    fn move_cursor_by(&mut self, distance: usize) {
        for _ in 0..distance {
            self.move_cursor();
        }
    }

    fn move_cursor(&mut self) {
        if self.cursor_idx >= self.buffer.len() {
            return;
        }

        if self.is_sprite_visible() {
            self.buffer[self.cursor_idx] = Self::LIT_PIXEL;
        }

        self.cursor_idx += 1;
    }

    fn is_sprite_visible(&self) -> bool {
        let column = self.cursor_idx % Self::WIDTH;

        (self.sprite_idx < 0 || column >= self.sprite_idx as usize)
            && (self.sprite_end() >= 0 && column <= self.sprite_end() as usize)
    }

    fn sprite_end(&self) -> i64 {
        self.sprite_idx + (self.sprite_len as i64) - 1
    }
}

impl CmdDetails {
    fn executes_during(&self, cycle: usize) -> bool {
        self.t_start < cycle && self.t_end >= cycle
    }
}

impl SignalStrength {
    fn new() -> Self {
        Self {
            cycles: vec![20, 60, 100, 140, 180, 220],
            cycles_idx: 0,
            total: 0,
        }
    }

    fn next_interesting_cycle(&self) -> Option<usize> {
        if self.cycles_idx == self.cycles.len() {
            None
        } else {
            Some(self.cycles[self.cycles_idx])
        }
    }

    fn record_next(&mut self, signal: i64) {
        let strength = (self.next_interesting_cycle().unwrap() as i64) * signal;
        self.total += strength;
        self.cycles_idx += 1;
    }
}

impl Command {
    fn duration(&self) -> usize {
        match self {
            Command::AddX(_) => 2,
            Command::Noop => 1,
        }
    }
}

impl Display for Crt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, p) in self.buffer.iter().enumerate() {
            write!(f, "{p}")?;

            if (i + 1) % Self::WIDTH == 0 {
                writeln!(f)?;
            }
        }

        Ok(())
    }
}

fn parse_command(input: &str) -> IResult<&str, Command> {
    alt((parse_addx, parse_noop))(input)
}

fn parse_addx(input: &str) -> IResult<&str, Command> {
    map(preceded(tag("addx "), integer), Command::AddX)(input)
}

fn parse_noop(input: &str) -> IResult<&str, Command> {
    map(tag("noop"), |_| Command::Noop)(input)
}
