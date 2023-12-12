use aoclib::{geometry::tile::DisplayWidth, parse, CommaSep};
use std::{
    path::Path,
    str::{self, FromStr},
};

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
    conditions: Vec<Condition>,
    damage_groups: Vec<u8>,
}

impl FromStr for ConditionRecord {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (conditions, damage_groups) = s
            .split_once(' ')
            .ok_or_else(|| Error::Parse("no space".into()))?;
        let conditions = Unsep::<Condition>::from_str(conditions)
            .map_err(|err| Error::Parse(err.to_string()))?
            .0;
        let damage_groups = CommaSep::<u8>::from_str(damage_groups)
            .map_err(|err| Error::Parse(err.to_string()))?
            .into();
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

pub fn part1(input: &Path) -> Result<(), Error> {
    let records = parse::<ConditionRecord>(input)?.collect::<Vec<_>>();
    let max_unknown_bytes = records
        .iter()
        .map(ConditionRecord::n_unknown_bits)
        .max()
        .ok_or(Error::NoSolution)?;
    println!("max unknown bytes in a condition: {max_unknown_bytes}");
    todo!()
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
