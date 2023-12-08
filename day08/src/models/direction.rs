#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    parse_display::Display,
    parse_display::FromStr,
)]
pub enum Direction {
    #[display("L")]
    Left,
    #[display("R")]
    Right,
}

/// produce an infinite, owned list of directions
pub fn directions_iter(list: &[Direction]) -> impl '_ + Iterator<Item = Direction> {
    list.iter().copied().cycle()
}
