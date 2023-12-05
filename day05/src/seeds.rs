use std::str::FromStr;

use crate::Error;

pub struct Seeds(pub Vec<i64>);

impl FromStr for Seeds {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s
            .strip_prefix("seeds: ")
            .ok_or_else(|| Error::Parse("no seeds prefix".into()))?;

        let items = s
            .split_ascii_whitespace()
            .map(|token| token.parse())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_err| Error::Parse("interpreting seed value".into()))?;

        Ok(Self(items))
    }
}
