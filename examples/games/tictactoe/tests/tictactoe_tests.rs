use endgame_grid::square;
use endgame_ludic::game::Game as _;
// for Game::new/start trait methods
use endgame_ludic::game::State as _;
// bring trait methods (moves, next, is_over, current_players) into scope
use endgame_ludic::strategy::{
    ConstantStrategy, FailureStrategy, FirstMoveStrategy, RandomStrategy, Strategy, TryStrategy,
};
use tictactoe::{Game, Move, Player};

fn payoffs_tuple(state: &tictactoe::State) -> (f64, f64) {
    let p = state.payoffs();
    let x = *p.payoff(&Player::X).expect("Payoff for X should exist");
    let o = *p.payoff(&Player::O).expect("Payoff for O should exist");
    (x.0, o.0)
}

#[test]
fn test_failure_strategy_always_returns_none() {
    let state = Game::default().start();
    // Initial state should be an undecided game with zero payoffs for both players.
    assert_eq!(payoffs_tuple(&state), (0.0, 0.0));

    let mut strategy = FailureStrategy::<Game>::new();
    for player in [Player::X, Player::O] {
        assert!(
            strategy.choose(&mut (), &state, &player).is_none(),
            "FailureStrategy should give up for all players."
        );
    }
}
#[test]
fn test_firstmove_strategy() {
    let state = Game::default().start();

    let expected_first: Move = state
        .moves(&Player::X)
        .next()
        .expect("There should be at least one legal move for X on the initial state.");

    let mut strat = FirstMoveStrategy::<Game>::new();
    let choice = strat
        .choose(&mut (), &state, &Player::X)
        .expect("FirstMoveStrategy should return a move on non-terminal state.");

    assert_eq!(
        choice,
        Some(expected_first),
        "FirstMoveStrategy should pick the first available legal move for X on the initial board."
    );

    assert_eq!(
        strat.choose(&mut (), &state, &Player::O),
        Some(None),
        "FirstMoveStrategy should return no move for player O on the initial state."
    );
}

#[test]
fn test_constant_strategy_returns_configured_choice() {
    let state = Game::default().start();
    // Configure X with a valid initial move.
    let m = Move(square::Coord::new(0, 0));
    let mut o_pass = ConstantStrategy::<Game>::new(Player::X, Some(m.clone()));
    assert_eq!(
        o_pass.choose(&mut (), &state, &Player::X),
        Some(Some(m)),
        "ConstantStrategy should have returned the specified move for X."
    );

    // Configure O with None on the initial state where O has no legal moves.
    let mut o_pass = ConstantStrategy::<Game>::new(Player::O, None);
    assert_eq!(
        o_pass.choose(&mut (), &state, &Player::O),
        Some(None),
        "ConstantStrategy should return Some(None) when no moves are available for O"
    );

    // If the player is not present in the map (failure configuration), choose returns None.
    let mut fail_cfg = ConstantStrategy::<Game>::failure();
    assert!(
        fail_cfg.choose(&mut (), &state, &Player::X).is_none(),
        "ConstantStrategy::failure should behave like FailureStrategy (None)"
    );
}

#[test]
fn test_try_strategy_falls_back() {
    let state = Game::default().start();

    let mut first = FirstMoveStrategy::<Game>::new();
    let first_x = first
        .choose(&mut (), &state, &Player::X)
        .expect("FirstMoveStrategy should always return a move on non-terminal state.")
        .expect("There should be a legal move for X on the initial state.");

    let initial = FailureStrategy::<Game>::new();
    let fallback = ConstantStrategy::new(Player::X, Some(first_x));
    let mut try_strat = TryStrategy::new(initial, fallback);

    let choice = try_strat.choose(&mut ((), ()), &state, &Player::X);
    assert_eq!(
        choice,
        Some(Some(first_x)),
        "TryStrategy should return fallback move from ConstantStrategy when initial strategy fails."
    );

    // If both strategies fail, the result should be None.
    let mut both_fail =
        TryStrategy::new(FailureStrategy::<Game>::new(), ConstantStrategy::failure());
    let choice2 = both_fail.choose(&mut ((), ()), &state, &Player::X);
    assert!(
        choice2.is_none(),
        "TryStrategy should return None when both strategies fail."
    );
}

#[test]
fn test_random_strategy_is_deterministic_with_seed() {
    let state = Game::default().start();

    // Two strategies with the same seed on the same state should pick the same
    // move.
    let mut r1 = RandomStrategy::<Game>::new(123456789);
    let mut r2 = RandomStrategy::<Game>::new(123456789);

    let c1 = r1
        .choose(&mut (), &state, &Player::X)
        .expect("RandomStrategy should always return a move choice on non-terminal state.");
    let c2 = r2
        .choose(&mut (), &state, &Player::X)
        .expect("RandomStrategy should always return a move choice on non-terminal state.");

    assert_eq!(
        c1, c2,
        "RandomStrategy should be deterministic given the same seed and state."
    );

    // The chosen move, if any, must be among the state's legal moves for X.
    if let Some(mv) = c1 {
        let legal: Vec<Move> = state.moves(&Player::X).collect();
        assert!(
            legal.contains(&mv),
            "RandomStrategy should return legal moves."
        );
    } else {
        unreachable!("On the initial state there should always be a legal move for X.");
    }
}