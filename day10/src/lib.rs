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

    fn is_parallel(self, direction: Direction) -> bool {
        matches!(
            (self, direction),
            (Self::Vertical, Direction::Down)
                | (Self::Vertical, Direction::Up)
                | (Self::Horizontal, Direction::Left)
                | (Self::Horizontal, Direction::Right)
        )
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
///
/// We need to modify the algorithm just a little, to exclude tiles which are
/// parallel to our direction of projection, but that's trivial.
fn is_inside(map: &Map, tile_styles: &StyleMap, point: Point) -> bool {
    fn is_inside(
        map: &Map,
        tile_styles: &StyleMap,
        point: Point,
        projection_direction: Direction,
    ) -> bool {
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

        let mut half_open = None;

        let (dx, dy) = projection_direction.deltas();
        let crossing_count = map
            .project(point, dx, dy)
            .filter(|&point| {
                tile_styles[point].is_main_loop() && !map[point].is_parallel(projection_direction)
            })
            .filter(|&point| {
                match (
                    half_open.take(),
                    map[point].trace(projection_direction),
                    map[point].trace(projection_direction.reverse()),
                ) {
                    (None, None, None) => {
                        // we don't have a pending half-opening, and this point does not create a half opening,
                        // so this must be perpendicular, which gives us a straightforward perpendicular crossing
                        true
                    }
                    (None, Some(direction), None) => {
                        // we don't have a pending half-opening, but this point creates a half opening
                        // don't record it yet, but keep track of that half opening
                        half_open = Some(direction);
                        false
                    }
                    (Some(half_open), None, Some(half_close)) => {
                        // we have a pending half opening, and a potential half closing
                        assert!(
                            half_open == half_close || half_open == half_close.reverse(),
                            "mismatched open and close"
                        );
                        // if they are the same, then we don't count this close as a crossing; it backed off.
                        // if they are different, we count this.
                        half_open != half_close
                    }
                    state => {
                        dbg!(state, point, map[point]);
                        unreachable!("invalid state")
                    }
                }
            })
            .count();

        crossing_count % 2 != 0
    }

    #[cfg(not(debug_assertions))]
    {
        is_inside(map, tile_styles, point, Direction::Up)
    }
    #[cfg(debug_assertions)]
    {
        let inside: [bool; 4] = std::array::from_fn(|idx| {
            // we should probably have a function like this in aoclib
            let direction = match idx {
                0 => Direction::Up,
                1 => Direction::Right,
                2 => Direction::Down,
                3 => Direction::Left,
                _ => unreachable!("array constructor will not over-call this fn"),
            };
            is_inside(map, tile_styles, point, direction)
        });
        match inside {
            [true, true, true, true] => true,
            [false, false, false, false] => false,
            _ => {
                let (dx, dy) = Direction::Up.deltas();
                for point in tile_styles.project(point, dx, dy) {
                    if tile_styles[point].is_main_loop() {
                        dbg!(point, map[point]);
                    }
                }
                dbg!(point, inside);
                panic!("projecting in different directions gave differing results")
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

    for point in tile_styles.points() {
        if !tile_styles[point].is_unknown() {
            continue;
        }
        tile_styles[point] = if is_inside(&map, &tile_styles, point) {
            TileStyle::Inside
        } else {
            TileStyle::Outside
        };
    }

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
