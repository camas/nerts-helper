use steamworks::SteamId;

use crate::{
    messages::server::{GamePhase, ServerMessage},
    position::Position,
};

use self::{card::Card, player::Player};

pub mod card;
pub mod player;
pub mod stack;

pub const ORIGIN_Y: i16 = 238;
pub const ORIGIN_Y_FLIPPED: i16 = 1382;
pub const STACKED_CARDS_Y_OFFSET: i16 = 32;

#[derive(Debug, Clone)]
pub struct GameState {
    /// Set true when the first parsable ServerMessage has been received
    pub initialized: bool,
    pub game_phase: GamePhase,
    pub players: Vec<Player>,
    pub center_cards: Vec<(Position, Option<Card>)>,
    bot_player_index: usize,
    bot_steam_id: SteamId,
    pub target_cursor_pos: Position,
    pub send_left_click: bool,
    pub send_right_click: bool,
    pub send_make_ready: bool,
    pub send_draw: bool,
    pub target_card_back: u8,
    pub target_card_color: u8,
    pub send_key_frame: bool,
}

impl GameState {
    pub fn new(steam_id: SteamId) -> Self {
        Self {
            initialized: false,
            game_phase: GamePhase::Lobby,
            players: Vec::new(),
            center_cards: Vec::new(),
            bot_player_index: 0,
            bot_steam_id: steam_id,
            target_cursor_pos: Position::zero(),
            send_left_click: false,
            send_right_click: false,
            send_make_ready: false,
            send_draw: false,
            target_card_back: 0,
            target_card_color: 0,
            send_key_frame: false,
        }
    }

    pub fn update(&mut self, server_message: &ServerMessage) {
        if !self.initialized {
            self.initialized = true;
        }
        self.game_phase = server_message.game_phase;
        if self.game_phase != GamePhase::Play {
            return;
        }
        self.center_cards = server_message
            .card_outline_messages
            .iter()
            .map(|m| (Position::new(m.x, m.y), None))
            .collect();
        self.center_cards.sort_by_key(|(p, _)| p.x);
        self.players = server_message
            .player_messages
            .iter()
            .map(Player::from_message)
            .collect();
        self.bot_player_index = self
            .players
            .iter()
            .position(|p| p.steam_id == self.bot_steam_id)
            .unwrap();

        // If nobody playing skip
        if self.players.iter().all(|p| !p.playing) {
            return;
        }

        let cards = server_message.card_messages.iter().map(Card::from_message);
        for card in cards {
            // Check if centre card
            let center = self
                .center_cards
                .iter_mut()
                .find(|(p, _)| *p == card.position);
            if let Some((_, center)) = center {
                assert!(center.is_none());
                *center = Some(card);
                continue;
            }

            // Must be a player card
            let (_, owner) = self
                .players
                .iter_mut()
                .filter(|p| p.playing)
                .enumerate()
                .find(|(i, p)| p.owns_card(&card, *i))
                .unwrap();
            owner.add_card(card);
        }

        // Sort stacked cards
        // Should always have top card at front of vec
        for player in self.players.iter_mut() {
            player.nerts_cards.sort_by_key(|c| c.position.x);
            if !player.flipped {
                player.nerts_cards.reverse();
            }
            player.held_cards.sort(player.flipped);
            for stack in player.table.iter_mut() {
                stack.sort(player.flipped);
            }
        }

        self.validate()
    }

    pub fn validate(&self) {
        // Odd number players should be flipped
        for (i, player) in self.players.iter().filter(|p| p.playing).enumerate() {
            assert_eq!(player.flipped, i % 2 == 1);
            if player.flipped {
                assert!(player.origin.y == ORIGIN_Y_FLIPPED);
            } else {
                assert!(player.origin.y == ORIGIN_Y);
            }
        }

        // Player origins should be correct
        assert!(self
            .players
            .iter()
            .filter(|p| p.playing)
            .all(|p| if p.flipped {
                p.origin.y == ORIGIN_Y_FLIPPED
            } else {
                p.origin.y == ORIGIN_Y
            }));

        // All playing players should have the right table size
        // Should be equal or higher as players might have left
        let expected_table_size = match self.number_playing() {
            1..=2 => 6,
            3 => 5,
            _ => 4,
        };
        assert!(self
            .players
            .iter()
            .filter(|p| p.playing)
            .all(|p| p.table.len() >= expected_table_size));

        // Check all played stacks are valid
        for player in self.players.iter() {
            player.held_cards.validate();
            for table_stack in player.table.iter() {
                table_stack.validate()
            }
        }

        // Check right number of centre positions
        // Can be greater again if players leave
        let expected_centre_size = self.number_playing() * 4;
        assert!(self.center_cards.len() >= expected_centre_size);
    }

    pub fn bot_player(&self) -> &Player {
        &self.players[self.bot_player_index]
    }

    pub fn number_playing(&self) -> usize {
        self.players.iter().filter(|p| p.playing).count()
    }
}

#[cfg(test)]
mod tests {
    use crate::messages::{
        card::CardMessage, cardoutline::CardOutlineMessage, player::PlayerMessage,
    };

    use super::*;

    #[test]
    fn test_parse_known() {
        let mut state = GameState::new(SteamId::from_raw(76561191240930714));
        let message = ServerMessage {
            game_phase: GamePhase::Play,
            player_messages: vec![
                PlayerMessage {
                    player_id: 76561198064411451,
                    origin_x: 554,
                    origin_y: 238,
                    flipped: false,
                    is_playing: true,
                    is_ready: false,
                    can_call_nerts: false,
                    show_deck_button: false,
                    effects: 0,
                    card_color: 6,
                    tableau_count: 5,
                    called_nerts: false,
                    nerts_cards: 13,
                    holding_nerts_card: false,
                    points_cards: 0,
                    total_score: 18,
                    history_points: vec![18],
                    history_nertsed: vec![true],
                    ignore_disable_foundation: false,
                    cursor_x: 3838,
                    cursor_y: 1086,
                },
                PlayerMessage {
                    player_id: 76561199244422576,
                    origin_x: 1256,
                    origin_y: 1382,
                    flipped: true,
                    is_playing: true,
                    is_ready: false,
                    can_call_nerts: false,
                    show_deck_button: false,
                    effects: 0,
                    card_color: 8,
                    tableau_count: 5,
                    called_nerts: false,
                    nerts_cards: 13,
                    holding_nerts_card: false,
                    points_cards: 0,
                    total_score: 22,
                    history_points: vec![22],
                    history_nertsed: vec![false],
                    ignore_disable_foundation: false,
                    cursor_x: 629,
                    cursor_y: 1432,
                },
                PlayerMessage {
                    player_id: 76561198040136714,
                    origin_x: 1958,
                    origin_y: 238,
                    flipped: false,
                    is_playing: true,
                    is_ready: false,
                    can_call_nerts: false,
                    show_deck_button: false,
                    effects: 0,
                    card_color: 3,
                    tableau_count: 5,
                    called_nerts: false,
                    nerts_cards: 13,
                    holding_nerts_card: false,
                    points_cards: 0,
                    total_score: 0,
                    history_points: vec![0],
                    history_nertsed: vec![false],
                    ignore_disable_foundation: false,
                    cursor_x: 1908,
                    cursor_y: 508,
                },
            ],
            card_messages: vec![
                CardMessage {
                    x: 636,
                    y: 378,
                    data: 6,
                    flags: 0,
                    height: 28,
                    holder: 255,
                },
                CardMessage {
                    x: 638,
                    y: 642,
                    data: 6,
                    flags: 4,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 651,
                    y: 642,
                    data: 6,
                    flags: 4,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 665,
                    y: 642,
                    data: 6,
                    flags: 4,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 678,
                    y: 642,
                    data: 6,
                    flags: 4,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 692,
                    y: 642,
                    data: 6,
                    flags: 4,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 705,
                    y: 642,
                    data: 6,
                    flags: 4,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 719,
                    y: 642,
                    data: 6,
                    flags: 4,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 732,
                    y: 642,
                    data: 6,
                    flags: 4,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 746,
                    y: 642,
                    data: 6,
                    flags: 4,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 759,
                    y: 642,
                    data: 6,
                    flags: 4,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 773,
                    y: 642,
                    data: 6,
                    flags: 4,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 786,
                    y: 642,
                    data: 6,
                    flags: 4,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 800,
                    y: 642,
                    data: 22,
                    flags: 5,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 1014,
                    y: 642,
                    data: 3,
                    flags: 1,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 1174,
                    y: 642,
                    data: 51,
                    flags: 1,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 1334,
                    y: 642,
                    data: 10,
                    flags: 1,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 1494,
                    y: 642,
                    data: 30,
                    flags: 1,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 1654,
                    y: 642,
                    data: 48,
                    flags: 1,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 2356,
                    y: 1828,
                    data: 8,
                    flags: 2,
                    height: 28,
                    holder: 255,
                },
                CardMessage {
                    x: 2354,
                    y: 1564,
                    data: 8,
                    flags: 6,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 2340,
                    y: 1564,
                    data: 8,
                    flags: 6,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 2327,
                    y: 1564,
                    data: 8,
                    flags: 6,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 2313,
                    y: 1564,
                    data: 8,
                    flags: 6,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 2300,
                    y: 1564,
                    data: 8,
                    flags: 6,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 2286,
                    y: 1564,
                    data: 8,
                    flags: 6,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 2273,
                    y: 1564,
                    data: 8,
                    flags: 6,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 2259,
                    y: 1564,
                    data: 8,
                    flags: 6,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 2246,
                    y: 1564,
                    data: 8,
                    flags: 6,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 2232,
                    y: 1564,
                    data: 8,
                    flags: 6,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 2219,
                    y: 1564,
                    data: 8,
                    flags: 6,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 2205,
                    y: 1564,
                    data: 8,
                    flags: 6,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 2192,
                    y: 1564,
                    data: 3,
                    flags: 7,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 1980,
                    y: 1564,
                    data: 49,
                    flags: 3,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 1820,
                    y: 1564,
                    data: 23,
                    flags: 3,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 1660,
                    y: 1564,
                    data: 13,
                    flags: 3,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 1500,
                    y: 1564,
                    data: 38,
                    flags: 3,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 1340,
                    y: 1564,
                    data: 30,
                    flags: 3,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 2040,
                    y: 378,
                    data: 3,
                    flags: 0,
                    height: 28,
                    holder: 255,
                },
                CardMessage {
                    x: 2042,
                    y: 642,
                    data: 3,
                    flags: 4,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 2055,
                    y: 642,
                    data: 3,
                    flags: 4,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 2069,
                    y: 642,
                    data: 3,
                    flags: 4,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 2082,
                    y: 642,
                    data: 3,
                    flags: 4,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 2096,
                    y: 642,
                    data: 3,
                    flags: 4,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 2109,
                    y: 642,
                    data: 3,
                    flags: 4,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 2123,
                    y: 642,
                    data: 3,
                    flags: 4,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 2136,
                    y: 642,
                    data: 3,
                    flags: 4,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 2150,
                    y: 642,
                    data: 3,
                    flags: 4,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 2163,
                    y: 642,
                    data: 3,
                    flags: 4,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 2177,
                    y: 642,
                    data: 3,
                    flags: 4,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 2190,
                    y: 642,
                    data: 3,
                    flags: 4,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 2204,
                    y: 642,
                    data: 40,
                    flags: 5,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 2418,
                    y: 642,
                    data: 45,
                    flags: 1,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 2578,
                    y: 642,
                    data: 1,
                    flags: 1,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 2738,
                    y: 642,
                    data: 48,
                    flags: 1,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 2898,
                    y: 642,
                    data: 13,
                    flags: 1,
                    height: 0,
                    holder: 255,
                },
                CardMessage {
                    x: 3058,
                    y: 642,
                    data: 29,
                    flags: 1,
                    height: 0,
                    holder: 255,
                },
            ],
            card_outline_messages: vec![
                CardOutlineMessage { x: 967, y: 1102 },
                CardOutlineMessage { x: 1127, y: 1102 },
                CardOutlineMessage { x: 1287, y: 1102 },
                CardOutlineMessage { x: 1447, y: 1102 },
                CardOutlineMessage { x: 1607, y: 1102 },
                CardOutlineMessage { x: 1767, y: 1102 },
                CardOutlineMessage { x: 1927, y: 1102 },
                CardOutlineMessage { x: 2087, y: 1102 },
                CardOutlineMessage { x: 2247, y: 1102 },
                CardOutlineMessage { x: 2407, y: 1102 },
                CardOutlineMessage { x: 2567, y: 1102 },
                CardOutlineMessage { x: 2727, y: 1102 },
            ],
            notification_message: None,
            emergency_shuffle_countdown: None,
            shuffle_count: 0,
        };
        state.update(&message);
    }
}
