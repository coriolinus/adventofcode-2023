use std::str::FromStr;

use crate::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeedRange {
    pub(crate) start: i64,
    pub(crate) length: i64,
}

impl SeedRange {
    pub fn contains(&self, value: i64) -> bool {
        (self.start..(self.start + self.length)).contains(&value)
    }

    /// Split this seed range at a particular value.
    ///
    /// `None` if this range does not contain that value.
    ///
    /// The first returned value always keeps the same start point as the origin seed range.
    /// The second returned value always starts at the split point.
    /// Lengths are adjusted appropriately.
    pub fn split_at(self, split_point: i64) -> Option<(Self, Self)> {
        self.contains(split_point).then(|| {
            let Self { start, length } = self;
            let first = Self {
                start,
                length: split_point - start,
            };
            let second = Self {
                start: split_point,
                length: start + length - split_point,
            };
            (first, second)
        })
    }
}

pub struct SeedRanges(pub Vec<SeedRange>);

impl FromStr for SeedRanges {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s
            .strip_prefix("seeds: ")
            .ok_or_else(|| Error::Parse("no seeds prefix".into()))?;

        let numbers = s
            .split_ascii_whitespace()
            .map(|token| token.parse::<i64>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_err| Error::Parse("interpreting seed value".into()))?;

        let mut ranges = Vec::with_capacity(numbers.len() / 2);
        for chunk in numbers.chunks(2) {
            let [start, length] = TryInto::<[_; 2]>::try_into(chunk)
                .map_err(|_err| Error::Parse("wrong seed range chunk size".into()))?;

            ranges.push(SeedRange { start, length });
        }

        Ok(Self(ranges))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_range_split_at() {
        let range = SeedRange {
            start: 2,
            length: 4,
        };
        let (left, right) = range.split_at(5).unwrap();
        assert_eq!(
            left,
            SeedRange {
                start: 2,
                length: 3
            }
        );
        assert_eq!(
            right,
            SeedRange {
                start: 5,
                length: 1
            }
        );
    }
}
