use std::collections::HashMap;

use endgame_ludic::game::{Game as _, State as _};
use endgame_ludic::strategy::{
    ConstantStrategy, FailureStrategy, FirstMoveStrategy, RandomStrategy, Strategy, TryStrategy,
};
use endgame_ludic::utils::play_out_with_two_strategies;
use rps::{Game, Move, Player};

fn make_game_with_rounds(rounds: usize) -> Game {
    assert!(rounds > 0, "There must be at least one round.");
    Game::new(&rps::Config { rounds })
}

fn apply_round(state: &rps::State, a_mv: Move, b_mv: Move) -> rps::State {
    state
        .next(&HashMap::from([(Player::A, a_mv), (Player::B, b_mv)]))
        .expect("Both players provided moves; next state should always exist.")
}

fn payoffs_tuple(state: &rps::State) -> (usize, usize) {
    let payoffs = state.payoffs();
    let a = *payoffs
        .payoff(&Player::A)
        .expect("Player A should have a payoff");
    let b = *payoffs
        .payoff(&Player::B)
        .expect("Player B should have a payoff");
    (a.0 as usize, b.0 as usize)
}

fn play_with_strategies<S1, S2>(strat_a: &mut S1, strat_b: &mut S2, game: &Game) -> rps::State
where
        for<'l> S1: Strategy<Game, State<'l>=()>,
        for<'l> S2: Strategy<Game, State<'l>=()>,
{
    play_out_with_two_strategies(
        game,
        Player::A,
        strat_a,
        &mut (),
        Player::B,
        strat_b,
        &mut (),
        game.start(),
    )
}

#[test]
fn test_early_termination_best_of_three() {
    use Move::*;

    // Best of 3: as soon as A (or B) reaches 2 wins, the game should be over.
    let game = make_game_with_rounds(3);
    let mut state = game.start();
    assert!(!state.is_over(), "Initial state should not be over");

    // Round 1: A wins (Rock beats Scissors)
    state = apply_round(&state, Rock, Scissors);
    assert!(
        !state.is_over(),
        "After 1-0 lead, no majority yet for best-of-3"
    );

    // Round 2: A wins again (Paper beats Rock) -> A has 2 wins out of 3; majority
    // clinched
    state = apply_round(&state, Paper, Rock);
    assert!(
        state.is_over(),
        "Game should end early when a player reaches majority (2 of 3)"
    );

    // Verify payoffs reflect A's two wins
    assert_eq!(payoffs_tuple(&state), (2, 0));
}

#[test]
fn test_continues_with_no_majority() {
    use Move::*;

    let game = make_game_with_rounds(3);
    let mut state = game.start();

    // Round 1: tie (no wins)
    state = apply_round(&state, Rock, Rock);
    assert!(
        !state.is_over(),
        "Tie in the first round should not end the game early"
    );
    assert_eq!(
        payoffs_tuple(&state),
        (0, 0),
        "Tie round should give zero payoffs so far"
    );

    // Round 2: A wins, but cannot have achieved majority yet (A has 1 win, majority
    // is 2)
    state = apply_round(&state, Scissors, Paper);
    assert!(
        !state.is_over(),
        "With only 1 win out of best-of-3, the game should continue"
    );
    assert_eq!(payoffs_tuple(&state), (1, 0));
}

#[test]
fn tests_ends_with_exhaustion() {
    use Move::*;
    // Verify that if all rounds are ties, that we have to play all rounds before
    // the game ends in a draw.
    let total_rounds = 5;
    let game = make_game_with_rounds(total_rounds);
    let mut state = game.start();
    let mut rounds = 0;
    while !state.is_over() {
        state = apply_round(&state, Rock, Rock);
        assert!(
            rounds < total_rounds || !state.is_over(),
            "Game should not end before all rounds are exhausted."
        );
        rounds += 1;
    }
    assert_eq!(
        rounds, total_rounds,
        "Not enough rounds were played {rounds} vs {total_rounds}"
    );

    // Final payoffs should be a draw.
    assert_eq!(payoffs_tuple(&state), (0, 0));
}

#[test]
fn test_withs_constant_strategy() {
    let game = make_game_with_rounds(3);
    // A: Rock, B: Scissors -> A wins each played round; early termination after 2
    // wins.
    let mut a = ConstantStrategy::<Game>::new(Player::A, Some(Move::Rock));
    let mut b = ConstantStrategy::<Game>::new(Player::B, Some(Move::Scissors));
    let state = play_with_strategies(&mut a, &mut b, &game);
    assert!(state.is_over());
    assert_eq!(payoffs_tuple(&state), (2, 0));
}

#[test]
fn test_firstmove_vs_constant_strategy() {
    let game = make_game_with_rounds(3);
    // FirstMoveStrategy for A picks Rock; ConstantStrategy for B picks Paper -> B
    // wins; early termination after 2 wins.
    let mut a = FirstMoveStrategy::<Game>::new();
    let mut b = ConstantStrategy::<Game>::new(Player::B, Some(Move::Paper));
    let state = play_with_strategies(&mut a, &mut b, &game);
    assert!(state.is_over());
    assert_eq!(payoffs_tuple(&state), (0, 2));
}

#[test]
fn test_random_strategy_deterministic_with_seed() {
    let game = make_game_with_rounds(5);
    let mut a1 = RandomStrategy::<Game>::new(42);
    let mut b1 = RandomStrategy::<Game>::new(123);
    let state1 = play_with_strategies(&mut a1, &mut b1, &game);
    let p1 = payoffs_tuple(&state1);

    // Re-run with the same seeds; expect identical payoffs.
    let mut a2 = RandomStrategy::<Game>::new(42);
    let mut b2 = RandomStrategy::<Game>::new(123);
    let state2 = play_with_strategies(&mut a2, &mut b2, &game);
    let p2 = payoffs_tuple(&state2);

    assert_eq!(
        p1, p2,
        "Seeded RandomStrategy should be deterministic over same game"
    );
}

#[test]
fn test_failure_and_try_fallback_firstmove() {
    let game = make_game_with_rounds(3);

    // Direct use of FailureStrategy cannot choose a move.
    let mut fail = FailureStrategy::<Game>::new();
    let mut state = game.start();
    assert!(
        fail.choose(&mut (), &state, &Player::A).is_none(),
        "FailureStrategy should always give up."
    );

    // TryStrategy should fall back to FirstMoveStrategy and be able to play the
    // game to completion.
    let mut a = TryStrategy::<Game, _, _>::new(FailureStrategy::new(), FirstMoveStrategy::new());
    let mut b = TryStrategy::<Game, _, _>::new(FailureStrategy::new(), FirstMoveStrategy::new());

    state = game.start();
    while !state.is_over() {
        let a_choice = a
            .choose(&mut ((), ()), &state, &Player::A)
            .expect("TryStrategy A failed to choose")
            .expect("TryStrategy A reported no available move unexpectedly");
        let b_choice = b
            .choose(&mut ((), ()), &state, &Player::B)
            .expect("TryStrategy B failed to choose")
            .expect("TryStrategy B reported no available move unexpectedly");
        state = apply_round(&state, a_choice, b_choice);
    }

    // Both will always pick Rock, leading to draws each round.
    assert_eq!(payoffs_tuple(&state), (0, 0));
}
