#![no_std]
use gmeta::{In, InOut, Metadata, Out};
use gstd::{prelude::*, ActorId};

pub struct ContractMetadata;

impl Metadata for ContractMetadata {
    type Init = In<ActorId>;
    type Handle = InOut<Action, Event>;
    type Reply = In<Event>;
    type Others = InOut<String, String>;
    type Signal = ();
    type State = Out<GameStatus>;
}

#[derive(Encode, Decode, TypeInfo)]
pub enum Action {
    StartGame,
    CheckWord { word: String },
    CheckGameStatus,
}

#[derive(Encode, Decode, TypeInfo, Debug, Clone)]
pub enum Event {
    GameStarted,
    MoveMade {
        correct_positions: Vec<u8>,
        contained_in_word: Vec<u8>,
    },
    WordChecked {
        player: ActorId,
        correct_positions: Vec<u8>,
        contained_letters: Vec<u8>,
    },
    GameOver(GameResult),
    GameError(String),
}

#[derive(Clone, Encode, Decode, TypeInfo, Debug, PartialEq)]
pub enum GameResult {
    Win,
    Lose,
    TimeOut,
}

#[derive(Clone, Encode, Decode, TypeInfo, Debug, PartialEq)]
pub enum GameStatus {
    None,
    InProgress { attempts: u8, start_height: u32 },
    Finished(GameResult),
}

#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum MetadataInit {
    Init(ActorId),
}

#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum MetadataState {
    GameStatus(GameStatus),
}

#[derive(Encode, Decode, TypeInfo)]
pub struct InitContractData {
    pub dictionary: ActorId,
}

#[derive(Debug, PartialEq, Clone, Encode, Decode, TypeInfo)]
pub enum WordleEvent {
    GameStarted,
    WordChecked {
        player: ActorId,
        correct_positions: Vec<u8>,
        contained_letters: Vec<u8>,
    },
    GameOver(GameResult),
    GameError(String),
}

impl From<Event> for WordleEvent {
    fn from(event: Event) -> Self {
        match event {
            Event::GameStarted => WordleEvent::GameStarted,
            Event::WordChecked {
                player,
                correct_positions,
                contained_letters,
            } => WordleEvent::WordChecked {
                player,
                correct_positions,
                contained_letters,
            },
            Event::GameOver(result) => WordleEvent::GameOver(result),
            Event::GameError(msg) => WordleEvent::GameError(msg),
            _ => WordleEvent::GameError("Unknown event".to_string()),
        }
    }
}

pub trait IntoGameSessionEvent {
    fn into_game_session_event(self) -> Event;
}

impl IntoGameSessionEvent for wordle_io::Event {
    fn into_game_session_event(self) -> Event {
        match self {
            wordle_io::Event::GameStarted { user: _ } => Event::GameStarted,
            wordle_io::Event::WordChecked {
                user,
                correct_positions,
                contained_in_word,
            } => Event::WordChecked {
                player: user,
                correct_positions,
                contained_letters: contained_in_word,
            },
        }
    }
}
