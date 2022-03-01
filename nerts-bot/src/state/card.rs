use crate::{messages::card::CardMessage, position::Position};

#[derive(Debug, Clone)]
pub struct Card {
    pub data: Option<CardData>,
    pub position: Position,
    pub face_up: bool,
    pub height: u8,
    pub holder_index: Option<usize>,
}

impl Card {
    pub fn from_message(message: &CardMessage) -> Self {
        let data = if message.data == 0 {
            None
        } else {
            let suit = if message.data < 13 {
                Suit::Clubs
            } else if message.data < 26 {
                Suit::Diamonds
            } else if message.data < 39 {
                Suit::Hearts
            } else {
                Suit::Spades
            };
            let value = Value::from_code(message.data % 13);
            Some(CardData { suit, value })
        };
        Self {
            data,
            position: Position::new(message.x, message.y),
            face_up: message.flags & 0x1 != 0,
            height: message.height,
            holder_index: if message.holder == 255 {
                None
            } else {
                Some(message.holder as usize)
            },
        }
    }

    /// Returns true if this card can be played on another one
    ///
    /// A card can be played on another if they are both face up, the opposite colors, and this card
    /// is one value lower than the other, ace low.
    pub fn can_play_on(&self, other: &Card) -> bool {
        if self.data.is_none() || other.data.is_none() || !self.face_up || !other.face_up {
            return false;
        }
        let self_data = self.data.as_ref().unwrap();
        let other_data = other.data.as_ref().unwrap();
        self_data.suit == other_data.suit && self_data.value.as_u8() == other_data.value.as_u8() + 1
    }

    pub fn as_small_string(&self) -> String {
        if let Some(data) = self.data.as_ref() {
            format!("{}{}", data.value.as_small_str(), data.suit.as_small_str())
        } else {
            "?".to_string()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CardData {
    pub suit: Suit,
    pub value: Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

impl Suit {
    pub fn as_small_str(&self) -> &'static str {
        match self {
            Suit::Clubs => "C",
            Suit::Diamonds => "D",
            Suit::Hearts => "H",
            Suit::Spades => "S",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Value {
    Ace,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
}

impl Value {
    pub fn from_code(code: u8) -> Value {
        match code % 13 {
            0 => Value::Ace,
            1 => Value::Two,
            2 => Value::Three,
            3 => Value::Four,
            4 => Value::Five,
            5 => Value::Six,
            6 => Value::Seven,
            7 => Value::Eight,
            8 => Value::Nine,
            9 => Value::Ten,
            10 => Value::Jack,
            11 => Value::Queen,
            12 => Value::King,
            _ => unreachable!(),
        }
    }

    pub fn as_u8(&self) -> u8 {
        match self {
            Value::Ace => 0,
            Value::Two => 1,
            Value::Three => 2,
            Value::Four => 3,
            Value::Five => 4,
            Value::Six => 5,
            Value::Seven => 6,
            Value::Eight => 7,
            Value::Nine => 8,
            Value::Ten => 9,
            Value::Jack => 10,
            Value::Queen => 11,
            Value::King => 12,
        }
    }

    pub fn as_small_str(&self) -> &'static str {
        match self {
            Value::Ace => "A",
            Value::Two => "2",
            Value::Three => "3",
            Value::Four => "4",
            Value::Five => "5",
            Value::Six => "6",
            Value::Seven => "7",
            Value::Eight => "8",
            Value::Nine => "9",
            Value::Ten => "⒑",
            Value::Jack => "J",
            Value::Queen => "Q",
            Value::King => "K",
        }
    }
}
