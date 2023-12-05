use std::{cmp::Ordering, str::FromStr};

use crate::{map_entry::MapEntry, seed_ranges::SeedRange, Error};

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

    /// Apply this map to a seed range.
    ///
    /// If the input `range` either fits entirely within a single `MapEntry`,
    /// or if it lies entirely outside any of them, then there will be a single
    /// item in the output vector. Otherwsie, this will produce a number of
    /// output items determined by the number of distint entries and gaps spanned
    /// by the `range`.
    ///
    /// Note that in the event that entries overlap each other, this may produce
    /// more than one output range even if certain output ranges are contiguous.
    ///
    /// This function will never produce an empty output vector.
    pub fn apply_range(&self, mut range: SeedRange) -> Vec<SeedRange> {
        let _original_length = range.length;

        let mut out = Vec::new();
        let mut segment: SeedRange;

        // implementation note: I'm doing manual bounds checking and unwrapping previously-checked
        // bounds quite a lot in here. given more time, I might be able to figure out a more elegant approach,
        // but this is what we've got for now.

        let mut eidx = 0;
        loop {
            // break if we're out of bounds
            let Some(entry) = self.entries.get(eidx) else {
                break;
            };

            // fast-forward to the first interesting point
            if entry.source_end() <= range.start {
                eidx += 1;
                continue;
            }

            // sanity: have we overshot our boundaries entirely?
            if entry.source_start >= range.end() {
                break;
            }

            if entry.source_start > range.start {
                // we need an unmodified segment before the entry,
                // and we know we're in bounds
                (segment, range) = range.split_at(entry.source_start).unwrap();
                out.push(segment);
            }

            debug_assert!(entry.source_start <= range.start);

            if entry.source_end() < range.end() {
                // we need to split again, to snip out the mapped segment
                (segment, range) = range.split_at(entry.source_end()).unwrap();
                out.push(segment + entry.delta());
                eidx += 1;
            } else {
                // we can push the remainder of the range now, then break;
                // we're done with our range
                out.push(range + entry.delta());
                // ensure we don't re-add the range again
                range.length = 0;
                break;
            }
        }

        if range.length > 0 {
            // most likely cause: all map entries were below the low end of the seed range
            out.push(range);
        }

        debug_assert_eq!(
            out.iter().map(|range| range.length).sum::<i64>(),
            _original_length
        );

        out
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

    #[test]
    fn apply_range_under() {
        let range = SeedRange {
            start: 0,
            length: 1,
        };
        let map = Map::new(
            "under",
            [MapEntry {
                destination_start: 4,
                source_start: 2,
                range_length: 2,
            }],
        )
        .unwrap();

        assert_eq!(map.apply_range(range), vec![range]);
    }

    #[test]
    fn apply_range_over() {
        let range = SeedRange {
            start: 10,
            length: 1,
        };
        let map = Map::new(
            "over",
            [MapEntry {
                destination_start: 4,
                source_start: 2,
                range_length: 2,
            }],
        )
        .unwrap();

        assert_eq!(map.apply_range(range), vec![range]);
    }

    #[test]
    fn apply_range_lower_bound() {
        let range = SeedRange {
            start: 2,
            length: 1,
        };
        let map = Map::new(
            "lower-bound",
            [MapEntry {
                destination_start: 4,
                source_start: 2,
                range_length: 2,
            }],
        )
        .unwrap();

        let expect = vec![SeedRange {
            start: 4,
            length: 1,
        }];

        assert_eq!(map.apply_range(range), expect);
    }

    #[test]
    fn apply_range_upper_bound_inner() {
        let range = SeedRange {
            start: 3,
            length: 1,
        };
        let map = Map::new(
            "upper-bound-inner",
            [MapEntry {
                destination_start: 4,
                source_start: 2,
                range_length: 2,
            }],
        )
        .unwrap();

        let expect = vec![SeedRange {
            start: 5,
            length: 1,
        }];

        assert_eq!(map.apply_range(range), expect);
    }

    #[test]
    fn apply_range_upper_bound_outer() {
        let range = SeedRange {
            start: 4,
            length: 1,
        };
        let map = Map::new(
            "upper-bound-outer",
            [MapEntry {
                destination_start: 4,
                source_start: 2,
                range_length: 2,
            }],
        )
        .unwrap();

        let expect = vec![SeedRange {
            start: 4,
            length: 1,
        }];

        assert_eq!(map.apply_range(range), expect);
    }

    #[test]
    fn apply_range_interior() {
        let range = SeedRange {
            start: 2,
            length: 1,
        };
        let map = Map::new(
            "interior",
            [MapEntry {
                destination_start: 10,
                source_start: 1,
                range_length: 3,
            }],
        )
        .unwrap();

        let expect = vec![SeedRange {
            start: 11,
            length: 1,
        }];

        assert_eq!(map.apply_range(range), expect);
    }

    #[test]
    fn apply_range_exterior() {
        let range = SeedRange {
            start: 2,
            length: 6,
        };
        let map = Map::new(
            "exterior",
            [MapEntry {
                destination_start: 10,
                source_start: 4,
                range_length: 2,
            }],
        )
        .unwrap();

        let expect = vec![
            SeedRange {
                start: 2,
                length: 2,
            },
            SeedRange {
                start: 10,
                length: 2,
            },
            SeedRange {
                start: 6,
                length: 2,
            },
        ];

        assert_eq!(map.apply_range(range), expect);
    }

    #[test]
    fn apply_range_overlap_lower() {
        let range = SeedRange {
            start: 2,
            length: 2,
        };
        let map = Map::new(
            "overlap-lower",
            [MapEntry {
                destination_start: 10,
                source_start: 3,
                range_length: 2,
            }],
        )
        .unwrap();

        let expect = vec![
            SeedRange {
                start: 2,
                length: 1,
            },
            SeedRange {
                start: 10,
                length: 1,
            },
        ];

        assert_eq!(map.apply_range(range), expect);
    }

    #[test]
    fn apply_range_overlap_upper() {
        let range = SeedRange {
            start: 4,
            length: 2,
        };
        let map = Map::new(
            "overlap-upper",
            [MapEntry {
                destination_start: 10,
                source_start: 3,
                range_length: 2,
            }],
        )
        .unwrap();

        let expect = vec![
            SeedRange {
                start: 11,
                length: 1,
            },
            SeedRange {
                start: 5,
                length: 1,
            },
        ];

        assert_eq!(map.apply_range(range), expect);
    }

    #[test]
    fn apply_range_complex() {
        let range = SeedRange {
            start: 9,
            length: 9,
        };
        let map = Map::new(
            "test",
            [
                MapEntry {
                    destination_start: 13,
                    source_start: 4,
                    range_length: 1,
                },
                MapEntry {
                    destination_start: 0,
                    source_start: 7,
                    range_length: 3,
                },
                MapEntry {
                    destination_start: 7,
                    source_start: 11,
                    range_length: 2,
                },
                MapEntry {
                    destination_start: 9,
                    source_start: 14,
                    range_length: 1,
                },
                MapEntry {
                    destination_start: 10,
                    source_start: 16,
                    range_length: 3,
                },
            ],
        )
        .unwrap();

        let expect = vec![
            SeedRange {
                start: 2,
                length: 1,
            },
            SeedRange {
                start: 10,
                length: 1,
            },
            SeedRange {
                start: 7,
                length: 2,
            },
            SeedRange {
                start: 13,
                length: 1,
            },
            SeedRange {
                start: 9,
                length: 1,
            },
            SeedRange {
                start: 15,
                length: 1,
            },
            SeedRange {
                start: 10,
                length: 2,
            },
        ];

        assert_eq!(map.apply_range(range), expect);
    }
}
