use aoclib::geometry::{Direction, Point};
use std::path::Path;

type Image = aoclib::geometry::map::Map<aoclib::geometry::map::tile::Bool>;

#[derive(Debug)]
struct ExpandingSpace {
    image: Image,
    doubled_rows: Vec<u64>,
    doubled_columns: Vec<u64>,
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
                    .then_some(idx as u64)
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
                    .then_some(idx as u64)
            })
            .collect();

        Ok(Self {
            image,
            doubled_rows,
            doubled_columns,
        })
    }

    fn expanded_distance_between(&self, a: Point, b: Point, expansion_factor: u64) -> u64 {
        debug_assert!(
            bool::from(self.image[a]),
            "it is only interesting to count distances betweeen galaxies"
        );
        debug_assert!(
            bool::from(self.image[b]),
            "it is only interesting to count distances betweeen galaxies"
        );

        let row_range = {
            let lower = a.y.min(b.y) as u64;
            let upper = a.y.max(b.y) as u64;
            lower..upper
        };
        let col_range = {
            let lower = a.x.min(b.x) as u64;
            let upper = a.x.max(b.x) as u64;
            lower..upper
        };
        let base_distance = (b - a).manhattan() as u64;
        let expansion_factor = expansion_factor - 1;
        let expanded_rows = expansion_factor
            * self
                .doubled_rows
                .iter()
                .filter(|&row| row_range.contains(row))
                .count() as u64;
        let expanded_cols = expansion_factor
            * self
                .doubled_columns
                .iter()
                .filter(|&col| col_range.contains(col))
                .count() as u64;
        base_distance + expanded_rows + expanded_cols
    }

    fn space_between_galaxies(&self, expansion_factor: u64) -> u64 {
        let galaxies = self
            .image
            .iter()
            .filter_map(|(point, &is_galaxy)| bool::from(is_galaxy).then_some(point))
            .collect::<Vec<_>>();
        galaxies
            .iter()
            .enumerate()
            .flat_map(|(idx, &a)| (idx + 1..galaxies.len()).map(move |bidx| (a, bidx)))
            .map(|(a, bidx)| (a, galaxies[bidx]))
            .map(|(a, b)| self.expanded_distance_between(a, b, expansion_factor))
            .sum()
    }
}

// too high: 20627195
pub fn part1(input: &Path) -> Result<(), Error> {
    let es = ExpandingSpace::parse(input)?;
    let space_between = es.space_between_galaxies(2);
    println!("sum of dists (pt 1): {space_between}");
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    let es = ExpandingSpace::parse(input)?;
    let space_between = es.space_between_galaxies(1000000);
    println!("sum of dists (pt 2): {space_between}");
    Ok(())
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

    fn example_input() -> ExpandingSpace {
        let mut tempfile = NamedTempFile::new().unwrap();
        write!(tempfile.as_file_mut(), "{}", EXAMPLE.trim_start()).unwrap();
        let es = ExpandingSpace::parse(tempfile.path()).unwrap();
        tempfile.close().unwrap();
        es
    }

    #[rstest]
    #[case("5 -> 9", Point::new(1, 4), Point::new(4, 0), 9)]
    #[case("1 -> 7", Point::new(3, 9), Point::new(7, 1), 15)]
    #[case("3 -> 6", Point::new(0, 7), Point::new(9, 3), 17)]
    #[case("8 -> 9", Point::new(0, 0), Point::new(4, 0), 5)]
    fn example_pt1(#[case] name: &str, #[case] a: Point, #[case] b: Point, #[case] expect: u64) {
        eprintln!("case {name}");
        let es = example_input();
        assert_eq!(es.expanded_distance_between(a, b, 2), expect);
    }

    #[rstest]
    #[case(10, 1030)]
    #[case(100, 8410)]
    fn example_pt2(#[case] expansion_factor: u64, #[case] expect: u64) {
        let es = example_input();
        let sum_of_dists = es.space_between_galaxies(expansion_factor);
        assert_eq!(sum_of_dists, expect);
    }
}
