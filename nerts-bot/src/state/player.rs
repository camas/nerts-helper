use log::error;
use steamworks::SteamId;

use crate::{messages::player::PlayerMessage, position::Position};

use super::{card::Card, stack::PlayedStack};

#[derive(Debug, Clone)]
pub struct Player {
    pub cursor: Position,
    pub steam_id: SteamId,
    pub playing: bool,
    pub ready: bool,
    pub origin: Position,
    pub flipped: bool,
    pub nerts_cards: Vec<Card>,
    pub draw_pile_down: Option<Card>,
    pub draw_pile_up: Option<Card>,
    pub table: Vec<PlayedStack>,
    pub held_cards: PlayedStack,
    pub can_call_nerts: bool,
    pub called_nerts: bool,
}

impl Player {
    /// Create a Player object from a PlayerMessage
    pub fn from_message(message: &PlayerMessage) -> Self {
        Self {
            cursor: Position::new(message.cursor_x, message.cursor_y),
            steam_id: SteamId::from_raw(message.player_id),
            playing: message.is_playing,
            ready: message.is_ready,
            origin: Position::new(message.origin_x, message.origin_y),
            flipped: message.flipped,
            nerts_cards: Vec::new(),
            draw_pile_down: None,
            draw_pile_up: None,
            table: (0..message.tableau_count)
                .map(|_| PlayedStack::default())
                .collect(),
            held_cards: PlayedStack::default(),
            can_call_nerts: message.can_call_nerts,
            called_nerts: message.called_nerts,
        }
    }

    pub fn owns_card(&self, card: &Card, player_index: usize) -> bool {
        // Bounds are just roughly taken from putting mouse in top right corner
        card.holder_index == Some(player_index)
            || card
                .position
                .within_box(self.origin, Position::new(1040 + self.extra_x(), 700))
    }

    pub fn add_card(&mut self, card: Card) {
        if card.holder_index.is_some() {
            self.held_cards.add_card(card);
            return;
        }

        if card.position == self.draw_pile_down_pos() {
            assert!(self.draw_pile_down.is_none());
            self.draw_pile_down = Some(card);
            return;
        }

        if card.position == self.draw_pile_up_pos() {
            assert!(self.draw_pile_up.is_none());
            self.draw_pile_up = Some(card);
            return;
        }

        let nerts_last_card_pos = self.nerts_last_card_pos();
        let within_x = if self.flipped {
            card.position.x >= nerts_last_card_pos.x - 170
                && card.position.x <= nerts_last_card_pos.x
        } else {
            card.position.x >= nerts_last_card_pos.x
                && card.position.x <= nerts_last_card_pos.x + 170
        };
        if within_x && card.position.y == nerts_last_card_pos.y {
            self.nerts_cards.push(card);
            return;
        }

        // Not in the rest so must be on the table
        let table_base_positions = self.table_base_positions();
        let result = if !self.flipped {
            table_base_positions.into_iter().enumerate().find(|(_, p)| {
                card.position
                    .within_box(*p - Position::new(0, 600), Position::new(1, 700))
            })
        } else {
            table_base_positions.into_iter().enumerate().find(|(_, p)| {
                card.position
                    .within_box(*p - Position::new(0, 10), Position::new(1, 700))
            })
        };
        match result {
            Some((stack_i, _)) => self.table[stack_i].add_card(card),
            None => {
                error!("Error matching card to position!");
                error!("Card: {:#?}", card);
                error!("Player: {:#?}", self);
                error!(
                    "If this was a face down draw pile then it was expected to be at {:?}",
                    self.draw_pile_down_pos()
                );
                error!("Face up expected at {:?}", self.draw_pile_up_pos());
                panic!();
            }
        }
    }

    /// Returns the extra size of the table based on the number of table spaces
    fn extra_x(&self) -> i16 {
        match self.table.len() {
            6 => 320,
            5 => 160,
            _ => 0,
        }
    }

    fn draw_pile_down_pos(&self) -> Position {
        if !self.flipped {
            self.origin + Position::new(82, 140)
        } else {
            self.origin + Position::new(940 + self.extra_x(), 446)
        }
    }

    fn draw_pile_up_pos(&self) -> Position {
        if !self.flipped {
            self.origin + Position::new(248, 140)
        } else {
            self.origin + Position::new(774 + self.extra_x(), 446)
        }
    }

    fn nerts_last_card_pos(&self) -> Position {
        if !self.flipped {
            self.origin + Position::new(84, 404)
        } else {
            self.origin + Position::new(938 + self.extra_x(), 182)
        }
    }

    pub fn table_base_positions(&self) -> Vec<Position> {
        let first_position = if !self.flipped {
            self.origin + Position::new(460, 404)
        } else {
            self.origin + Position::new(564 + self.extra_x(), 182)
        };

        let mut positions = vec![first_position];
        for i in 1..(self.table.len() as i16) {
            if !self.flipped {
                positions.push(first_position + Position::new(160 * i, 0));
            } else {
                positions.push(first_position + Position::new(-160 * i, 0));
            }
        }

        positions
    }
}
