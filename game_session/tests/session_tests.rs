use game_session_io::*;
use gstd::ActorId;
use gtest::{Program, System};

const USER: u64 = 10;
const WORDLE_PROGRAM_ID: u64 = 2;

#[test]
fn test_game_start_and_lose() {
    let system = System::new();
    system.init_logger();
    system.mint_to(USER, 1000000000000000000000);
    let session_program: Program = Program::from_file(
        &system,
        "./wasm32-unknown-unknown/debug/game_session_program.opt.wasm",
    );

    let wordle_program: Program = Program::from_file(
        &system,
        "./wasm32-unknown-unknown/debug/wordle_program.opt.wasm",
    );
    wordle_program.send_bytes(USER, []);
    system.run_next_block();
    let wordle_program_address: ActorId = WORDLE_PROGRAM_ID.into();
    session_program.send(USER, wordle_program_address);
    system.run_next_block();
    session_program.send(USER, Action::StartGame);
    system.run_next_block();
    let state: GameStatus = session_program.read_state(()).unwrap();
    assert_eq!(
        state,
        GameStatus::InProgress {
            attempts: 0,
            start_height: 3
        }
    );

    session_program.send(
        USER,
        Action::CheckWord {
            word: "apple".to_string(),
        },
    );
    system.run_next_block();
    for _ in 0..6 {
        session_program.send(
            USER,
            Action::CheckWord {
                word: "apple".to_string(),
            },
        );
        system.run_next_block();
    }
    let final_state: GameStatus = session_program.read_state(()).unwrap();
    assert_eq!(final_state, GameStatus::Finished(GameResult::Lose));
}

#[test]
fn test_game_timeout() {
    let system = System::new();
    system.init_logger();
    system.mint_to(USER, 1000000000000000000000);
    let session_program: Program = Program::from_file(
        &system,
        "./wasm32-unknown-unknown/debug/game_session_program.opt.wasm",
    );

    let wordle_program: Program = Program::from_file(
        &system,
        "./wasm32-unknown-unknown/debug/wordle_program.opt.wasm",
    );

    wordle_program.send_bytes(USER, []);
    system.run_next_block();
    let wordle_program_address: ActorId = WORDLE_PROGRAM_ID.into();

    session_program.send(USER, wordle_program_address);
    system.run_next_block();
    session_program.send(USER, Action::StartGame);
    system.run_next_block();
    let state: GameStatus = session_program.read_state(()).unwrap();
    assert_eq!(
        state,
        GameStatus::InProgress {
            attempts: 0,
            start_height: 3
        }
    );

    session_program.send(
        USER,
        Action::CheckWord {
            word: "apple".to_string(),
        },
    );
    system.run_next_block();
    system.run_to_block(300);
    let timeout_state: GameStatus = session_program.read_state(()).unwrap();
    assert_eq!(timeout_state, GameStatus::Finished(GameResult::TimeOut));
}

#[test]
fn test_game_win() {
    let system = System::new();
    system.init_logger();
    system.mint_to(USER, 1000000000000000000000);
    let session_program: Program = Program::from_file(
        &system,
        "./wasm32-unknown-unknown/debug/game_session_program.opt.wasm",
    );

    let wordle_program: Program = Program::from_file(
        &system,
        "wasm32-unknown-unknown/debug/wordle_program.opt.wasm",
    );

    wordle_program.send_bytes(USER, []);
    system.run_next_block();
    let wordle_program_address: ActorId = WORDLE_PROGRAM_ID.into();

    session_program.send(USER, wordle_program_address);
    system.run_next_block();
    session_program.send(USER, Action::StartGame);
    system.run_next_block();
    let state: GameStatus = session_program.read_state(()).unwrap();
    assert_eq!(
        state,
        GameStatus::InProgress {
            attempts: 0,
            start_height: 3
        }
    );

    session_program.send(
        USER,
        Action::CheckWord {
            word: "apple".to_string(),
        },
    );
    system.run_next_block();
    session_program.send(
        USER,
        Action::CheckWord {
            word: "human".to_string(),
        },
    );
    system.run_next_block();
    session_program.send(
        USER,
        Action::CheckWord {
            word: "horse".to_string(),
        },
    );
    system.run_next_block();
    session_program.send(
        USER,
        Action::CheckWord {
            word: "house".to_string(),
        },
    );
    system.run_next_block();
    let final_state: GameStatus = session_program.read_state(()).unwrap();
    assert_eq!(final_state, GameStatus::Finished(GameResult::Win));
}
