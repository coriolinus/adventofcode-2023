use aoclib::{geometry::tile::DisplayWidth, parse, CommaSep};
use std::{
    path::Path,
    str::{self, FromStr},
};

type Conditions = Vec<Condition>;
type DamageGroups = Vec<u8>;

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
    type Err = ParseUnsepError<<T as FromStr>::Err>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let chunker = s.as_bytes().chunks_exact(T::DISPLAY_WIDTH);
        if !chunker.remainder().is_empty() {
            return Err(ParseUnsepError::InputRem);
        }
        let values = chunker
            .map(|chunk| {
                let s = str::from_utf8(chunk)?;
                T::from_str(s).map_err(ParseUnsepError::Instance)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self(values))
    }
}

#[derive(Debug, thiserror::Error)]
enum ParseUnsepError<E> {
    #[error("parsing an instance")]
    Instance(#[source] E),
    #[error("input did not divide neatly by display width")]
    InputRem,
    #[error("input did not divide on a utf8 character boundary")]
    ChunkDivision(#[from] std::str::Utf8Error),
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

    fn unfold(&mut self) {
        let conditions_len = self.conditions.len();
        let damage_groups_len = self.damage_groups.len();

        for _ in 0..4 {
            self.conditions.push(Condition::Unknown);
            self.conditions.extend_from_within(0..conditions_len);

            self.damage_groups.extend_from_within(0..damage_groups_len);
        }
    }
}

// too low: 4264
pub fn part1(input: &Path) -> Result<(), Error> {
    let records = parse::<ConditionRecord>(input)?.collect::<Vec<_>>();

    let sum_of_valid_mappings = todo!();
    // println!("sum of valid mappings (pt 1): {sum_of_valid_mappings}");
    Ok(())
}

// ah damnit. had an inkling this would happen. But thought I'd do the dumb thing first, in case it helped.
pub fn part2(input: &Path) -> Result<(), Error> {
    let records = parse::<ConditionRecord>(input)?
        .map(|mut record| {
            record.unfold();
            record
        })
        .collect::<Vec<_>>();

    let max_unknown = records
        .iter()
        .map(|record| record.n_unknown_bits())
        .max()
        .unwrap();
    let max_len = records
        .iter()
        .map(|record| record.conditions.len())
        .max()
        .unwrap();

    println!("unknown: {max_unknown} / max: {max_len}");

    let sum_of_valid_mappings = todo!();
    // println!("sum of valid mappings (pt 1): {sum_of_valid_mappings}");
    Ok(())
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
        let mappings = todo!();
        // assert_eq!(mappings, expect);
    }
}
