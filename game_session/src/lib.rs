#![no_std]
#![allow(warnings)]
use game_session_io::*;
use gstd::{exec, msg, prelude::*, ActorId, MessageId};
use wordle_io::Action as WordleAction;
use wordle_io::Event as WordleEvent;

static mut GAME_SESSION: Option<GameSession> = None;

const MAX_ATTEMPTS: u8 = 6;
const MAX_BLOCKS: u32 = 200;

pub struct GameSession {
    pub wordle_program: ActorId,
    pub status: GameStatus,
    pub msg_ids: Option<(MessageId, MessageId)>,
    pub session_status: SessionStatus,
}

#[derive(Debug, PartialEq)]
pub enum SessionStatus {
    Waiting,
    MessageSent,
    ReplyReceived(WordleEvent),
}

#[no_mangle]
extern "C" fn init() {
    let wordle_program: ActorId = msg::load().expect("Failed to decode program address");
    let game_session = GameSession {
        wordle_program,
        status: GameStatus::None,
        msg_ids: None,
        session_status: SessionStatus::Waiting,
    };
    unsafe { GAME_SESSION = Some(game_session) };
}

#[no_mangle]
extern "C" fn handle() {
    let action: Action = msg::load().expect("Failed to decode action");
    let game = unsafe { GAME_SESSION.as_mut().expect("Game session not initialized") };

    match game.session_status {
        SessionStatus::Waiting => match action {
            Action::StartGame => {
                if let GameStatus::None = game.status {
                    let start_msg = WordleAction::StartGame {
                        user: msg::source(),
                    };
                    let msg_id = msg::send(game.wordle_program, start_msg, 0)
                        .expect("Failed to send StartGame message");

                    game.msg_ids = Some((msg_id, msg::id()));
                    game.session_status = SessionStatus::MessageSent;

                    exec::wait();
                } else {
                    panic!("Game already in progress");
                }
            }
            Action::CheckWord { word } => {
                if word.len() != 5 {
                    panic!("Word must be 5 letters long");
                }
                if !word.chars().all(|c| c.is_ascii_lowercase()) {
                    panic!("Word must contain only lowercase letters");
                }

                if let GameStatus::InProgress { attempts, .. } = game.status {
                    if attempts >= MAX_ATTEMPTS {
                        panic!("Maximum attempts reached");
                    }

                    let check_msg = WordleAction::CheckWord {
                        user: msg::source(),
                        word,
                    };
                    let msg_id = msg::send(game.wordle_program, check_msg, 0)
                        .expect("Failed to send CheckWord message");

                    game.msg_ids = Some((msg_id, msg::id()));
                    game.session_status = SessionStatus::MessageSent;

                    exec::wait();
                } else {
                    panic!("No active game");
                }
            }
            Action::CheckGameStatus => {
                if let GameStatus::InProgress { start_height, .. } = game.status {
                    if exec::block_height() - start_height >= MAX_BLOCKS {
                        game.status = GameStatus::Finished(GameResult::TimeOut);
                        msg::reply(Event::GameOver(GameResult::TimeOut), 0)
                            .expect("Failed to reply with GameOver");
                    }
                }
            }
        },
        SessionStatus::MessageSent => {
            msg::reply(Event::GameError("Message already sent".to_string()), 0)
                .expect("Failed to reply with already sent message");
        }
        SessionStatus::ReplyReceived(ref wordle_event) => {
            match wordle_event {
                WordleEvent::GameStarted { user: _ } => {
                    game.status = GameStatus::InProgress {
                        attempts: 0,
                        start_height: exec::block_height(),
                    };

                    msg::send_delayed(
                        exec::program_id(),
                        Action::CheckGameStatus {},
                        0,
                        MAX_BLOCKS,
                    )
                    .expect("Failed to send delayed message");

                    msg::reply(Event::GameStarted, 0).expect("Failed to reply GameStarted");
                }
                WordleEvent::WordChecked {
                    user,
                    correct_positions,
                    contained_in_word,
                } => {
                    if let GameStatus::InProgress {
                        ref mut attempts, ..
                    } = game.status
                    {
                        *attempts += 1;

                        if correct_positions.len() == 5 {
                            game.status = GameStatus::Finished(GameResult::Win);
                            msg::reply(Event::GameOver(GameResult::Win), 0)
                                .expect("Failed to reply Win result");
                        } else if *attempts >= MAX_ATTEMPTS {
                            game.status = GameStatus::Finished(GameResult::Lose);
                            msg::reply(Event::GameOver(GameResult::Lose), 0)
                                .expect("Failed to reply Lose result");
                        } else {
                            msg::reply(
                                Event::WordChecked {
                                    player: *user,
                                    correct_positions: correct_positions.clone(),
                                    contained_letters: contained_in_word.clone(),
                                },
                                0,
                            )
                            .expect("Failed to reply WordChecked");
                        }
                    }
                }
            }
            game.session_status = SessionStatus::Waiting;
        }
    }
}

#[no_mangle]
extern "C" fn handle_reply() {
    let reply: WordleEvent = msg::load().expect("Failed to decode reply");
    let game = unsafe { GAME_SESSION.as_mut().expect("Game session not initialized") };

    if let Some((sent_msg_id, original_msg_id)) = game.msg_ids {
        if msg::reply_to().expect("Failed to get reply_to") == sent_msg_id
            && game.session_status == SessionStatus::MessageSent
        {
            game.session_status = SessionStatus::ReplyReceived(reply);
            exec::wake(original_msg_id).expect("Failed to wake message");
        }
    }
}

#[no_mangle]
extern "C" fn state() {
    let game = unsafe { GAME_SESSION.as_ref().expect("Game session not initialized") };
    msg::reply(game.status.clone(), 0).expect("Failed to return state");
}
