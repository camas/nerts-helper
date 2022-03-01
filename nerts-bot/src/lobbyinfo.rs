use steamworks::{LobbyId, SteamId};

use crate::Bot;

#[derive(Debug, Clone, Copy)]
pub enum LobbyInfo {
    SteamLobby(LobbyId),
    FriendLobby(SteamId, LobbyId),
}

impl LobbyInfo {
    pub fn lobby_id(&self) -> LobbyId {
        match self {
            LobbyInfo::SteamLobby(id) => *id,
            LobbyInfo::FriendLobby(_, id) => *id,
        }
    }

    pub fn member_count(&self, bot: &Bot) -> usize {
        bot.client.matchmaking().lobby_member_count(self.lobby_id())
    }

    pub fn member_limit(&self, bot: &Bot) -> Option<usize> {
        bot.client.matchmaking().lobby_member_limit(self.lobby_id())
    }
}
