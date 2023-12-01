use aoclib::parse;
use std::{path::Path, str::FromStr};

struct NumericDigit(u32);

impl FromStr for NumericDigit {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let digits = s.chars().filter_map(|c| c.to_digit(10)).collect::<Vec<_>>();
        let value = digits
            .first()
            .zip(digits.last())
            .map(|(first, last)| *first * 10 + *last)
            .unwrap_or_default();
        Ok(Self(value))
    }
}

struct SpellOrNumericDigit(u32);

impl FromStr for SpellOrNumericDigit {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const DIGIT_NAMES: [&str; 9] = [
            "one", "two", "three", "four", "five", "six", "seven", "eight", "nine",
        ];

        fn digit_at_idx(s: &str, idx: usize) -> Option<u32> {
            s.as_bytes()
                .get(idx)
                .and_then(|b| (*b as char).to_digit(10))
                .or_else(|| {
                    for (digit_idx, name) in DIGIT_NAMES.iter().copied().enumerate() {
                        if s.get(idx..)?.starts_with(name) {
                            return Some((digit_idx as u32) + 1);
                        }
                    }
                    None
                })
        }

        let digits = (0..s.len())
            .filter_map(|idx| digit_at_idx(s, idx))
            .collect::<Vec<_>>();
        let value = digits
            .first()
            .zip(digits.last())
            .map(|(first, last)| *first * 10 + *last)
            .unwrap_or_default();

        Ok(Self(value))
    }
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let sum = parse::<NumericDigit>(input)?
        .map(|value| value.0)
        .sum::<u32>();
    println!("sum of calibration values (pt 1): {sum}");
    Ok(())
}

// not right; too high: 54112
pub fn part2(input: &Path) -> Result<(), Error> {
    let sum = parse::<SpellOrNumericDigit>(input)?
        .map(|value| value.0)
        .sum::<u32>();
    println!("sum of calibration values (pt 2): {sum}");
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("no solution found")]
    NoSolution,
}
