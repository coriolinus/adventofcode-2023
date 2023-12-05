#[derive(Debug, Clone, Copy, parse_display::FromStr, parse_display::Display)]
#[display("{destination_start} {source_start} {range_length}")]
pub struct MapEntry {
    pub(crate) destination_start: i64,
    pub(crate) source_start: i64,
    pub(crate) range_length: i64,
}

impl MapEntry {
    #[inline]
    #[allow(dead_code)]
    pub(crate) fn destination_end(&self) -> i64 {
        self.destination_start + self.range_length
    }
    #[inline]
    pub(crate) fn source_end(&self) -> i64 {
        self.source_start + self.range_length
    }
    #[inline]
    pub(crate) fn contains(&self, v: i64) -> bool {
        (self.source_start..self.source_end()).contains(&v)
    }
    #[inline]
    pub(crate) fn delta(&self) -> i64 {
        self.destination_start - self.source_start
    }
    #[inline]
    pub fn apply(&self, value: i64) -> i64 {
        if self.contains(value) {
            value + self.delta()
        } else {
            value
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(0, 0)]
    #[case(49, 49)]
    #[case(50, 52)]
    #[case(51, 53)]
    #[case(96, 98)]
    #[case(97, 99)]
    #[case(98, 98)]
    #[case(99, 99)]
    fn pt1_example(#[case] value: i64, #[case] expect: i64) {
        let entry = MapEntry {
            destination_start: 52,
            source_start: 50,
            range_length: 48,
        };
        let got = entry.apply(value);
        assert_eq!(got, expect);
    }
}
