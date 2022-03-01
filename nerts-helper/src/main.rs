use std::{io::Write, time::Duration};

use flexi_logger::Logger;
use log::info;

use nerts_bot::{
    lobbyinfo::LobbyInfo,
    messages::server::GamePhase,
    position::Position,
    state::{
        card::{Card, Suit, Value},
        GameState,
    },
    Bot,
};
use rand::prelude::*;
use tokio::time::Instant;

#[tokio::main]
async fn main() {
    // Start logger
    Logger::try_with_env_or_str("debug")
        .unwrap()
        .start()
        .unwrap();

    // Create rng
    let mut rng = ThreadRng::default();

    // Create bot
    let bot_handle = Bot::start().await.unwrap();

    // println!("Fetching lobbies...");
    // for lobby in bot.lobbies().await.unwrap().iter() {
    //     match lobby {
    //         LobbyInfo::SteamLobby(id) => print!("Public {} ", id.raw()),
    //         LobbyInfo::FriendLobby(id, lobby_id) => {
    //             print!("Friend {} {} ", id.steamid32(), lobby_id.raw())
    //         }
    //     }
    //     let max_string = lobby
    //         .member_limit(&bot)
    //         .map_or_else(|| "N/A".to_string(), |a| a.to_string());
    //     println!("{}/{}", lobby.member_count(&bot), max_string);
    // }

    // Find a friends lobby
    info!("Finding friend lobby");
    std::io::stdout().flush().unwrap();
    let friend_lobby = loop {
        let lobbies = bot_handle.lock().await.lobbies().await.unwrap();
        let lobby = lobbies
            .into_iter()
            .find(|l| matches!(l, LobbyInfo::FriendLobby(_, _)));
        if let Some(lobby) = lobby {
            break lobby;
        }
        // Sleep if unsuccessful
        tokio::time::sleep(Duration::from_millis(500)).await;
    };

    // Join the lobby
    info!("Joining lobby {:?}", friend_lobby.lobby_id());
    bot_handle
        .lock()
        .await
        .join_lobby(friend_lobby)
        .await
        .unwrap();

    // Main loop
    // Can get stuck as sometimes the wait_until conditions don't happen and there are no timeouts
    // Every loop the bot tries to do one of the following, starting from the top:
    //   - If not in game, click ready and wait for game to start
    //   - Call nerts
    //   - If holding a card play it if possible, drop it if not
    //   - Find a card that can be played and pick it up
    //   - If there's an empty space on the table play a card from the nerts pile on it
    //   - If nothing else, draw from the deck
    bot_handle.lock().await.state.send_make_ready = true;
    bot_handle.wait_until(|bot| bot.state.initialized).await;
    let mut last = Instant::now();
    'main: loop {
        // Get a lock on the bot
        let mut bot = bot_handle.lock().await;

        // Draw the game to console if it's been more than a second since the last time
        // Won't happen while stuck in a wait_until so could be improved
        let curr_instant = Instant::now();
        if curr_instant.duration_since(last).as_millis() > 1000 {
            last = curr_instant;
            draw_game(&bot.state);
        }

        // *shrugs*
        if bot.state.players.is_empty() {
            drop(bot);
            bot_handle
                .wait_until(|bot| !bot.state.players.is_empty())
                .await;
            continue;
        }

        match bot.state.game_phase {
            GamePhase::Lobby | GamePhase::Nerts => {
                // Send ready if not ready
                info!("Waiting for game to start");
                if !bot.state.bot_player().ready {
                    bot.state.send_make_ready = true;
                    bot.send_client_message().await;
                }
                drop(bot);
                // Wait until playing
                bot_handle
                    .wait_until(|bot| bot.state.game_phase == GamePhase::Play)
                    .await;
            }
            GamePhase::Intro => {
                // Send ready if not ready
                // Might have mixed up the phases slightly but it works so
                info!("Waiting for intro to end");
                if !bot.state.bot_player().ready {
                    bot.state.send_make_ready = true;
                    bot.send_client_message().await;
                }
                drop(bot);
                // Wait until playing
                bot_handle
                    .wait_until(|bot| bot.state.game_phase == GamePhase::Play)
                    .await;
            }
            GamePhase::Play => {
                let bot_player = bot.state.bot_player();

                // Call nerts if possible
                if bot_player.can_call_nerts {
                    info!("Calling nerts");
                    bot.state.send_make_ready = true;
                    bot.send_client_message().await;
                    drop(bot);
                    bot_handle
                        .wait_until(|bot| !bot.state.bot_player().can_call_nerts)
                        .await;
                    continue;
                }

                // If holding card either play it or drop it
                if !bot_player.held_cards.cards.is_empty() {
                    // Should never be holding a stack
                    assert!(bot_player.held_cards.cards.len() == 1);
                    let held_card = bot_player.held_cards.cards.first().unwrap();

                    // Check if we can play on any of the center cards
                    let play_on = bot.state.center_cards.iter().find(|(_, c)| match c {
                        Some(c) => held_card.can_play_on(c),
                        None => held_card.data.as_ref().unwrap().value == Value::Ace,
                    });
                    if let Some((pos, card)) = play_on {
                        // Play card
                        let card_str = match card {
                            Some(card) => card.as_small_string(),
                            None => "_".to_string(),
                        };
                        info!("Playing {} on {}", held_card.as_small_string(), card_str);
                        let new_pos =
                            *pos + Position::new(rng.gen_range(10..50), rng.gen_range(10..80));
                        assert_ne!(bot.state.target_cursor_pos, new_pos);
                        bot.state.target_cursor_pos = new_pos;
                        bot.state.send_left_click = true;
                        bot.send_client_message().await;

                        // Wait until mouse moved
                        drop(bot);
                        bot_handle
                            .wait_until(|bot| bot.state.bot_player().cursor == new_pos)
                            .await;
                        continue;
                    } else {
                        // Drop card
                        info!(
                            "Couldn't play {}. Dropping card",
                            held_card.as_small_string()
                        );
                        bot.state.send_right_click = true;
                        bot.send_client_message().await;

                        // Wait until not holding card
                        drop(bot);
                        bot_handle
                            .wait_until(|bot| bot.state.bot_player().held_cards.cards.is_empty())
                            .await;
                        continue;
                    }
                }

                // Find a card we can play and pick it up
                let mut playable = Vec::new();
                if !bot_player.nerts_cards.is_empty() {
                    playable.push(bot_player.nerts_cards.first().unwrap());
                }
                for stack in bot_player.table.iter() {
                    if !stack.cards.is_empty() {
                        playable.push(stack.cards.first().unwrap());
                    }
                }
                if bot_player.draw_pile_up.is_some() {
                    playable.push(bot_player.draw_pile_up.as_ref().unwrap());
                }
                for card in playable {
                    if !card.face_up || card.data.is_none() {
                        continue;
                    }
                    let play_on = bot.state.center_cards.iter().find(|(_, c)| match c {
                        Some(c) => card.can_play_on(c),
                        None => card.data.as_ref().unwrap().value == Value::Ace,
                    });
                    if play_on.is_none() {
                        continue;
                    }
                    let (_, play_on) = play_on.unwrap();
                    let play_on_str = match play_on {
                        Some(c) => c.as_small_string(),
                        None => "_".to_string(),
                    };
                    info!(
                        "Picking up {} to play on {}",
                        card.as_small_string(),
                        play_on_str,
                    );

                    let new_pos =
                        card.position + Position::new(rng.gen_range(10..50), rng.gen_range(10..80));
                    assert_ne!(bot.state.target_cursor_pos, new_pos);
                    bot.state.target_cursor_pos = new_pos;
                    bot.state.send_left_click = true;
                    bot.send_client_message().await;

                    // Wait until mouse moved
                    drop(bot);
                    bot_handle
                        .wait_until(|bot| bot.state.bot_player().cursor == new_pos)
                        .await;
                    continue 'main;
                }

                // If possible, play from nerts pile onto table
                if !bot_player.nerts_cards.is_empty() {
                    let empty_table = bot_player
                        .table
                        .iter()
                        .enumerate()
                        .find(|(_, s)| s.cards.is_empty());
                    if let Some((table_index, _)) = empty_table {
                        info!("Playing nerts card onto empty table space");
                        let table_pos = bot_player.table_base_positions()[table_index];
                        let nerts_card_pos = bot_player.nerts_cards.first().unwrap().position;

                        let new_pos = nerts_card_pos
                            + Position::new(rng.gen_range(10..50), rng.gen_range(10..80));
                        assert_ne!(bot.state.target_cursor_pos, new_pos);
                        bot.state.target_cursor_pos = new_pos;
                        bot.state.send_left_click = true;
                        bot.send_client_message().await;

                        // Wait until mouse moved
                        drop(bot);
                        bot_handle
                            .wait_until(|bot| bot.state.bot_player().cursor == new_pos)
                            .await;

                        let mut bot = bot_handle.lock().await;
                        let new_pos =
                            table_pos + Position::new(rng.gen_range(10..50), rng.gen_range(10..80));
                        assert_ne!(bot.state.target_cursor_pos, new_pos);
                        bot.state.target_cursor_pos = new_pos;
                        bot.state.send_left_click = true;
                        bot.send_client_message().await;

                        // Wait until mouse moved
                        drop(bot);
                        bot_handle
                            .wait_until(|bot| bot.state.bot_player().cursor == new_pos)
                            .await;

                        continue;
                    }
                }

                // If we can do nothing else, draw
                // Also randomise card color and back and mouse position for the hell of it
                let new_pos = Position::new(rng.gen_range(200..3800), rng.gen_range(200..2200));
                bot.state.send_draw = true;
                bot.state.target_card_back = rng.gen_range(0..12);
                bot.state.target_card_color = rng.gen_range(0..12);
                bot.state.target_cursor_pos = new_pos;
                bot.send_client_message().await;
                drop(bot);
                bot_handle
                    .wait_until(|bot| {
                        bot.state.bot_player().cursor == new_pos
                            || bot.state.bot_player().can_call_nerts
                    })
                    .await;
            }
        }
    }
}

fn draw_game(state: &GameState) {
    println!();
    println!();
    let center_string = state
        .center_cards
        .iter()
        .map(|(_, c)| match c {
            Some(c) => draw_card(c),
            None => "__".to_string(),
        })
        .collect::<Vec<_>>()
        .join(" ");
    println!("              Center: {}", center_string);
    println!();
    for player in state.players.iter().filter(|p| p.playing) {
        let nerts_card_str = match player.nerts_cards.first() {
            Some(c) => draw_card(c),
            None => "__".to_string(),
        };
        let table_str = player
            .table
            .iter()
            .map(|s| match s.cards.first() {
                Some(c) => draw_card(c),
                None => "__".to_string(),
            })
            .collect::<Vec<_>>()
            .join(" ");
        println!(
            "{:<17} N: {:2} {}  T: {}",
            player.steam_id.raw(),
            player.nerts_cards.len(),
            nerts_card_str,
            table_str,
        );
    }
}

/// Returns a terminal colored string representing a given card
fn draw_card(card: &Card) -> String {
    if card.data.is_none() || !card.face_up {
        return "\x1b[47m\x1b[31m▒▒\x1b[0m".to_string();
    }
    let data = card.data.as_ref().unwrap();
    let suit_part = match data.suit {
        Suit::Spades => "\x1b[30m♠",
        Suit::Hearts => "\x1b[31m♥",
        Suit::Diamonds => "\x1b[30m♦",
        Suit::Clubs => "\x1b[31m♣",
    };
    format!(
        "{}{}{}{}{}",
        BG_WHITE,
        FG_BLACK,
        data.value.as_small_str(),
        suit_part,
        RESET
    )
}

// const FG_RED: &str = "\x1b[31m";
const FG_BLACK: &str = "\x1b[30m";
const BG_WHITE: &str = "\x1b[47m";
const RESET: &str = "\x1b[0m";

#[cfg(test)]
mod tests {
    use nerts_bot::state::card::CardData;

    use super::*;

    #[test]
    fn test_draw_card() {
        println!(
            "{}",
            draw_card(&Card {
                position: Position::new(0, 0),
                data: Some(CardData {
                    value: Value::King,
                    suit: Suit::Hearts,
                }),
                face_up: true,
                height: 0,
                holder_index: None,
            })
        );
    }
}
