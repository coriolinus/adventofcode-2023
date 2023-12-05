use std::str::FromStr;

use crate::{map_entry::MapEntry, Error};

#[derive(Debug)]
pub struct Map {
    name: String,
    entries: Vec<MapEntry>,
}

impl Map {
    /// Both validate that there are no ambiguous inputs, and apply the internal precondition that entries are sorted by `source_start`.
    fn validate(&mut self) -> Result<(), Error> {
        self.entries.sort_by_key(|entry| entry.source_start);
        for window in self.entries.windows(2) {
            let [left, right] = TryInto::<[_; 2]>::try_into(window)
                .expect("`windows(2)` always produces a window of size 2");
            if left.source_end() > right.source_start {
                let input = right.source_start;
                let output1 = left.apply(input);
                let output2 = right.apply(input);
                if output1 == output2 {
                    // technically the ranges overlapped, but they formed a contiguous whole,
                    // so there's no ambiguity after all
                    continue;
                }
                let name = self.name.clone();
                return Err(Error::Overlaps {
                    name,
                    input,
                    output1,
                    output2,
                });
            }
        }

        Ok(())
    }

    pub fn new(
        name: impl Into<String>,
        entries: impl IntoIterator<Item = MapEntry>,
    ) -> Result<Self, Error> {
        let name = name.into();
        let entries = entries.into_iter().collect();
        let mut map = Map { name, entries };
        map.validate()?;
        Ok(map)
    }

    pub fn apply(&self, value: i64) -> i64 {
        // linear scan might seem like an odd choice here, but I think it's justified:
        // there are only ~40 entries for any particular map in the input, and that will be "fast enough".
        // a more complicated data structure seems likely to introduce overhead which might overwhelm the
        // theoretical speed advantages, and is very likely to introduce opportunities for bugs to slip in
        for entry in &self.entries {
            if entry.contains(value) {
                return entry.apply(value);
            }
        }
        value
    }
}

impl FromStr for Map {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();
        let name_line = lines
            .next()
            .ok_or_else(|| Error::Parse("no name line".into()))?;
        let name = name_line
            .trim_end()
            .strip_suffix(" map:")
            .ok_or_else(|| Error::Parse(format!("not a map line: {name_line}")))?;

        let entries = lines
            .filter(|line| !line.is_empty())
            .map(MapEntry::from_str)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| Error::Parse(format!("in {name}: {err}")))?;

        Self::new(name, entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(0, 0)]
    #[case(49, 49)]
    #[case(50, 52)]
    #[case(51, 53)]
    #[case(96, 98)]
    #[case(97, 99)]
    #[case(98, 50)]
    #[case(99, 51)]
    fn pt1_example(#[case] value: i64, #[case] expect: i64) {
        let map = Map::new(
            "seed-to-soil",
            [
                MapEntry {
                    destination_start: 50,
                    source_start: 98,
                    range_length: 2,
                },
                MapEntry {
                    destination_start: 52,
                    source_start: 50,
                    range_length: 48,
                },
            ],
        )
        .unwrap();

        let got = map.apply(value);
        assert_eq!(got, expect);
    }
}
