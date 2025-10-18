use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;

use crate::payoffs::Payoffs;

//////////////////////////////////////////////////////////////////////////////////////////////////

/// A State is intended to correspond roughly to a turn of a game.
pub trait State<G: Game>: Debug + Eq + Clone + Hash + Sized + Sync + Send {
    /// Which player has a choice of moves from this `State`?
    fn current_players(&self) -> HashSet<G::Player>;

    /// Is this a terminal state for the `Game`?
    fn is_over(&self) -> bool;

    /// What moves are available for the given `Player` from this `State`?
    /// If there are none for all `Player`s, this is a terminal state.
    /// For simplicity, in games where a player has no valid moves, but the
    /// game is not actually over, this would be itself encoded as an explicit
    /// "pass" move.
    ///
    /// It must be the case that when calling `next`, all moves for a given
    /// player are compatible with all possible moves of all other players.
    /// For example, consider a game where players are moving pieces on a
    /// shared board.  If only one piece can be in a given location, then
    /// multiple players moving a piece to the same location would generally
    /// not be possible.  There are a number of different ways to resolve this conflict,
    /// but they will be game specific.  One approach would be to introduce some kind of
    /// tie-breaking rule.  Or if a move could conflict it would just not be
    /// reported a possible move, etc.
    fn moves<'l>(&'l self, player: &G::Player) -> G::MoveIterator<'l>;

    /// Compute the new state resulting from making the given moves, if
    /// there is one.  This will only produce a result for moves returned
    /// by the moves() function.
    fn next(&self, moves: &HashMap<G::Player, G::Move>) -> Option<Self>;

    /// Returns a map payoffs for this state.  A positive payoff corresponds to
    /// winning, a negative a payoff corresponds to losing, and a zero payoff
    /// indicates a draw or no conclusion yet.  The magnitude of the payoff
    /// corresponds to the strength of the win or loss.  In games where there
    /// is no scoring, the payoffs will generally be normalized to the unit
    /// magnitude.
    ///
    /// Unlike some other designs, returning the payoff of all players is
    /// intended to perhaps simplify some computations.  Other implementations
    /// of games tend to make the payoff relative to the current player.  This
    /// leads to code that needs to negate or swap values when propagating
    /// the payoffs  back through earlier search states.
    fn payoffs(&self) -> Payoffs<G>;
}

//////////////////////////////////////////////////////////////////////////////////////////////////

/// TODO Seems like Clone shouldn't really be necessary?
pub trait Game: Clone + Default + Sized {
    /// Descriptive name for the game.  For use in debugging output, etc.
    fn name() -> String;

    /// The type of a player for this game.
    type Player: Debug + PartialEq + Eq + PartialOrd + Ord + Hash + Clone;
    /// The type of a game move.
    type Move: Debug + PartialEq + Eq + Hash + Clone;
    /// Provide a custom iterator for a given game, so that we can lazily
    /// enumerate moves.
    type MoveIterator<'l>: Iterator<Item=Self::Move>;
    /// The type of the states associated with this game.
    type State: State<Self>;
    /// The type of a configuration for this game.
    type Config: Debug + Eq + Hash + Clone + Send;

    /// Create a new instance of this game.
    fn new(config: &Self::Config) -> Self;

    /// Obtain the set of players in this game.
    fn players(&self) -> HashSet<Self::Player>;

    /// Create a new initial starting state.
    // TODO Allow for additional configuration?
    fn start(&self) -> Self::State;
}
