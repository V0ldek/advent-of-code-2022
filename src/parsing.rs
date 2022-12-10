use std::str::FromStr;

use nom::{
    character::complete::{char, digit1, line_ending, multispace0},
    combinator::{map_res, opt, recognize},
    error::ParseError,
    multi::separated_list0,
    sequence::{terminated, tuple},
    IResult, Parser,
};

pub fn line_separated<I, O, E, F>(f: F) -> impl FnMut(I) -> IResult<I, Vec<O>, E>
where
    I: Clone
        + nom::InputLength
        + nom::Slice<std::ops::RangeFrom<usize>>
        + nom::Slice<std::ops::RangeTo<usize>>
        + nom::Slice<std::ops::Range<usize>>
        + nom::InputIter
        + nom::InputTake
        + nom::Compare<&'static str>
        + nom::InputTakeAtPosition,
    F: Parser<I, O, E>,
    E: ParseError<I>,
    <I as nom::InputIter>::Item: nom::AsChar + std::clone::Clone,
    <I as nom::InputTakeAtPosition>::Item: nom::AsChar + std::clone::Clone,
{
    terminated(separated_list0(line_ending, f), multispace0)
}

pub fn integer<F: FromStr>(input: &str) -> IResult<&str, F> {
    map_res(recognize(tuple((opt(char('-')), digit1))), str::parse)(input)
}
