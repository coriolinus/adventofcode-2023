use std::collections::HashMap;

use crate::{input::NodeDefinition, Error};

use super::Direction;

pub const INITIAL_NAME: &str = "AAA";
pub const TARGET_NAME: &str = "ZZZ";

pub struct Network {
    names: HashMap<String, usize>,
    name_for: HashMap<usize, String>,
    left: Vec<usize>,
    right: Vec<usize>,
}

impl TryFrom<Vec<NodeDefinition>> for Network {
    type Error = Error;

    fn try_from(value: Vec<NodeDefinition>) -> Result<Self, Error> {
        let mut names = HashMap::with_capacity(value.len());
        let mut name_for = HashMap::with_capacity(value.len());
        let mut left_name = Vec::with_capacity(value.len());
        let mut right_name = Vec::with_capacity(value.len());

        for NodeDefinition { name, left, right } in value {
            let idx = names.len();
            let ejected = names.insert(name.clone(), idx);
            if let Some(ejected) = ejected {
                return Err(Error::Parse(format!(
                    "duplicate node definition: {name} ({idx} & {ejected})"
                )));
            }
            name_for.insert(idx, name);
            left_name.push(left);
            right_name.push(right);
        }

        let left = left_name
            .iter()
            .map(|left| {
                names.get(left).copied().ok_or_else(|| {
                    Error::Parse(format!("left name \"{left}\" not found in node names"))
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let right = right_name
            .iter()
            .map(|right| {
                names.get(right).copied().ok_or_else(|| {
                    Error::Parse(format!("right name \"{right}\" not found in node names"))
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            names,
            name_for,
            left,
            right,
        })
    }
}

impl Network {
    pub fn step(&self, position: usize, direction: Direction) -> usize {
        match direction {
            Direction::Left => self.left[position],
            Direction::Right => self.right[position],
        }
    }

    pub fn position_of(&self, name: &str) -> Option<usize> {
        self.names.get(name).copied()
    }

    pub fn name_of(&self, position: usize) -> Option<&str> {
        self.name_for.get(&position).map(String::as_str)
    }

    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.names.keys().map(String::as_str)
    }
}
