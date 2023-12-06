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
        .map(|token| token.parse::<u32>().ok());
    let distances = distance_line
        .split_ascii_whitespace()
        .map(|token| token.parse::<u32>().ok());

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

struct Race {
    time: u32,
    distance: u32,
}

impl Race {
    fn distance_for(&self, hold_time: u32) -> u32 {
        let speed = hold_time;
        let travel_time = self.time - hold_time;
        travel_time * speed
    }

    fn ways_to_win(&self) -> u32 {
        (0..self.time)
            .filter(|&time| self.distance_for(time) > self.distance)
            .count() as _
    }
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let races = parse(input)?;
    let record_beating_product = races.iter().map(Race::ways_to_win).product::<u32>();
    println!("record beating product (pt 1): {record_beating_product}");
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    unimplemented!("input file: {:?}", input)
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
}
