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
enum Card {
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

impl From<[Card; 5]> for HandType {
    fn from(cards: [Card; 5]) -> Self {
        let counter = cards.into_iter().collect::<Counter<_>>();
        debug_assert_eq!(counter.values().sum::<usize>(), 5);
        let frequencies = counter.most_common_ordered();
        match frequencies.as_slice() {
            &[(_, 5)] => Self::FiveOfAKind,
            &[(_, 4), ..] => Self::FourOfAKind,
            &[(_, 3), (_, 2)] => Self::FullHouse,
            &[(_, 3), ..] => Self::ThreeOfAKind,
            &[(_, 2), (_, 2), ..] => Self::TwoPair,
            &[(_, 2), ..] => Self::OnePair,
            _ => Self::HighCard,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Hand {
    type_: HandType,
    cards: [Card; 5],
}

impl Hand {
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
struct HandWithBid {
    hand: Hand,
    bid: u64,
}

impl FromStr for HandWithBid {
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

pub fn part1(input: &Path) -> Result<(), Error> {
    let mut hand_bids = parse::<HandWithBid>(input)?.collect::<Vec<_>>();
    hand_bids.sort_by_key(|hand_bid| Reverse(hand_bid.hand));
    let total_winnings = hand_bids
        .iter()
        .enumerate()
        .map(|(idx, HandWithBid { bid, .. })| {
            let rank = idx + 1;
            rank as u64 * *bid
        })
        .sum::<u64>();
    println!("total winnings (pt 1): {total_winnings}");
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    todo!()
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
