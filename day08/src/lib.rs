use std::{
    ops::{Div, Mul, Rem},
    path::Path,
};

use models::{Direction, Network};

use crate::models::{directions_iter, INITIAL_NAME, TARGET_NAME};

pub mod input;
pub mod models;

fn steps_for(
    network: &Network,
    position: &mut usize,
    directions: &[Direction],
    is_finished: impl Fn(usize) -> bool,
) -> usize {
    let mut total = None;

    for (steps, direction) in directions_iter(directions).enumerate() {
        if is_finished(*position) {
            total = Some(steps);
            break;
        }

        *position = network.step(*position, direction);
    }

    total
        .expect("as directions is an infinite iterator, we are only here if we've assigned a total")
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let (directions, network) = input::parse(input)?;
    let target = network.position_of(TARGET_NAME).ok_or(Error::NoSolution)?;
    let mut position = network.position_of(INITIAL_NAME).ok_or(Error::NoSolution)?;

    let total_steps = steps_for(&network, &mut position, &directions, |position| {
        position == target
    });

    println!("total steps (pt 1): {total_steps}");
    Ok(())
}

// these two should almost certainly go into aoclib

/// Euclid's Algorithm
fn gcd2<T>(a: T, b: T) -> T
where
    T: Copy + Eq + Default + Rem<Output = T>,
{
    if a == T::default() {
        b
    } else if b == T::default() {
        a
    } else {
        gcd2(b, a % b)
    }
}

/// Euclid's Algorithm
#[allow(dead_code)]
fn gcd<T>(ts: &[T]) -> T
where
    T: Copy + Eq + Default + Rem<Output = T>,
{
    ts.iter().copied().reduce(gcd2).unwrap_or_default()
}

fn lcm2<T>(a: T, b: T) -> T
where
    T: Copy + Eq + Default + Rem<Output = T> + Mul<Output = T> + Div<Output = T>,
{
    let divisor = gcd2(a, b);
    if divisor == T::default() {
        return a * b;
    }
    a * b / divisor
}

fn lcm<T>(ts: &[T]) -> T
where
    T: Copy + Eq + Default + Rem<Output = T> + Mul<Output = T> + Div<Output = T>,
{
    ts.iter().copied().reduce(lcm2).unwrap_or_default()
}

pub fn part2(input: &Path) -> Result<(), Error> {
    // this feels like an occasion for chinese remainder theorem, because it rarely fails to show up in AoC at some point,
    // but it seems non-obvious that this will actually work.
    //
    // let's give it a shot the iterative way; if it takes too long to solve, we can try going CRT on it.
    //
    // [edit] yeah, 3 hours wasn't enough, and I'm not going to let it do more than that. Saw by accident that LCM
    // works out pretty well, which is nice, becasue CRT is complicated. Let's try it on my input though.

    let (directions, network) = input::parse(input)?;
    let mut positions = network
        .names()
        // v-- this line is _very important_! do not omit!
        .filter(|name| name.ends_with('A'))
        .map(|name| network.position_of(name).ok_or(Error::NoSolution))
        .collect::<Result<Vec<_>, _>>()?;

    let steps = positions
        .iter_mut()
        .map(|position| {
            steps_for(&network, position, &directions, |position| {
                network
                    .name_of(position)
                    .expect("all valid positions have names")
                    .ends_with('Z')
            }) as u128
        })
        .collect::<Vec<_>>();

    let total_steps = lcm(&steps);

    println!("total steps (pt 2): {total_steps}");
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(&[], 0)]
    #[case(&[1], 1)]
    #[case(&[1, 2, 3, 4, 5], 60)]
    #[case(&[2, 4, 6, 8, 10], 120)]
    #[case(&[3, 6, 9, 12, 15], 180)]
    #[case(&[21, 110], 2310)]
    fn test_lcm(#[case] ts: &[u32], #[case] expect: u32) {
        assert_eq!(lcm(ts), expect);
    }
}
