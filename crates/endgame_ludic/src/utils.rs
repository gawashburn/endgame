use crate::game::{Game, State};
use crate::strategy::Strategy;
use std::collections::{HashMap, HashSet};

/// Helper to play a game using the same `Strategy` for all players.  Given a
/// strategy state and a starting `State` it will play until the game is
/// complete or the `Strategy` cannot decide on a move for a player.
///
/// Ideally, we would provide a version that could use a distinct `Strategy` for
/// each player. However, given that `Strategy` is not `dyn` compatible, it
/// would be necessary to first define a wrapper `enum` that could hold all
/// `Strategy` of interest.
// TODO Introduce an enum to distinguish the two return cases?
pub fn play_out_with_strategy<'l, G: Game, S: Strategy<G>>(
    strategy: &mut S,
    strategy_state: &mut S::State<'l>,
    mut game_state: G::State,
) -> G::State {
    while !game_state.is_over() {
        let mut moves = HashMap::new();
        for player in game_state.current_players() {
            match strategy.choose(strategy_state, &game_state, &player) {
                Some(Some(m)) => {
                    moves.insert(player, m);
                }
                Some(None) => {
                    // No-op as this player cannot move.
                }
                None => {
                    // Strategy could not decide, so just return the current state.
                    return game_state;
                }
            }
        }
        // Transition to the next state.
        game_state = game_state
            .next(&moves)
            .expect("Strategy produced an invalid move.")
    }
    game_state
}

/// Plays out an exactly two-player game with (possibly) distinct strategies.  Given strategy
/// state and a starting game `State` it will play until the game is complete or one of the
/// `Strategy` cannot decide on a move for a player.
///
/// It must be the case that the two provided `Player`s match the `Player`s reported by the
/// `State`.
pub fn play_out_with_two_strategies<'l, G, S1, S2>(
    game: &G,
    player1: G::Player,
    strategy1: &mut S1,
    strategy_state1: &mut S1::State<'l>,
    player2: G::Player,
    strategy2: &mut S2,
    strategy_state2: &mut S2::State<'l>,
    mut state: G::State,
) -> G::State
where
    G: Game,
    S1: Strategy<G>,
    S2: Strategy<G>,
{
    // Verify that these are exactly the players for this given game instance.
    assert!(
        {
            let state_players = game.players();
            let provided: HashSet<G::Player> = HashSet::from([player1.clone(), player2.clone()]);
            state_players == provided
        },
        "Provided players do not match the players for the given state. Provided: {:?} and {:?}, State: {:?}",
        player1, player2,
        game.players()
    );

    while !state.is_over() {
        let mut moves = HashMap::new();
        for player in state.current_players() {
            let choice = if player == player1 {
                strategy1.choose(strategy_state1, &state, &player)
            } else if player == player2 {
                strategy2.choose(strategy_state2, &state, &player)
            } else {
                panic!(
                    "State has a player in state that is not one of two provided players. Player: {:?}",
                    player
                );
            };

            match choice {
                Some(Some(m)) => {
                    moves.insert(player, m);
                }
                Some(None) => {
                    // No-op as this player cannot move.
                }
                None => {
                    // Strategy could not decide, so just return the current state.
                    return state;
                }
            }
        }
        state = state
            .next(&moves)
            .expect("Strategy produced an invalid move.");
    }

    state
}
