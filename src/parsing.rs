use nom::{
    character::complete::{char as nom_char, digit1},
    combinator::{map_res, recognize},
    error::ParseError,
    multi::separated_list0,
    IResult, Parser,
};

pub fn line_separated<I, O, E, F>(f: F) -> impl FnMut(I) -> IResult<I, Vec<O>, E>
where
    I: Clone + nom::InputLength + nom::Slice<std::ops::RangeFrom<usize>> + nom::InputIter,
    F: Parser<I, O, E>,
    E: ParseError<I>,
    <I as nom::InputIter>::Item: nom::AsChar,
{
    separated_list0(nom_char('\n'), f)
}

pub fn integer(input: &str) -> IResult<&str, u64> {
    map_res(recognize(digit1), str::parse)(input)
}