use aoclib::parse;
use std::{path::Path, str::FromStr};

#[derive(Debug, Default, PartialEq, Eq)]
struct CubeSet {
    red: u32,
    green: u32,
    blue: u32,
}

impl FromStr for CubeSet {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut red = None;
        let mut green = None;
        let mut blue = None;

        for cube_count in s.split(',').map(str::trim) {
            let Some((count, color)) = cube_count.split_once(' ') else {
                return Err(Error::InvalidInput("no space in cube count".into()));
            };

            let count = count
                .parse()
                .map_err(|err| Error::InvalidInput(format!("parsing cube count: {err}")))?;
            let storage = match color {
                "red" => &mut red,
                "blue" => &mut blue,
                "green" => &mut green,
                _ => return Err(Error::InvalidInput(format!("unknown color \"{color}\""))),
            };

            if storage.is_some() {
                return Err(Error::InvalidInput(format!(
                    "{color} attempted to set twice"
                )));
            }

            *storage = Some(count);
        }

        Ok(Self {
            red: red.unwrap_or_default(),
            green: green.unwrap_or_default(),
            blue: blue.unwrap_or_default(),
        })
    }
}

#[derive(Debug)]
struct Game {
    number: u32,
    draws: Vec<CubeSet>,
}

impl FromStr for Game {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((game_n, draws)) = s.split_once(':') else {
            return Err(Error::InvalidInput("no colon in game line".into()));
        };

        const GAME: &str = "Game ";
        if !game_n.starts_with(GAME) {
            // dangerous and wrong: we shouldn't expect that slicing to `[..GAME.len()]` will work in the arbitrary case
            // it's probably ok for puzzle inputs though.
            return Err(Error::InvalidInput(format!(
                "expected {GAME:?}; found {:?}",
                &s[..GAME.len()]
            )));
        }

        let number = game_n[GAME.len()..]
            .parse()
            .map_err(|err| Error::InvalidInput(format!("parsing game number: {err}")))?;

        let draws = draws
            .split(';')
            .map(CubeSet::from_str)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Game { number, draws })
    }
}

impl Game {
    fn is_possible(&self, bag: &CubeSet) -> bool {
        self.draws
            .iter()
            .all(|set| set.red <= bag.red && set.green <= bag.green && set.blue <= bag.blue)
    }
}

pub fn part1(input: &Path) -> Result<(), Error> {
    const BAG: CubeSet = CubeSet {
        red: 12,
        green: 13,
        blue: 14,
    };

    let id_sum = parse::<Game>(input)?
        .filter_map(|game| game.is_possible(&BAG).then_some(game.number))
        .sum::<u32>();

    println!("sum of ids of valid games (pt 1): {id_sum}");
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    unimplemented!("input file: {:?}", input)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("no solution found")]
    NoSolution,
}
