use aoclib::{geometry::tile::DisplayWidth, parse, CommaSep};
use arrayvec::ArrayVec;
use std::{
    ops::Shl,
    path::Path,
    str::{self, FromStr},
};

// Actual input has at most 20 conditions, so we can assume that we never have more than 32 unknown elements.
type Word = u32;
type Conditions = ArrayVec<Condition, 32>;
// In a 32 bit word, we cannot possibly have more than 16 distinct groups.
type DamageGroups = ArrayVec<u8, 16>;

fn get_bit<I>(value: Word, idx: I) -> bool
where
    Word: Shl<I, Output = Word>,
{
    value & (1 << idx) != 0
}

fn set_bit<I>(value: Word, idx: I, bit_value: bool) -> Word
where
    Word: Shl<I, Output = Word>,
{
    if bit_value {
        value | (1 << idx)
    } else {
        value & !(1 << idx)
    }
}

/// Compute the actual damage groups from a mapped value
fn damage_groups(mut mapped_value: Word) -> DamageGroups {
    let mut found_groups = DamageGroups::new();

    while mapped_value != 0 {
        // get rid of the trailing 0s, then count the trailing ones
        mapped_value >>= mapped_value.trailing_zeros();
        let group = mapped_value.trailing_ones() as _;
        found_groups.push(group);
        mapped_value >>= group;
    }

    found_groups.reverse();
    found_groups
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, parse_display::FromStr, parse_display::Display, strum::EnumIs,
)]
enum Condition {
    #[display(".")]
    Operational,
    #[display("#")]
    Damaged,
    #[display("?")]
    Unknown,
}

impl DisplayWidth for Condition {
    const DISPLAY_WIDTH: usize = 1;
}

// another candidate for aoclib
struct Unsep<T>(pub Vec<T>);

impl<T> FromStr for Unsep<T>
where
    T: FromStr + DisplayWidth,
{
    type Err = ParseUnspError<<T as FromStr>::Err>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let chunker = s.as_bytes().chunks_exact(T::DISPLAY_WIDTH);
        if !chunker.remainder().is_empty() {
            return Err(ParseUnspError::InputRem);
        }
        let values = chunker
            .map(|chunk| {
                let s = str::from_utf8(chunk)?;
                T::from_str(s).map_err(ParseUnspError::Instance)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self(values))
    }
}

#[derive(Debug, Clone)]
struct ConditionRecord {
    conditions: Conditions,
    damage_groups: DamageGroups,
}

impl FromStr for ConditionRecord {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (conditions, damage_groups) = s
            .split_once(' ')
            .ok_or_else(|| Error::Parse("no space".into()))?;
        let conditions = Unsep::<Condition>::from_str(conditions)
            .map_err(|err| Error::Parse(err.to_string()))?
            .0
            .as_slice()
            .try_into()
            .map_err(|err| Error::Parse(format!("converting conditions to array vec: {err}")))?;
        let damage_groups = Vec::from(
            CommaSep::<u8>::from_str(damage_groups).map_err(|err| Error::Parse(err.to_string()))?,
        )
        .as_slice()
        .try_into()
        .map_err(|err| Error::Parse(format!("converting damage groups to array vec: {err}")))?;

        Ok(Self {
            conditions,
            damage_groups,
        })
    }
}

impl ConditionRecord {
    fn n_unknown_bits(&self) -> usize {
        self.conditions
            .iter()
            .copied()
            .filter(Condition::is_unknown)
            .count()
    }

    /// Get the complete set of mappings to check.
    fn possible_mappings(&self) -> impl Iterator<Item = Word> {
        let unknown_bits = self.n_unknown_bits();
        let max_value = 1 << unknown_bits;
        0..max_value
    }

    /// Apply the `mapping` value to the unknown bits of `self.conditions`, producing a full condition value.
    ///
    /// Note that this proceeds from the least significant bits of the mapping and output. This produces values
    /// whose bits are in the opposite order (big-endian) from the naive display of the `conditions` vector.
    fn apply_mapping(&self, mapping: Word) -> Word {
        let mut mapping_idx = 0;
        let mut output = 0;

        for (output_idx, condition) in self.conditions.iter().rev().copied().enumerate() {
            // dbg!(mapping_idx, output_idx, condition);
            match condition {
                Condition::Operational => {
                    // noop; this is a 0 in the output
                    // (only noop because we know it is initialized to 0)
                }
                Condition::Damaged => {
                    // set a 1 bit in the output
                    // output |= 1 << output_idx;
                    output = set_bit(output, output_idx, true)
                }
                Condition::Unknown => {
                    // if the appropriate bit of the mapping is set, set a 1 bit in the output
                    if get_bit(mapping, mapping_idx) {
                        output = set_bit(output, output_idx, true)
                    }
                    mapping_idx += 1;
                }
            }
            // eprintln!("output:  {output:032b}");
        }

        // {
        //     let mut display_conditions = ArrayVec::<u8, 32>::new();
        //     for condition in self.conditions.iter() {
        //         display_conditions.push(match condition {
        //             Condition::Operational => b'0',
        //             Condition::Damaged => b'1',
        //             Condition::Unknown => b'?',
        //         });
        //     }
        //     let display_conditions = str::from_utf8(&display_conditions).unwrap();
        //     eprintln!("display: {display_conditions:>32}");
        //     eprintln!("output:  {output:032b} ({output:>4}) <- mapping {mapping:016b} ({mapping})");
        // }

        output
    }

    fn valid_mappings(&self) -> impl '_ + Iterator<Item = Word> {
        self.possible_mappings()
            .map(|mapping| self.apply_mapping(mapping))
            .filter(|&word| damage_groups(word) == self.damage_groups)
    }
}

#[derive(Debug, thiserror::Error)]
enum ParseUnspError<E> {
    #[error("parsing an instance")]
    Instance(#[source] E),
    #[error("input did not divide neatly by display width")]
    InputRem,
    #[error("input did not divide on a utf8 character boundary")]
    ChunkDivision(#[from] std::str::Utf8Error),
}

// too low: 4264
pub fn part1(input: &Path) -> Result<(), Error> {
    let records = parse::<ConditionRecord>(input)?.collect::<Vec<_>>();

    let sum_of_valid_mappings = records
        .into_iter()
        .map(|record| record.valid_mappings().count())
        .sum::<usize>();
    println!("sum of valid mappings (pt 1): {sum_of_valid_mappings}");
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    unimplemented!("input file: {:?}", input)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Parse: {0}")]
    Parse(String),
    #[error("no solution found")]
    NoSolution,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("???.### 1,1,3", 1)]
    #[case(".??..??...?##. 1,1,3", 4)]
    #[case("?#?#?#?#?#?#?#? 1,3,1,6", 1)]
    #[case("????.#...#... 4,1,1", 1)]
    #[case("????.######..#####. 1,6,5", 4)]
    #[case("?###???????? 3,2,1", 10)]
    fn example_pt1(#[case] condition_record: &str, #[case] expect: usize) {
        dbg!(condition_record);
        let condition_record = condition_record.parse::<ConditionRecord>().unwrap();
        let mappings = condition_record.valid_mappings().count();
        assert_eq!(mappings, expect);
    }
}
