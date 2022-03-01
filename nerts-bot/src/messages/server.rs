use super::{
    card::CardMessage,
    cardoutline::CardOutlineMessage,
    io::reader::{Deserialize, MessageReader},
    notification::NotificationMessage,
    player::PlayerMessage,
};

#[derive(Debug)]
pub struct ServerMessage {
    pub game_phase: GamePhase,
    pub player_messages: Vec<PlayerMessage>,
    pub card_messages: Vec<CardMessage>,
    pub card_outline_messages: Vec<CardOutlineMessage>,
    pub notification_message: Option<NotificationMessage>,
    pub emergency_shuffle_countdown: Option<u8>,
    pub shuffle_count: u8,
}

impl Deserialize for ServerMessage {
    fn deserialize(r: &mut MessageReader) -> Self {
        ServerMessage {
            game_phase: GamePhase::new(r.read()),
            player_messages: r.read(),
            card_messages: r.read(),
            card_outline_messages: r.read(),
            notification_message: r.read(),
            emergency_shuffle_countdown: r.read(),
            shuffle_count: r.read(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamePhase {
    Lobby,
    Intro,
    Play,
    Nerts,
}

impl GamePhase {
    pub fn new(phase: u8) -> Self {
        match phase {
            0 => GamePhase::Lobby,
            1 => GamePhase::Intro,
            2 => GamePhase::Play,
            3 => GamePhase::Nerts,
            _ => unreachable!("Unknown game phase: {}", phase),
        }
    }
}
