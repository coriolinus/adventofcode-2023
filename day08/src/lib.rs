use std::path::Path;

use crate::models::{directions_iter, INITIAL_NAME, TARGET_NAME};

pub mod input;
pub mod models;

pub fn part1(input: &Path) -> Result<(), Error> {
    let (directions, network) = input::parse(input)?;
    let target = network.position_of(TARGET_NAME).ok_or(Error::NoSolution)?;
    let mut position = network.position_of(INITIAL_NAME).ok_or(Error::NoSolution)?;
    let mut total_steps = None;

    for (steps, direction) in directions_iter(&directions).enumerate() {
        if position == target {
            total_steps = Some(steps);
            break;
        }

        position = network.step(position, direction);
    }

    let total_steps = total_steps.ok_or(Error::NoSolution)?;

    println!("total steps (pt 1): {total_steps}");
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    unimplemented!("input file: {:?}", input)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("no solution found")]
    NoSolution,
}
