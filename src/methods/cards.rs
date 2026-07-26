use crate::core::range::uniform_u64_inclusive;
use crate::core::source::Source;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Suit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
}

impl std::fmt::Display for Suit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let symbol = match self {
            Suit::Hearts => "♥",
            Suit::Diamonds => "♦",
            Suit::Clubs => "♣",
            Suit::Spades => "♠",
        };
        write!(f, "{}", symbol)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rank {
    Ace,
    Number(u8),
    Jack,
    Queen,
    King,
}

impl std::fmt::Display for Rank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Rank::Ace => write!(f, "A"),
            Rank::Number(n) => write!(f, "{}", n),
            Rank::Jack => write!(f, "J"),
            Rank::Queen => write!(f, "Q"),
            Rank::King => write!(f, "K"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Card {
    pub suit: Suit,
    pub rank: Rank,
}

impl std::fmt::Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.rank, self.suit)
    }
}

fn standard_deck() -> Vec<Card> {
    let mut deck = Vec::with_capacity(52);
    for suit in [Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades] {
        deck.push(Card {
            suit,
            rank: Rank::Ace,
        });
        for n in 2..=10 {
            deck.push(Card {
                suit,
                rank: Rank::Number(n),
            });
        }
        deck.push(Card {
            suit,
            rank: Rank::Jack,
        });
        deck.push(Card {
            suit,
            rank: Rank::Queen,
        });
        deck.push(Card {
            suit,
            rank: Rank::King,
        });
    }
    deck
}

/// Shuffle a deck using Fisher-Yates with the given source.
pub fn shuffle_deck(
    source: &mut dyn Source,
    deck: &mut [Card],
) -> Result<(), crate::core::SourceError> {
    let n = deck.len();
    for i in (1..n).rev() {
        let j = uniform_u64_inclusive(source, 0, i as u64)? as usize;
        deck.swap(i, j);
    }
    Ok(())
}

/// Draw `count` cards from a fresh 52-card deck.
pub fn draw_cards(
    source: &mut dyn Source,
    count: usize,
) -> Result<Vec<Card>, crate::core::SourceError> {
    let mut deck = standard_deck();
    shuffle_deck(source, &mut deck)?;
    Ok(deck.into_iter().take(count).collect())
}
