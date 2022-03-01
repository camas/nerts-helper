use std::{sync::Arc, time::Duration};

use compression::decompress;
use error::BotError;
use lobbyinfo::LobbyInfo;
use log::{debug, trace};
use messages::{
    client::ClientMessage,
    io::{reader::MessageReader, writer::MessageWriter},
    server::ServerMessage,
};
use state::GameState;
use steamworks::{CallbackHandle, FriendFlags, P2PSessionRequest, SteamId};
use tokio::sync::{broadcast, mpsc, oneshot, Mutex, MutexGuard};

pub mod compression;
mod error;
pub mod lobbyinfo;
pub mod messages;
pub mod position;
pub mod state;

type Result<T> = std::result::Result<T, BotError>;

const APP_ID: u32 = 1131190;
const TO_CLIENT_CHANNEL: i32 = 1;
const TO_SERVER_CHANNEL: i32 = 2;

#[derive(Clone)]
pub struct BotHandle {
    bot: Arc<Mutex<Bot>>,
}

impl BotHandle {
    pub async fn lock(&self) -> MutexGuard<'_, Bot> {
        self.bot.lock().await
    }

    pub async fn wait_until<F>(&self, test: F)
    where
        F: Fn(MutexGuard<'_, Bot>) -> bool,
    {
        tokio::select! {
            _ = async {
                let mut data_received_rx = self.lock().await.data_received_tx.subscribe();
                loop {
                    if test(self.lock().await) {
                        break;
                    }
                    data_received_rx.recv().await.unwrap();
                }
            } => {},
            _ = tokio::time::sleep(Duration::from_millis(5000)) => {},
        }
    }
}

pub struct Bot {
    pub client: steamworks::Client<steamworks::ClientManager>,
    /// Will shut down the callback thread when dropped
    _shutdown_tx: broadcast::Sender<()>,
    lobby: Option<LobbyInfo>,
    server_id: Option<SteamId>,
    last_data: Option<Vec<u8>>,
    send_client_message_tx: mpsc::Sender<()>,
    pub state: GameState,
    _session_req_cb: CallbackHandle,
    data_received_tx: broadcast::Sender<()>,
}

impl Bot {
    pub async fn start() -> Result<BotHandle> {
        // Login with steam
        let (client, single_client) = steamworks::Client::init_app(APP_ID)?;
        let client_ = client.clone();
        let steam_id = client.user().steam_id();
        debug!("Connected to steam as {}", steam_id.raw());

        // Accept all p2p requests
        let _session_req_cb = client.register_callback(move |req: P2PSessionRequest| {
            debug!("Accepted p2p {:?}", req.remote);
            client_.networking().accept_p2p_session(req.remote);
        });

        let (data_received_tx, _) = broadcast::channel(10);

        let (send_client_message_tx, send_client_message_rx) = mpsc::channel::<()>(10);
        let (shutdown_tx, mut shutdown_rx1) = broadcast::channel::<()>(1);
        let mut shutdown_rx2 = shutdown_tx.subscribe();
        let mut shutdown_rx3 = shutdown_tx.subscribe();
        let client_1 = client.clone();
        let client_2 = client.clone();
        let bot = Bot {
            client,
            _shutdown_tx: shutdown_tx,
            lobby: None,
            server_id: None,
            last_data: None,
            send_client_message_tx,
            state: GameState::new(steam_id),
            _session_req_cb,
            data_received_tx,
        };
        let handle = BotHandle {
            bot: Arc::new(Mutex::new(bot)),
        };

        // Steam callback task
        let _ = tokio::task::spawn_blocking(move || loop {
            if matches!(
                shutdown_rx1.try_recv(),
                Err(broadcast::error::TryRecvError::Closed)
            ) {
                break;
            }
            single_client.run_callbacks();
            std::thread::sleep(Duration::from_millis(10));
        });

        // Start send loop
        let handle_ = handle.clone();
        let _ = tokio::spawn(async move {
            tokio::select! {
                _ = shutdown_rx2.recv() => {}
                _ = Bot::send_loop(handle_, client_1, send_client_message_rx) => {}
            }
        });

        // Start receive loop
        let handle_ = handle.clone();
        let _ = tokio::spawn(async move {
            tokio::select! {
                _ = shutdown_rx3.recv() => {}
                _ = Bot::receive_loop(handle_, client_2) => {}
            };
        });

        Ok(handle)
    }

    /// Sends current information frequently or updated information on events like clicking, nerts, etc.
    async fn send_loop(
        bot: BotHandle,
        client: steamworks::Client<steamworks::ClientManager>,
        mut send_client_message_rx: mpsc::Receiver<()>,
    ) -> Result<()> {
        loop {
            // Wait for event or timeout
            tokio::select! {
                _ = send_client_message_rx.recv() => {}
                _ = tokio::time::sleep(Duration::from_millis(100)) => {}
            }

            // Get message and server id from bot
            let (message, server_id) = {
                let mut bot = bot.lock().await;
                (bot.create_client_message(), bot.server_id)
            };

            // If not connected to server, skip
            if server_id.is_none() {
                continue;
            }
            let server_id = server_id.unwrap();

            // Serialize and send
            trace!("Sending {:?}", message);
            let mut w = MessageWriter::new();
            w.write(message);
            let res = client.networking().send_p2p_packet(
                server_id,
                steamworks::SendType::Reliable,
                &w.finish(),
                TO_SERVER_CHANNEL,
            );
            assert!(res);
        }
    }

    /// Receives ServerMessages
    async fn receive_loop(
        bot: BotHandle,
        client: steamworks::Client<steamworks::ClientManager>,
    ) -> Result<()> {
        loop {
            let res = client.networking().is_p2p_packet_available();
            if res.is_none() {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            let mut buf = vec![0u8; 0x1000];
            let res = client
                .networking()
                .read_p2p_packet(&mut buf, TO_CLIENT_CHANNEL);
            if res.is_none() {
                continue;
            }
            let (steam_id, size) = res.unwrap();
            buf.truncate(size);
            let bot_ = bot.clone();
            tokio::spawn(async move {
                bot_.lock().await.handle_packet(steam_id, buf);
            });
        }
    }

    pub async fn lobbies(&self) -> Result<Vec<LobbyInfo>> {
        let mut lobbies = Vec::new();

        // Get public lobbies
        let (result_tx, result_rx) = oneshot::channel();
        self.client.matchmaking().request_lobby_list(|result| {
            result_tx.send(result).unwrap();
        });

        let result = result_rx.await.unwrap()?;
        for id in result {
            lobbies.push(LobbyInfo::SteamLobby(id));
        }

        // Get friend lobbies
        for friend in self.client.friends().get_friends(FriendFlags::IMMEDIATE) {
            if let Some(friend_game) = friend.game_played() {
                if friend_game.game.app_id().0 == APP_ID && friend_game.lobby.raw() != 0 {
                    lobbies.push(LobbyInfo::FriendLobby(friend.id(), friend_game.lobby));
                }
            }
        }

        Ok(lobbies)
    }

    pub async fn join_lobby(&mut self, lobby_info: LobbyInfo) -> Result<()> {
        let lobby_id = lobby_info.lobby_id();

        // Make request with steamworks
        let (result_tx, result_rx) = oneshot::channel();
        self.client.matchmaking().join_lobby(lobby_id, |result| {
            result_tx.send(result).unwrap();
        });

        // Receive and verify result
        let result = result_rx.await.unwrap().map_err(|_| BotError::JoinLobby)?;
        assert_eq!(result, lobby_id);

        self.lobby = Some(lobby_info);
        self.server_id = Some(
            self.client
                .matchmaking()
                .lobby_game_server(lobby_info.lobby_id())
                .unwrap()
                .steam_id
                .unwrap(),
        );

        // Ask for keyframe and send first message
        self.state.send_key_frame = true;
        self.send_client_message_tx.send(()).await.unwrap();

        Ok(())
    }

    pub fn steam_id(&self) -> SteamId {
        self.client.user().steam_id()
    }

    /// Tells the bot to send a ClientMessage immediately
    ///
    /// The send loop will still need a lock on the bot to create a new message before it can send it.
    pub async fn send_client_message(&self) {
        self.send_client_message_tx.send(()).await.unwrap();
    }

    fn handle_packet(&mut self, steam_id: SteamId, data: Vec<u8>) {
        if Some(steam_id) != self.server_id {
            return;
        }
        let data = decompress(&data);
        let new_data;
        match data[0] {
            0 => new_data = data[1..].to_vec(),
            1 => {
                if self.last_data.is_none() {
                    return;
                }
                let last_data = self.last_data.as_ref().unwrap();
                if last_data.len() != data.len() - 1 {
                    unreachable!()
                }
                new_data = data[1..]
                    .iter()
                    .zip(last_data.iter())
                    .map(|(a, b)| a.wrapping_add(*b))
                    .collect();
            }
            _ => unreachable!(),
        }
        self.last_data = Some(new_data.clone());
        let mut r = MessageReader::new(&new_data);
        let message = r.read::<ServerMessage>();
        assert!(r.remaining_len() == 0);
        trace!("Received {:?}", message);
        self.state.update(&message);
        let _ = self.data_received_tx.send(());
    }

    fn create_client_message(&mut self) -> ClientMessage {
        let message = ClientMessage {
            x: self.state.target_cursor_pos.x,
            y: self.state.target_cursor_pos.y,
            left_click: self.state.send_left_click,
            right_click: self.state.send_right_click,
            make_ready: self.state.send_make_ready,
            draw: self.state.send_draw,
            card_back: self.state.target_card_back,
            card_color: self.state.target_card_color,
            send_key_frame: self.state.send_key_frame,
        };
        self.state.send_left_click = false;
        self.state.send_right_click = false;
        self.state.send_make_ready = false;
        self.state.send_draw = false;
        self.state.send_key_frame = false;
        message
    }
}
