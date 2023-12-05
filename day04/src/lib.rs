use aoclib::parse;
use std::{path::Path, str::FromStr};

#[derive(Debug, Default)]
struct Card {
    n: u32,
    winning: Vec<u8>,
    have: Vec<u8>,
    points: u32,
}

impl FromStr for Card {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut card = Card::default();

        let Some((card_n, rest)) = s.split_once(':') else {
            return Err(Error::Parse("no colon".into()));
        };
        const CARD: &str = "Card ";
        if !card_n.starts_with(CARD) {
            return Err(Error::Parse("invalid prefix".into()));
        }
        card.n = card_n[CARD.len()..]
            .trim_start()
            .parse()
            .map_err(|_err| Error::Parse("unparseable n".into()))?;

        let (winning, have) = rest.split_once('|').ok_or(Error::Parse("no pipe".into()))?;
        card.winning = winning
            .split_ascii_whitespace()
            .map(|token| token.parse::<u8>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_err| Error::Parse("converting winning values to ints".into()))?;

        card.have = have
            .split_ascii_whitespace()
            .map(|token| token.parse::<u8>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_err| Error::Parse("converting have values to ints".into()))?;

        card.compute_points();
        Ok(card)
    }
}

impl Card {
    fn compute_points(&mut self) {
        self.points = 0;
        for winner in &self.winning {
            if self.have.contains(winner) {
                if self.points > 0 {
                    self.points *= 2;
                } else {
                    self.points = 1;
                }
            }
        }
    }
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let points = parse::<Card>(input)?.map(|card| card.points).sum::<u32>();
    println!("total points (pt 1): {points}");
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    unimplemented!("input file: {:?}", input)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("invalid card: {0}")]
    Parse(String),
    #[error("no solution found")]
    NoSolution,
}
