use std::path::Path;

use aoclib::geometry::{tile::DisplayWidth, Direction, Point};
use strum::IntoEnumIterator as _;

type Map = aoclib::geometry::Map<Tile>;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    parse_display::FromStr,
    parse_display::Display,
    strum::EnumIs,
    strum::EnumIter,
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
) -> impl '_ + Iterator<Item = Result<(Point, Direction), TraceError>> {
    let mut position = initial + direction;

    std::iter::once(Ok((initial, direction))).chain(std::iter::from_fn(move || {
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

        Some(Ok((to_return, direction)))
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

/// Replace the start point with an appropriate tile.
///
/// This is a destructive edit; ensure you have retained the start point elsewhere.
///
/// Returns `true` when it has successfully completed a traversal and replaced the start point.
fn replace_start_tile(map: &mut Map, initial: Point, direction: Direction) -> bool {
    let mut last_item = (initial, direction);
    for maybe_position in trace_path(map, initial, direction) {
        let Ok(item) = maybe_position else {
            return false;
        };
        last_item = item;
    }
    let (_last, last_direction) = last_item;
    // we need the tile type which starts from the (opposite of!) final direction and produces the initial
    let tile = Tile::iter()
        .find(|tile| tile.trace(last_direction.reverse()) == Some(direction))
        .expect("there exists a tile type which works here");
    map[initial] = tile;
    true
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumIs, parse_display::Display)]
enum TileStyle {
    #[display(".")]
    Unknown,
    #[display("#")]
    MainLoop,
    #[display("I")]
    Inside,
    #[display("O")]
    Outside,
}

impl DisplayWidth for TileStyle {
    const DISPLAY_WIDTH: usize = 1;
}

impl From<Tile> for TileStyle {
    fn from(value: Tile) -> Self {
        match value {
            Tile::Start => Self::MainLoop,
            _ => Self::Unknown,
        }
    }
}

type StyleMap = aoclib::geometry::Map<TileStyle>;

/// There's a well-known algorithm for determining whether an arbitrary point is
/// inside or outside a path: project a line in any arbitrary direction. If it
/// crosses the path an odd number of times, it's inside; otherwise, it's
/// outside.
fn flood_inside(map: &Map, tile_styles: &mut StyleMap) {
    assert_eq!(
        map.bottom_left(),
        tile_styles.bottom_left(),
        "map and tile bottom left coords must agree"
    );
    assert_eq!(
        map.top_right(),
        tile_styles.top_right(),
        "map and tile top right coords must agree"
    );

    // we choose to flood diagonally, to simplify our logic; no need to care about parallelism
    // as we will flood in the up-right direction, our start points must be the left and bottom edges
    // it is unnecessary but aesthetically pleasing to organize our edge traversal contiguously
    let left_edge = {
        let (dx, dy) = Direction::Down.deltas();
        map.project(map.top_left(), dx, dy)
    };
    let bottom_edge = {
        let (dx, dy) = Direction::Right.deltas();
        // start one to the right, to avoid double-counting the bottom-left point
        map.project(map.bottom_left() + Direction::Right, dx, dy)
    };
    let edges = left_edge.chain(bottom_edge);

    let dx = Direction::Right.deltas().0 + Direction::Up.deltas().0;
    let dy = Direction::Right.deltas().1 + Direction::Up.deltas().1;

    for edge_point in edges {
        let mut inside = false;
        for point in map.project(edge_point, dx, dy) {
            if tile_styles[point].is_main_loop() {
                // the complex case: if this is a crossing, invert `inside`. Otherwise, don't.
                debug_assert!(!matches!(map[point], Tile::Ground | Tile::Start));
                let is_crossing = matches!(
                    map[point],
                    Tile::Horizontal | Tile::Vertical | Tile::Seven | Tile::L
                );
                if is_crossing {
                    inside = !inside;
                }
            } else {
                // the simple case: just assign according to the current side status
                // nothing here changes the status
                debug_assert!(tile_styles[point].is_unknown());
                tile_styles[point] = if inside {
                    TileStyle::Inside
                } else {
                    TileStyle::Outside
                };
            }
        }
    }
}

pub fn part2(input: &Path) -> Result<(), Error> {
    let mut map = <Map as TryFrom<&Path>>::try_from(input)?;

    let start_points = map
        .iter()
        .filter_map(|(position, tile)| tile.is_start().then_some(position))
        .collect::<Vec<_>>();
    let [start] = start_points.as_slice() else {
        eprintln!("could not determine unique start point");
        return Err(Error::NoSolution);
    };

    // determine initial loop direction
    let Some(initial_direction) =
        Direction::iter().find(|&direction| replace_start_tile(&mut map, *start, direction))
    else {
        eprintln!("no valid circuit found from start point");
        return Err(Error::NoSolution);
    };

    let mut tile_styles = map.clone().convert_tile_type::<TileStyle>();
    for item in trace_path(&map, *start, initial_direction) {
        let point = item.expect("we had a valid trace of this loop earlier").0;
        tile_styles[point] = TileStyle::MainLoop;
    }

    flood_inside(&map, &mut tile_styles);

    debug_assert!(!tile_styles.iter().any(|(_point, tile)| tile.is_unknown()));

    let enclosed = tile_styles
        .iter()
        .filter(|(_point, tile)| tile.is_inside())
        .count();
    println!("n enclosed tiles (pt 2): {enclosed}");
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("no solution found")]
    NoSolution,
}
