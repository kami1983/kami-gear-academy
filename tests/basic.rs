use gtest::{Program, System };
use pebbles_game_io::*;
use gstd::{Encode, Decode};

const USER_ID_DEPLOY: u64 = 42;
const USER_ID_ALICE: u64 = 43;
// const USER_ID_BOB: u64 = 44;

fn init_program(sys: &System, init_msg: PebblesInit) -> GameState{
    // let sys = System::new();
    sys.init_logger();
    let game = Program::current(sys);
    assert_eq!(game.id(), 1.into());

    let init_result = game.send_bytes(USER_ID_DEPLOY, init_msg.encode());
    assert!(!init_result.main_failed());

    game.read_state(0).expect("Invalid state.")
}

#[test]
fn test_init()  {

    let sys = System::new();
    let init_state = init_program(&sys, PebblesInit {
        difficulty: DifficultyLevel::Easy,
        pebbles_count: 15,
        max_pebbles_per_turn: 2,
    });

    assert_eq!(init_state.pebbles_count, 15);
    assert_eq!(init_state.max_pebbles_per_turn, 2);
    assert_eq!(init_state.difficulty, DifficultyLevel::Easy);
    assert_eq!(init_state.pebbles_remaining, 15);

}

#[test]
fn test_turn() {
    let max_pebbles_per_turn = 2;
    let mut user_turn = 1;

    let sys = System::new();

    let init_state = init_program(&sys, PebblesInit {
        difficulty: DifficultyLevel::Easy,
        pebbles_count: 15,
        max_pebbles_per_turn,
    });
    let game = sys.get_program(1).unwrap();

    let action = PebblesAction::Turn(user_turn + max_pebbles_per_turn);
    let result = game.send(USER_ID_ALICE, action);
    assert!(result.main_failed()); // should fail

    let action = PebblesAction::Turn(user_turn);
    let result = game.send(USER_ID_ALICE, action);
    assert!(!result.main_failed()); // should pass

    // Check the event
    let event = PebblesEvent::decode(&mut &result.log()[0].payload()[..]).expect("Failed to decode event");

    // extract the turn from the event, it is a program turn value.
    let program_turn = match event {
        PebblesEvent::CounterTurn(n) => n,
        _ => panic!("Unexpected event"),
    };

    assert_eq!(event, PebblesEvent::CounterTurn(program_turn));

    // Check the state
    let mut state: GameState  = game.read_state(0).expect("Invalid state.");

    assert!(state.pebbles_remaining > 0);
    assert_eq!(state.first_player, init_state.first_player);

    let mut winner = Player::Program;

    while state.pebbles_remaining > 0 {
        if state.pebbles_remaining <= max_pebbles_per_turn {
            // all pebbles are taken by the user for the last turn and the user wins.
            user_turn = state.pebbles_remaining;
            winner = Player::User;
        }
        let action = PebblesAction::Turn(user_turn);
        let result = game.send(USER_ID_ALICE, action);
        assert!(!result.main_failed());
        state = game.read_state(0).expect("Invalid state.");
    }

    // Check the winner
    assert_eq!(state.winner.expect("No Winner"), winner);
}

#[test]
fn test_give_up() {
    let sys = System::new();
    let _ = init_program(&sys, PebblesInit {
        difficulty: DifficultyLevel::Easy,
        pebbles_count: 15,
        max_pebbles_per_turn: 2,
    });
    let game = sys.get_program(1).unwrap();

    game.send(USER_ID_ALICE, PebblesAction::GiveUp);
    let state: GameState = game.read_state(0).expect("Invalid state.");
    assert_eq!(state.winner.expect("No Winner"), Player::Program);
}

#[test]
fn test_restart() {
    let sys = System::new();
    let _ = init_program(&sys, PebblesInit {
        difficulty: DifficultyLevel::Easy,
        pebbles_count: 15,
        max_pebbles_per_turn: 2,
    });
    let game = sys.get_program(1).unwrap();

    game.send(USER_ID_ALICE, PebblesAction::GiveUp);
    let state: GameState = game.read_state(0).expect("Invalid state.");
    assert_eq!(state.winner.expect("No Winner"), Player::Program);

    game.send(USER_ID_ALICE, PebblesAction::Restart {
        difficulty: DifficultyLevel::Hard,
        pebbles_count: 35,
        max_pebbles_per_turn: 5,
    });
    let state: GameState = game.read_state(0).expect("Invalid state.");
    assert_eq!(state.winner, None);
    assert_eq!(state.max_pebbles_per_turn, 5);
    assert_eq!(state.difficulty, DifficultyLevel::Hard);
    assert_eq!(state.pebbles_count, 35);

}