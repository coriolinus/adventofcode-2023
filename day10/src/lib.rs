use std::path::Path;

use aoclib::geometry::{tile::DisplayWidth, Direction, Point};

type Map = aoclib::geometry::Map<Tile>;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, parse_display::FromStr, parse_display::Display, strum::EnumIs,
)]
enum Tile {
    #[display("|")]
    Vertical,
    #[display("-")]
    Horizontal,
    L,
    J,
    #[display("7")]
    Seven,
    F,
    #[display(".")]
    Ground,
    #[display("S")]
    Start,
}

impl Tile {
    fn trace(self, direction: Direction) -> Option<Direction> {
        match (self, direction) {
            (Self::Vertical, Direction::Down) => Some(Direction::Up),
            (Self::Vertical, Direction::Up) => Some(Direction::Down),
            (Self::Horizontal, Direction::Left) => Some(Direction::Right),
            (Self::Horizontal, Direction::Right) => Some(Direction::Left),
            (Self::L, Direction::Up) => Some(Direction::Right),
            (Self::L, Direction::Right) => Some(Direction::Up),
            (Self::J, Direction::Up) => Some(Direction::Left),
            (Self::J, Direction::Left) => Some(Direction::Up),
            (Self::Seven, Direction::Left) => Some(Direction::Down),
            (Self::Seven, Direction::Down) => Some(Direction::Left),
            (Self::F, Direction::Down) => Some(Direction::Right),
            (Self::F, Direction::Right) => Some(Direction::Down),
            _ => None,
        }
    }
}

impl DisplayWidth for Tile {
    const DISPLAY_WIDTH: usize = 1;
}

#[derive(Debug, thiserror::Error)]
enum TraceError {
    #[error("path led out-of-bounds at {0:?}")]
    OutOfBounds(Point),
    #[error("disjoint piping at {0:?}")]
    DisjointPiping(Point),
}

/// Trace a path through the pipes.
///
/// Starts with the initial position; can get traced distance with `.enumerate()`.
///
/// Terminates under three circumstances:
///
/// - further progress would exit the bounds of the map
/// - disjoint piping: next position does not have an input where we just came from
/// - next position == initial position; we can get the circuit size by `count()`ing this iterator
fn trace_path(
    map: &Map,
    initial: Point,
    mut direction: Direction,
) -> impl '_ + Iterator<Item = Result<Point, TraceError>> {
    let mut position = initial + direction;

    std::iter::once(Ok(initial)).chain(std::iter::from_fn(move || {
        if position == initial {
            // successful termination
            return None;
        }

        if !map.in_bounds(position) {
            return Some(Err(TraceError::OutOfBounds(position)));
        }

        let to_return = position;

        // we trace from the reverse of our current direction, as from the point of the new position, that's where we came from
        direction = match map[position].trace(direction.reverse()) {
            Some(direction) => direction,
            None => return Some(Err(TraceError::DisjointPiping(position))),
        };
        position += direction;

        Some(Ok(to_return))
    }))
}

/// Find the path length of a successful complete traversal of a path.
///
/// `None` if any trace error occurs.
fn path_length(map: &Map, initial: Point, direction: Direction) -> Option<usize> {
    let mut steps = 0;
    for (s, maybe_position) in trace_path(map, initial, direction).enumerate() {
        maybe_position.ok()?;
        steps = s;
    }
    Some(steps)
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let map = <Map as TryFrom<&Path>>::try_from(input)?;

    let start_points = map
        .iter()
        .filter_map(|(position, tile)| tile.is_start().then_some(position))
        .collect::<Vec<_>>();
    let [start] = start_points.as_slice() else {
        eprintln!("could not determine unique start point");
        return Err(Error::NoSolution);
    };

    // A valid start position must have exactly two connections... but we don't know which two those are.
    // So just try all of them.
    let Some(path_len) =
        Direction::iter().find_map(|direction| path_length(&map, *start, direction))
    else {
        eprintln!("no valid circuit found from start point");
        return Err(Error::NoSolution);
    };

    if path_len % 2 != 1 {
        eprintln!("no unique farthest point determinable");
        return Err(Error::NoSolution);
    }

    let steps_to_farthest = (path_len + 1) / 2;
    println!("steps to farthest (pt 1): {steps_to_farthest}");
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    unimplemented!("input file: {:?}", input)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("no solution found")]
    NoSolution,
}
