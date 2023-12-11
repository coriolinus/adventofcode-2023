use aoclib::parse;
use counter::Counter;
use std::{cmp::Reverse, path::Path, str::FromStr};

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
enum CardPt1 {
    #[display("A")]
    Ace,
    #[display("K")]
    King,
    #[display("Q")]
    Queen,
    #[display("J")]
    Jack,
    #[display("T")]
    Ten,
    #[display("9")]
    Nine,
    #[display("8")]
    Eight,
    #[display("7")]
    Seven,
    #[display("6")]
    Six,
    #[display("5")]
    Five,
    #[display("4")]
    Four,
    #[display("3")]
    Three,
    #[display("2")]
    Two,
}

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
enum CardPt2 {
    #[display("A")]
    Ace,
    #[display("K")]
    King,
    #[display("Q")]
    Queen,
    #[display("T")]
    Ten,
    #[display("9")]
    Nine,
    #[display("8")]
    Eight,
    #[display("7")]
    Seven,
    #[display("6")]
    Six,
    #[display("5")]
    Five,
    #[display("4")]
    Four,
    #[display("3")]
    Three,
    #[display("2")]
    Two,
    #[display("J")]
    Joker,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum HandType {
    FiveOfAKind,
    FourOfAKind,
    FullHouse,
    ThreeOfAKind,
    TwoPair,
    OnePair,
    HighCard,
}

impl From<[CardPt1; 5]> for HandType {
    fn from(cards: [CardPt1; 5]) -> Self {
        let counter = cards.into_iter().collect::<Counter<_>>();
        debug_assert_eq!(counter.values().sum::<usize>(), 5);
        let frequencies = counter.most_common_ordered();
        match frequencies.as_slice() {
            [(_, 5)] => Self::FiveOfAKind,
            [(_, 4), ..] => Self::FourOfAKind,
            [(_, 3), (_, 2)] => Self::FullHouse,
            [(_, 3), ..] => Self::ThreeOfAKind,
            [(_, 2), (_, 2), ..] => Self::TwoPair,
            [(_, 2), ..] => Self::OnePair,
            _ => Self::HighCard,
        }
    }
}

impl From<[CardPt2; 5]> for HandType {
    fn from(cards: [CardPt2; 5]) -> Self {
        let mut counter = cards.into_iter().collect::<Counter<_>>();
        debug_assert_eq!(counter.values().sum::<usize>(), 5);
        let joker_count = counter.remove(&CardPt2::Joker).unwrap_or_default();
        let mut frequencies = counter.most_common_ordered();
        if let Some((_, count)) = frequencies.get_mut(0) {
            *count += joker_count;
        } else {
            frequencies.push((CardPt2::Joker, joker_count));
        }
        match frequencies.as_slice() {
            [(_, 5)] => Self::FiveOfAKind,
            [(_, 4), ..] => Self::FourOfAKind,
            [(_, 3), (_, 2)] => Self::FullHouse,
            [(_, 3), ..] => Self::ThreeOfAKind,
            [(_, 2), (_, 2), ..] => Self::TwoPair,
            [(_, 2), ..] => Self::OnePair,
            _ => Self::HighCard,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Hand<Card> {
    type_: HandType,
    cards: [Card; 5],
}

impl<Card> Hand<Card>
where
    Card: std::fmt::Debug + Copy,
    [Card; 5]: Into<HandType>,
{
    fn new(cards: impl IntoIterator<Item = Card>) -> Result<Self, Error> {
        let cards: [Card; 5] = cards
            .into_iter()
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|err| Error::Parse(format!("wrong length: {err:?}")))?;
        let type_ = cards.into();
        Ok(Hand { type_, cards })
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct HandWithBid<Card> {
    hand: Hand<Card>,
    bid: u64,
}

impl<Card> FromStr for HandWithBid<Card>
where
    Card: std::fmt::Debug + FromStr + Copy,
    Error: From<<Card as FromStr>::Err>,
    [Card; 5]: Into<HandType>,
{
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((hand, bid)) = s.split_once(' ') else {
            return Err(Error::Parse("no space in hand with bid".into()));
        };
        let cards = (0..hand.len())
            .map(|idx| {
                let s = &hand[idx..idx + 1];
                s.parse::<Card>()
            })
            .collect::<Result<Vec<_>, _>>()?;
        let hand = Hand::new(cards)?;

        let bid = bid
            .parse()
            .map_err(|err| Error::Parse(format!("parsing bid: {err}")))?;

        Ok(Self { hand, bid })
    }
}

fn compute_total_winnings<Card>(input: &Path, part: u8) -> Result<(), Error>
where
    Card: std::fmt::Debug + FromStr + Copy + Ord,
    Error: From<<Card as FromStr>::Err>,
    [Card; 5]: Into<HandType>,
{
    let mut hand_bids = parse::<HandWithBid<Card>>(input)?.collect::<Vec<_>>();
    hand_bids.sort_by_key(|hand_bid| Reverse(hand_bid.hand));
    let total_winnings = hand_bids
        .iter()
        .enumerate()
        .map(|(idx, HandWithBid { bid, .. })| {
            let rank = idx + 1;
            rank as u64 * *bid
        })
        .sum::<u64>();
    println!("total winnings (pt {part}): {total_winnings}");
    Ok(())
}

pub fn part1(input: &Path) -> Result<(), Error> {
    compute_total_winnings::<CardPt1>(input, 1)
}

pub fn part2(input: &Path) -> Result<(), Error> {
    compute_total_winnings::<CardPt2>(input, 2)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("no solution found")]
    NoSolution,
}

impl From<parse_display::ParseError> for Error {
    fn from(value: parse_display::ParseError) -> Self {
        Self::Parse(value.to_string())
    }
}
