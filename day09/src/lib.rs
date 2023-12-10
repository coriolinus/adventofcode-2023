use aoclib::parse;
use std::{path::Path, str::FromStr};

type Item = i32;
type Sequence = Vec<Item>;

fn predict_next_value(sequence: &Sequence) -> Item {
    let mut diffs = Vec::with_capacity(sequence.len() - 1);
    diffs.extend(sequence.windows(2).map(|window| {
        let [left, right] = window
            .try_into()
            .expect("`windows(2)` produces windows of size 2");
        right - left
    }));

    let next_diff = if diffs.iter().all(|&d| d == 0) {
        0
    } else {
        predict_next_value(&diffs)
    };

    sequence.last().copied().unwrap_or_default() + next_diff
}

// this should probably go into Aoclib
struct SpaceSep<T>(Vec<T>);

impl<T> FromStr for SpaceSep<T>
where
    T: FromStr,
{
    type Err = SpaceSepError<<T as FromStr>::Err>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ts = s
            .split_ascii_whitespace()
            .map(T::from_str)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(SpaceSep(ts))
    }
}

impl<T> SpaceSep<T> {
    pub fn into_inner(self) -> Vec<T> {
        self.0
    }
}

#[derive(Debug, thiserror::Error)]
#[error("failed to parse as space-separated line")]
struct SpaceSepError<E>(#[from] E);

pub fn part1(input: &Path) -> Result<(), Error> {
    let sequences = parse::<SpaceSep<Item>>(input)?
        .map(SpaceSep::into_inner)
        .collect::<Vec<_>>();
    let soev = sequences.iter().map(predict_next_value).sum::<Item>();
    println!("sum of extrapolated values (pt 1): {soev}");
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    unimplemented!("input file: {:?}", input)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("no solution found")]
    NoSolution,
}
