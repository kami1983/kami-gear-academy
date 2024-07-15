#![no_std]

use gstd::{msg, exec};
use pebbles_game_io::*;

static mut PEBBLES_GAME: Option<GameState> = None;

#[no_mangle]
extern "C" fn init() {
    let init_msg: PebblesInit = msg::load().expect("Unable to load PebblesInit");

    if init_msg.pebbles_count == 0 || init_msg.max_pebbles_per_turn > init_msg.pebbles_count {
        panic!("Invalid init message");
    }

    // Randomly choose the first player
    let first_player = if get_random_u32() % 2 == 0 {
        Player::User
    } else {
        Player::Program
    };

    // Initialize the game state
    let mut game_state = GameState {
        pebbles_count: init_msg.pebbles_count.clone(),
        max_pebbles_per_turn: init_msg.max_pebbles_per_turn,
        pebbles_remaining: init_msg.pebbles_count,
        difficulty: init_msg.difficulty,
        first_player: first_player.clone(),
        winner: None,
    };

    if let Player::Program = first_player {
        program_turn(&mut game_state);
    }

    unsafe {
        PEBBLES_GAME = Some(game_state);
    }
}

#[no_mangle]
extern "C" fn handle() {
    let action: PebblesAction = msg::load().expect("Unable to load PebblesAction");

    unsafe {
        if let Some(ref mut game_state) = PEBBLES_GAME {
            match action {
                PebblesAction::Turn(n) => {
                    if n < 1 || n > game_state.max_pebbles_per_turn || n > game_state.pebbles_remaining {
                        gstd::debug!("Invalid turn: {}", n);
                        panic!("Invalid turn");
                    }

                    game_state.pebbles_remaining -= n;
                    if game_state.pebbles_remaining == 0 {
                        game_state.winner = Some(Player::User);
                        msg::reply(PebblesEvent::Won(Player::User), 0).expect("Unable to reply");
                    } else {
                        program_turn(game_state);
                    }
                },
                PebblesAction::GiveUp => {
                    game_state.winner = Some(Player::Program);
                    msg::reply(PebblesEvent::Won(Player::Program), 0).expect("Unable to reply");
                },
                PebblesAction::Restart { difficulty, pebbles_count, max_pebbles_per_turn } => {
                    game_state.difficulty = difficulty;
                    game_state.pebbles_count = pebbles_count;
                    game_state.max_pebbles_per_turn = max_pebbles_per_turn;
                    game_state.pebbles_remaining = pebbles_count;
                    game_state.winner = None;

                    let first_player = if get_random_u32() % 2 == 0 {
                        Player::User
                    } else {
                        Player::Program
                    };
                    game_state.first_player = first_player.clone();

                    if let Player::Program = first_player {
                        program_turn(game_state);
                    }
                }
            }
        }
    }
}

#[no_mangle]
extern "C" fn state() {
    unsafe {
        if let Some(ref game_state) = PEBBLES_GAME {
            msg::reply(game_state.clone(), 0).expect("Unable to reply");
        }
    }
}


fn get_random_u32() -> u32 {
    let salt = msg::id();
    let (hash, _num) = exec::random(salt.into()).expect("get_random_u32(): random call failed");
    u32::from_le_bytes([hash[0].clone(), hash[1].clone(), hash[2].clone(), hash[3].clone()])
}


fn program_turn(game_state: &mut GameState) {
    let move_n = match game_state.difficulty {
        DifficultyLevel::Easy => {
            if game_state.pebbles_remaining > game_state.max_pebbles_per_turn {
                get_random_u32() % game_state.max_pebbles_per_turn + 1
            } else {
                game_state.pebbles_remaining
            }
        },
        DifficultyLevel::Hard => find_best_move(game_state.pebbles_remaining, game_state.max_pebbles_per_turn),
    };

    game_state.pebbles_remaining -= move_n;

    if game_state.pebbles_remaining == 0 {
        game_state.winner = Some(Player::Program);
        msg::reply(PebblesEvent::Won(Player::Program), 0).expect("Unable to reply");
    } else {
        msg::reply(PebblesEvent::CounterTurn(move_n), 0).expect("Unable to reply");
    }
}

fn find_best_move(pebbles_remaining: u32, max_pebbles_per_turn: u32) -> u32 {
    let modulo = (max_pebbles_per_turn + 1) as u32;
    let remainder = pebbles_remaining % modulo;
    if remainder == 0 {
        1
    } else {
        remainder
    }
}