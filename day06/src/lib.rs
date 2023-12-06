use itertools::Itertools;
use std::path::Path;

fn parse(input: impl AsRef<Path>) -> Result<Vec<Race>, Error> {
    let data = std::fs::read_to_string(input)?;
    let mut lines = data.lines();
    let time_line = lines.next().ok_or_else(Error::parse("no time line"))?;
    let distance_line = lines.next().ok_or_else(Error::parse("no distance line"))?;

    let time_line = time_line
        .strip_prefix("Time:")
        .ok_or_else(Error::parse("wrong time prefix"))?;
    let distance_line = distance_line
        .strip_prefix("Distance:")
        .ok_or_else(Error::parse("wrong distance prefix"))?;

    let times = time_line
        .split_ascii_whitespace()
        .map(|token| token.parse::<u64>().ok());
    let distances = distance_line
        .split_ascii_whitespace()
        .map(|token| token.parse::<u64>().ok());

    let races = times
        .zip_eq(distances)
        .map::<Result<_, Error>, _>(|(time, distance)| {
            let time = time.ok_or_else(Error::parse("time not parseable as int"))?;
            let distance = distance.ok_or_else(Error::parse("distance not parseable as int"))?;
            Ok(Race { time, distance })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(races)
}

#[derive(Debug, PartialEq, Eq)]
struct Race {
    time: u64,
    distance: u64,
}

impl Race {
    fn distance_for(&self, hold_time: u64) -> u64 {
        let speed = hold_time;
        let travel_time = self.time - hold_time;
        travel_time * speed
    }

    fn ways_to_win(&self) -> u64 {
        (0..self.time)
            .filter(|&time| self.distance_for(time) > self.distance)
            .count() as _
    }

    fn combine_lexicographically(left: Self, right: Self) -> Self {
        fn next_power_of_10(value: u64) -> u64 {
            if value == 0 {
                return 1;
            }
            let mut digits = value.ilog10();
            if value > 10_u64.pow(digits) {
                digits += 1;
            }
            10_u64.pow(digits)
        }
        /// merge these numbers lexicographically
        fn merge(left: u64, right: u64) -> u64 {
            (left * next_power_of_10(right)) + right
        }
        Self {
            time: merge(left.time, right.time),
            distance: merge(left.distance, right.distance),
        }
    }
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let races = parse(input)?;
    let record_beating_product = races.iter().map(Race::ways_to_win).product::<u64>();
    println!("record beating product (pt 1): {record_beating_product}");
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    let race = parse(input)?
        .into_iter()
        .reduce(Race::combine_lexicographically)
        .ok_or(Error::NoSolution)?;
    let ways_to_win = race.ways_to_win();
    println!("ways to win merged (pt 2): {ways_to_win}");
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("no solution found")]
    NoSolution,
}

impl Error {
    fn parse(s: impl Into<String>) -> impl FnOnce() -> Self {
        move || Self::Parse(s.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn race_example_pt1() {
        let race = Race {
            time: 7,
            distance: 9,
        };
        assert_eq!(race.ways_to_win(), 4);
    }

    #[test]
    fn race_merge() {
        let left = Race {
            time: 1,
            distance: 34,
        };
        let right = Race {
            time: 2,
            distance: 56,
        };
        let expect = Race {
            time: 12,
            distance: 3456,
        };
        assert_eq!(Race::combine_lexicographically(left, right), expect);
    }
}
