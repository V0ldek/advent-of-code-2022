use crate::Solution;
use std::collections::HashMap;

#[derive(Default)]
pub struct Day6 {}

impl Solution for Day6 {
    type Part1Result = usize;
    type Part2Result = Self::Part1Result;

    type Input = Vec<char>;

    fn parse<'a>(
        &mut self,
        input: &'a str,
    ) -> Result<Self::Input, nom::Err<nom::error::Error<&'a str>>> {
        Ok(input.chars().into_iter().collect())
    }

    fn run_part_1(&mut self, data: &Self::Input) -> Self::Part1Result {
        find_marker(data, 4).unwrap() + 1
    }

    fn run_part_2(&mut self, data: &Self::Input) -> Self::Part2Result {
        find_marker(data, 14).unwrap() + 1
    }
}

fn find_marker(data: &[char], window_size: usize) -> Option<usize> {
    let mut char_set = CharSet::new();
    let mut start = 0;

    for (i, c) in data.iter().copied().enumerate() {
        if i - start == window_size {
            char_set.remove(data[start]);
            start += 1
        }

        char_set.add(c);

        if char_set.unique() == window_size {
            return Some(i);
        }
    }

    None
}

struct CharSet {
    counters: HashMap<char, usize>,
    unique_chars: usize,
}

impl CharSet {
    fn new() -> Self {
        Self {
            counters: HashMap::new(),
            unique_chars: 0,
        }
    }

    fn add(&mut self, c: char) {
        self.counters
            .entry(c)
            .and_modify(|x| *x += 1)
            .or_insert_with(|| {
                self.unique_chars += 1;
                1
            });
    }

    fn remove(&mut self, c: char) {
        if self.counters[&c] == 1 {
            self.counters.remove(&c);
            self.unique_chars -= 1;
        } else {
            self.counters.entry(c).and_modify(|x| *x -= 1);
        }
    }

    fn unique(&self) -> usize {
        self.unique_chars
    }
}
