use super::io::reader::{Deserialize, MessageReader};

#[derive(Debug)]
pub struct PlayerMessage {
    pub player_id: u64,
    pub origin_x: i16,
    pub origin_y: i16,
    pub flipped: bool,
    pub is_playing: bool,
    pub is_ready: bool,
    pub can_call_nerts: bool,
    pub show_deck_button: bool,
    pub effects: u32,
    pub card_color: u8,
    pub tableau_count: u8,
    pub called_nerts: bool,
    pub nerts_cards: u8,
    pub holding_nerts_card: bool,
    pub points_cards: u8,
    pub total_score: i16,
    pub history_points: Vec<i8>,
    pub history_nertsed: Vec<bool>,
    pub ignore_disable_foundation: bool,
    pub cursor_x: i16,
    pub cursor_y: i16,
}

impl Deserialize for PlayerMessage {
    fn deserialize(r: &mut MessageReader) -> Self {
        PlayerMessage {
            player_id: r.read(),
            origin_x: r.read(),
            origin_y: r.read(),
            flipped: r.read(),
            is_playing: r.read(),
            is_ready: r.read(),
            can_call_nerts: r.read(),
            show_deck_button: r.read(),
            effects: r.read(),
            card_color: r.read(),
            tableau_count: r.read(),
            called_nerts: r.read(),
            nerts_cards: r.read(),
            holding_nerts_card: r.read(),
            points_cards: r.read(),
            total_score: r.read(),
            history_points: r.read(),
            history_nertsed: r.read(),
            ignore_disable_foundation: r.read(),
            cursor_x: r.read(),
            cursor_y: r.read(),
        }
    }
}
