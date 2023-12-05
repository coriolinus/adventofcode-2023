use aoclib::geometry::{
    map::{tile::DisplayWidth, Map},
    Direction, MapConversionErr, Point,
};
use std::{fmt, path::Path, str::FromStr};

#[derive(Clone, Copy, strum::EnumIs)]
enum Tile {
    #[strum(to_string = ".")]
    Empty,
    // an ascii digit; part of a number
    Digit(u8),
    // a non-digit symbol
    Symbol(char),
}

impl DisplayWidth for Tile {
    const DISPLAY_WIDTH: usize = 1;
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tile::Empty => f.write_str("."),
            Tile::Digit(d) => write!(f, "{d}"),
            Tile::Symbol(s) => write!(f, "{s}"),
        }
    }
}

impl FromStr for Tile {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 1 {
            return Err(Error::ParseTile("tile must have width of 1".into()));
        }
        let c = s.chars().next().unwrap();
        match c {
            '.' => Ok(Tile::Empty),
            _ if c.is_ascii_digit() => Ok(Tile::Digit(s.parse().unwrap())),
            _ => Ok(Tile::Symbol(c)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Number {
    left: Point,
    right: Point,
    value: u32,
}

impl Number {
    fn from_points(map: &Map<Tile>, left: Point, right: Point) -> Option<Self> {
        if left.y != right.y {
            return None;
        }

        let mut value = 0;
        let (dx, dy) = Direction::Left.deltas();
        for (power, point) in map
            .project(right, dx, dy)
            .enumerate()
            .take((right.x - left.x + 1) as _)
        {
            let Tile::Digit(digit) = map[point] else {
                return None;
            };
            value += digit as u32 * 10_u32.pow(power as _);
        }

        Some(Number { left, right, value })
    }

    fn find(map: &Map<Tile>) -> impl '_ + Iterator<Item = Number> {
        let mut start = None;
        let mut end = None;
        let mut current = map.top_left();

        std::iter::from_fn(move || {
            loop {
                // terminal check
                if !map.in_bounds(current) {
                    return None;
                }

                // remember if we found a number
                let mut number = None;

                // scan for a number
                if map[current].is_digit() {
                    if start.is_none() {
                        start = Some(current);
                    }
                    end = Some(current);
                } else if let Some((left, right)) = start.take().zip(end.take()) {
                    number = Self::from_points(map, left, right);
                }

                // advance the current position
                current += Direction::Right;
                if !map.in_bounds(current) {
                    current.x = 0;
                    current += Direction::Down;

                    // we might have had a trailing number
                    if let Some((left, right)) = start.take().zip(end.take()) {
                        number = Self::from_points(map, left, right);
                    }
                }

                if number.is_some() {
                    return number;
                }
            }
        })
    }

    fn adjacent(&self, map: &Map<Tile>) -> impl '_ + Iterator<Item = Point> {
        let width = (1 + self.right.x - self.left.x) as usize;

        let top = {
            let (dx, dy) = Direction::Right.deltas();
            map.project(self.left + Direction::Up, dx, dy)
                .take(width + 1)
        };
        let right = {
            let (dx, dy) = Direction::Down.deltas();
            map.project(self.right + Direction::Right, dx, dy).take(2)
        };
        let bottom = {
            let (dx, dy) = Direction::Left.deltas();
            map.project(self.right + Direction::Down, dx, dy)
                .take(width + 1)
        };
        let left = {
            let (dx, dy) = Direction::Up.deltas();
            map.project(self.left + Direction::Left, dx, dy).take(2)
        };

        top.chain(right).chain(bottom).chain(left)
    }

    fn is_part_number(&self, map: &Map<Tile>) -> bool {
        self.adjacent(map).any(|point| map[point].is_symbol())
    }
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let map = <Map<Tile> as TryFrom<_>>::try_from(input)?;

    let sum_of_part_numbers = Number::find(&map)
        .filter_map(|number| number.is_part_number(&map).then_some(number.value))
        .sum::<u32>();

    println!("sum of part numbers (pt 1): {sum_of_part_numbers}");
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    unimplemented!("input file: {:?}", input)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("parsing tile: {0}")]
    ParseTile(String),
    #[error("failed to read input map")]
    MapConversion(#[from] MapConversionErr),
    #[error("no solution found")]
    NoSolution,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(",.\n.1")]
    #[case(",\n1")]
    #[case(".,\n1.")]
    #[case("1,")]
    #[case("1.\n.,")]
    #[case("1\n,")]
    #[case(".1\n,.")]
    #[case(",1")]
    fn detects_parts_all_directions(#[case] input: &str) {
        let map = <Map<Tile> as TryFrom<_>>::try_from(input).unwrap();
        eprintln!("{map}");
        let numbers = Number::find(&map).collect::<Vec<_>>();
        assert_eq!(numbers.len(), 1);
        assert_eq!(numbers[0].value, 1);
        assert!(numbers[0].is_part_number(&map));
    }
}
