use thiserror::Error;

#[derive(Debug, Error)]
pub enum BotError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Steam(#[from] steamworks::SteamError),

    #[error("Failed to join lobby")]
    JoinLobby,
}
