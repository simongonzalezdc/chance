use crate::core::range::uniform_u64_inclusive;
use crate::core::source::Source;
use crate::methods::shuffle::shuffle;

const MAJOR_ARCANA: &[&str] = &[
    "The Fool",
    "The Magician",
    "The High Priestess",
    "The Empress",
    "The Emperor",
    "The Hierophant",
    "The Lovers",
    "The Chariot",
    "Strength",
    "The Hermit",
    "Wheel of Fortune",
    "Justice",
    "The Hanged Man",
    "Death",
    "Temperance",
    "The Devil",
    "The Tower",
    "The Star",
    "The Moon",
    "The Sun",
    "Judgement",
    "The World",
];

const SUIT_NAMES: &[&str] = &["Wands", "Cups", "Swords", "Pentacles"];
const COURT_NAMES: &[&str] = &["Page", "Knight", "Queen", "King"];

#[derive(Debug, Clone)]
pub struct TarotCard {
    pub name: String,
    pub upright: bool,
}

impl std::fmt::Display for TarotCard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({})",
            self.name,
            if self.upright { "upright" } else { "reversed" }
        )
    }
}

fn standard_tarot_deck() -> Vec<TarotCard> {
    let mut deck = Vec::with_capacity(78);
    for &name in MAJOR_ARCANA {
        deck.push(TarotCard {
            name: name.to_string(),
            upright: true,
        });
    }
    for &suit in SUIT_NAMES {
        for n in 1..=10 {
            deck.push(TarotCard {
                name: format!("{} of {}", n, suit),
                upright: true,
            });
        }
        for &court in COURT_NAMES {
            deck.push(TarotCard {
                name: format!("{} of {}", court, suit),
                upright: true,
            });
        }
    }
    deck
}

pub fn draw_tarot(
    source: &mut dyn Source,
    count: usize,
) -> Result<Vec<TarotCard>, crate::core::SourceError> {
    let mut deck = standard_tarot_deck();
    shuffle(source, &mut deck[..])?;

    // Apply upright/reversed orientation.
    for card in deck.iter_mut() {
        card.upright = uniform_u64_inclusive(source, 0, 1)? == 1;
    }

    Ok(deck.into_iter().take(count).collect())
}
