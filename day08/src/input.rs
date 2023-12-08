use aoclib::input::parse_two_phase;

use crate::{
    models::{Direction, Network},
    Error,
};
use std::{path::Path, str::FromStr};

struct Directions(Vec<Direction>);

impl FromStr for Directions {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim_end();
        let mut directions = Vec::with_capacity(s.len());
        let mut buffer = [0; 4];
        for c in s.chars() {
            let c = c.encode_utf8(&mut buffer);
            let d = c.parse().map_err(|err| {
                Error::Parse(format!("attempting to parse direction: \"{c}\": {err}"))
            })?;
            directions.push(d);
        }
        Ok(Self(directions))
    }
}

#[derive(Debug, parse_display::Display, parse_display::FromStr)]
#[display("{name} = ({left}, {right})")]
pub struct NodeDefinition {
    pub name: String,
    pub left: String,
    pub right: String,
}

#[derive(Default)]
struct Nodes(Vec<NodeDefinition>);

impl FromStr for Nodes {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let definitions = s
            .lines()
            .filter(|line| !line.is_empty())
            .map(NodeDefinition::from_str)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| Error::Parse(err.to_string()))?;
        Ok(Self(definitions))
    }
}

pub fn parse(path: impl AsRef<Path>) -> Result<(Vec<Direction>, Network), Error> {
    let (directions, definitions) = parse_two_phase::<Directions, Nodes>(path.as_ref())
        .map_err(|err| Error::Parse(err.to_string()))?;
    let directions = directions.0;
    let definitions = definitions
        .map(|nodes| nodes.0)
        .reduce(|mut left, right| {
            left.extend(right);
            left
        })
        .unwrap_or_default();
    let network = definitions.try_into()?;
    Ok((directions, network))
}
