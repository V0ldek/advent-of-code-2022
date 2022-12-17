#![feature(is_some_and)]
use advent_of_code_2022::*;
use clap::{Parser, ValueEnum};
use color_eyre::eyre::{Context, Result};
use std::time::Instant;

#[derive(Parser, Debug)]
struct Args {
    #[clap(short, long, value_parser = clap::value_parser!(u8).range(1..26))]
    day: u8,
    #[clap(short, long, value_enum)]
    part: Part,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
enum Part {
    One,
    Two,
    Both,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    let input_directory = format!("./input/day{}", args.day);

    match args.day {
        1 => run::<Day1>(args.part, &input_directory),
        2 => run::<Day2>(args.part, &input_directory),
        3 => run::<Day3>(args.part, &input_directory),
        4 => run::<Day4>(args.part, &input_directory),
        5 => run::<Day5>(args.part, &input_directory),
        6 => run::<Day6>(args.part, &input_directory),
        7 => run::<Day7>(args.part, &input_directory),
        8 => run::<Day8>(args.part, &input_directory),
        9 => run::<Day9>(args.part, &input_directory),
        10 => run::<Day10>(args.part, &input_directory),
        11 => run::<Day11>(args.part, &input_directory),
        12 => run::<Day12>(args.part, &input_directory),
        13 => run::<Day13>(args.part, &input_directory),
        14 => run::<Day14>(args.part, &input_directory),
        15 => run::<Day15>(args.part, &input_directory),
        16 => run::<Day16>(args.part, &input_directory),
        17 => run::<Day17>(args.part, &input_directory),
        18 => run::<Day18>(args.part, &input_directory),
        19 => run::<Day19>(args.part, &input_directory),
        20 => run::<Day20>(args.part, &input_directory),
        21 => run::<Day21>(args.part, &input_directory),
        22 => run::<Day22>(args.part, &input_directory),
        23 => run::<Day23>(args.part, &input_directory),
        24 => run::<Day24>(args.part, &input_directory),
        25 => run::<Day25>(args.part, &input_directory),
        _ => unreachable!(),
    }?;

    Ok(())
}

fn run<S: Solution>(part: Part, input_directory: &str) -> Result<()> {
    let mut inputs: Vec<_> = std::fs::read_dir(input_directory)
        .wrap_err(format!("error reading input directory '{input_directory}'"))?
        .flatten()
        .filter(|x| x.file_type().is_ok_and(|t| t.is_file()))
        .filter(|x| x.path().extension().is_some_and(|ext| ext == "txt"))
        .collect();
    inputs.sort_by_key(|x| x.path());

    for input_file in inputs {
        let path = input_file.path();
        println!("Running on {}", path.display());
        let input = std::fs::read_to_string(&path)
            .wrap_err(format!("error reading file {}", path.display()))?;
        run_on_file::<S>(part, &input).wrap_err("error running solution")?;
    }

    Ok(())
}

fn run_on_file<S: Solution>(part: Part, input: &str) -> Result<()> {
    let mut solution = S::default();
    let data = solution
        .parse(input)
        .map_err(|e| e.map_input(|x| x.to_owned()))
        .wrap_err("error parsing input")?;

    let start = Instant::now();

    match part {
        Part::One => {
            let result = solution.run_part_1(&data);
            let duration = start.elapsed();
            println!("{result}");
            println!("completed in {duration:?}");
        }
        Part::Two => {
            let result = solution.run_part_2(&data);
            let duration = start.elapsed();
            println!("{result}");
            println!("completed in {duration:?}");
        }
        Part::Both => {
            let result1 = solution.run_part_1(&data);
            let after_part_1 = Instant::now();
            let part_1_duration = after_part_1 - start;
            println!("Part 1:\n{result1}");
            println!("completed in {part_1_duration:?}");

            let before_part_2 = Instant::now();
            let result2 = solution.run_part_2(&data);
            let after_part_2 = Instant::now();
            let part_2_duration = after_part_2 - before_part_2;
            println!("Part 2:\n{result2}");
            println!("completed in {part_2_duration:?}");

            println!("total elapsed: {:?}", part_1_duration + part_2_duration);
        }
    }

    Ok(())
}
