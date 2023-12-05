use std::path::Path;

use aoclib::input::parse_two_phase;
use map::Map;
use seed_ranges::SeedRanges;
use seeds::Seeds;

mod map;
mod map_entry;
mod seed_ranges;
mod seeds;

pub fn part1(input: &Path) -> Result<(), Error> {
    let (seeds, maps) = parse_two_phase::<Seeds, Map>(input)?;
    let maps = maps.collect::<Vec<_>>();
    // note: we depend on the input file's map ordering being appropriate, allowing a direct pass-through.

    let lowest_location = seeds
        .0
        .iter()
        .copied()
        .map(|mut value| {
            for map in &maps {
                value = map.apply(value);
            }
            value
        })
        .min()
        .ok_or(Error::NoSolution)?;

    println!("lowest location (pt 1): {lowest_location}");
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    let (seeds, maps) = parse_two_phase::<SeedRanges, Map>(input)?;
    let maps = maps.collect::<Vec<_>>();
    // note: we depend on the input file's map ordering being appropriate, allowing a direct pass-through.
    todo!()
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("overlaps in map {name}: input {input} ambiguous between {output1} and {output2}")]
    Overlaps {
        name: String,
        input: i64,
        output1: i64,
        output2: i64,
    },
    #[error("no solution found")]
    NoSolution,
}

impl From<aoclib::input::TwoPhaseError> for Error {
    fn from(value: aoclib::input::TwoPhaseError) -> Self {
        Self::Parse(value.to_string())
    }
}
