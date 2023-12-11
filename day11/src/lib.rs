use aoclib::geometry::{Direction, Point};
use itertools::Itertools;
use std::path::Path;

type Image = aoclib::geometry::map::Map<aoclib::geometry::map::tile::Bool>;

#[derive(Debug)]
struct ExpandingSpace {
    image: Image,
    doubled_rows: Vec<i32>,
    doubled_columns: Vec<i32>,
}

impl ExpandingSpace {
    fn parse(input: &Path) -> Result<Self, Error> {
        let image = <Image as TryFrom<&Path>>::try_from(input)?;
        let doubled_rows = image
            .edge(Direction::Left)
            .enumerate()
            .filter_map(|(idx, point)| {
                let (dx, dy) = Direction::Right.deltas();
                image
                    .project(point, dx, dy)
                    .all(|point| !bool::from(image[point]))
                    .then_some(idx as i32)
            })
            .collect();
        let doubled_columns = image
            .edge(Direction::Down)
            .enumerate()
            .filter_map(|(idx, point)| {
                let (dx, dy) = Direction::Up.deltas();
                image
                    .project(point, dx, dy)
                    .all(|point| !bool::from(image[point]))
                    .then_some(idx as i32)
            })
            .collect();

        Ok(Self {
            image,
            doubled_rows,
            doubled_columns,
        })
    }

    fn expanded_distance_between(&self, a: Point, b: Point) -> i32 {
        debug_assert!(
            bool::from(self.image[a]),
            "it is only interesting to count distances betweeen galaxies"
        );
        debug_assert!(
            bool::from(self.image[b]),
            "it is only interesting to count distances betweeen galaxies"
        );

        let row_range = {
            let lower = a.y.min(b.y);
            let upper = a.y.max(b.y);
            lower..upper
        };
        let col_range = {
            let lower = a.x.min(b.x);
            let upper = a.x.max(b.x);
            lower..upper
        };
        (b - a).manhattan()
            + self
                .doubled_rows
                .iter()
                .filter(|&row| row_range.contains(row))
                .count() as i32
            + self
                .doubled_columns
                .iter()
                .filter(|&col| col_range.contains(col))
                .count() as i32
    }
}

// too high: 20627195
pub fn part1(input: &Path) -> Result<(), Error> {
    let es = ExpandingSpace::parse(input)?;
    let galaxies = es
        .image
        .iter()
        .filter_map(|(point, &is_galaxy)| bool::from(is_galaxy).then_some(point))
        .collect::<Vec<_>>();
    let sum_of_dists = galaxies
        .iter()
        .cartesian_product(galaxies.iter())
        .filter(|(a, b)| a < b)
        .map(|(&a, &b)| es.expanded_distance_between(a, b))
        .sum::<i32>();
    println!("sum of dists (pt 1): {sum_of_dists}");
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

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use std::io::Write as _;
    use tempfile::NamedTempFile;

    const EXAMPLE: &str = "
...#......
.......#..
#.........
..........
......#...
.#........
.........#
..........
.......#..
#...#.....
";

    #[rstest]
    #[case("5 -> 9", Point::new(1, 4), Point::new(4, 0), 9)]
    #[case("1 -> 7", Point::new(3, 9), Point::new(7, 1), 15)]
    #[case("3 -> 6", Point::new(0, 7), Point::new(9, 3), 17)]
    #[case("8 -> 9", Point::new(0, 0), Point::new(4, 0), 5)]
    fn example_pt1(#[case] name: &str, #[case] a: Point, #[case] b: Point, #[case] expect: i32) {
        eprintln!("case {name}");
        let mut tempfile = NamedTempFile::new().unwrap();
        write!(tempfile.as_file_mut(), "{}", EXAMPLE.trim_start()).unwrap();
        let es = ExpandingSpace::parse(tempfile.path()).unwrap();
        tempfile.close().unwrap();

        assert_eq!(es.expanded_distance_between(a, b), expect);
    }
}
