use crate::game::{Game, State};
use rand::Rng;
use rand_chacha::ChaCha20Rng;
use rand_core::{CryptoRngCore, SeedableRng};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{DefaultHasher, Hash, Hasher};
//use rand_core::{CryptoRngCore, RngCore, SeedableRng};
use std::marker::PhantomData;

//////////////////////////////////////////////////////////////////////////////////////////////////

/// An abstract representation of a strategy for playing a given game.
pub trait Strategy<G: Game>: Debug {
    /// The type of additional per invocation state that can be supplied to an
    /// invocation of `choose` on a `Strategy`.
    // TODO Better name?
    type Config<'l>;

    /// Given a `State` of the `Game`, attempt to choose a valid move for the
    /// given `Player`. If the strategy cannot recommend a `Move`, `None`
    /// will be returned.  If there is no possible move for the player,
    /// `Some(None)` will be returned.
    // TODO Would there be value in moving to using a Result instead of Option here?
    // TODO use contracts crate or similar to validate the post-condition?
    //   Perhaps consider contracts or secrust?
    fn choose<'l>(
        &mut self,
        config: Self::Config<'l>,
        state: &G::State,
        player: &G::Player,
    ) -> Option<Option<G::Move>>;
}

//////////////////////////////////////////////////////////////////////////////////////////////////

/// `FailureStrategy` is the trivial `Strategy` that will just give up and
/// propose no `Move`.
#[derive(Default)]
pub struct FailureStrategy<G: Game> {
    marker: PhantomData<G>,
}

impl<G: Game> Debug for FailureStrategy<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FailureStrategy").finish()
    }
}

impl<G: Game> FailureStrategy<G> {
    pub fn new() -> Self {
        Self {
            marker: PhantomData,
        }
    }
}

impl<G: Game> Strategy<G> for FailureStrategy<G> {
    /// `FailureStrategy` requires no configuration information.
    type Config<'l> = ();

    fn choose<'l>(
        &mut self,
        _config: Self::Config<'l>,
        _state: &G::State,
        _player: &G::Player,
    ) -> Option<Option<G::Move>> {
        None
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////

/// `ConstantStrategy` is a `Strategy` that will always select make the same
/// choice of move for a given `Player`.  Note that care must be taken when
/// using `ConstantStrategy` as constructing one correctly generally requires
/// possessing some additional knowledge about the state of the game and how the
/// resulting instance will be used.
pub struct ConstantStrategy<G: Game> {
    map: HashMap<G::Player, Option<G::Move>>,
}

impl<G: Game> Debug for ConstantStrategy<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConstantStrategy")
            .field("map", &self.map)
            .finish()
    }
}

impl<G: Game> ConstantStrategy<G> {
    pub fn failure() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn new(player: G::Player, choice: Option<G::Move>) -> Self {
        Self {
            map: HashMap::from([(player, choice)]),
        }
    }

    pub fn from(map: HashMap<G::Player, Option<G::Move>>) -> Self {
        Self { map }
    }
}

impl<G: Game> Strategy<G> for ConstantStrategy<G> {
    type Config<'l> = ();
    fn choose<'l>(
        &mut self,
        _config: Self::Config<'l>,
        state: &G::State,
        player: &G::Player,
    ) -> Option<Option<G::Move>> {
        let choice = self.map.get(&player).cloned();

        // Verify that the choice is valid for this state.
        assert!(
            {
                let moves: Vec<G::Move> = state.moves(player).collect();
                // No selected move.
                choice.is_none()
                    // No moves available.
                    || matches!(choice.clone(), Some(None) if moves.is_empty())
                    // Move is a valid choice.
                    || matches!(choice.clone(), Some(Some(m)) if moves.iter().find(|n|
                        **n == m).is_some())
            },
            "ConstantStrategy was constructed with an invalid move choice, {choice:?}, for player \
             {player:?}."
        );

        choice
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////

/// `TryStrategy` will start by using an initial `Strategy` implementation to
/// try to find a suitable `Move` for the given `Game` state and player.  If no
/// `Move` is chosen, it will then make use of a fallback `Strategy` to make the
/// choice instead.
pub struct TryStrategy<G: Game, S1: Strategy<G>, S2: Strategy<G>> {
    initial: S1,
    fallback: S2,
    marker: PhantomData<G>,
}

impl<G: Game, S1: Strategy<G>, S2: Strategy<G>> Debug for TryStrategy<G, S1, S2> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TryStrategy")
            .field("initial", &self.initial)
            .field("fallback", &self.fallback)
            .finish()
    }
}

impl<G: Game, S1: Strategy<G>, S2: Strategy<G>> TryStrategy<G, S1, S2> {
    pub fn new(initial: S1, fallback: S2) -> Self {
        Self {
            initial,
            fallback,
            marker: PhantomData,
        }
    }
}

impl<G: Game, S1: Strategy<G>, S2: Strategy<G>> Strategy<G> for TryStrategy<G, S1, S2> {
    /// `TryStrategy` uses the combination of the configuration data from `S1`
    /// and `S2`.
    type Config<'l> = (S1::Config<'l>, S2::Config<'l>);

    fn choose<'l>(
        &mut self,
        config: Self::Config<'l>,
        state: &G::State,
        player: &G::Player,
    ) -> Option<Option<G::Move>> {
        self.initial
            .choose(config.0, state, player)
            .or_else(|| self.fallback.choose(config.1, state, player))
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////

/// `FirstMoveStrategy` always selects the first available legal `Move`, if one exists.
/// Otherwise, it will return `Some(None)`.
#[derive(Default)]
pub struct FirstMoveStrategy<G: Game> {
    marker: PhantomData<G>,
}

impl<G: Game> FirstMoveStrategy<G> {
    pub fn new() -> Self {
        Self { marker: PhantomData }
    }
}

impl<G: Game> Debug for FirstMoveStrategy<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FirstMoveStrategy").finish()
    }
}

impl<G: Game> Strategy<G> for FirstMoveStrategy<G> {
    type Config<'l> = ();

    fn choose<'l>(
        &mut self,
        _config: Self::Config<'l>,
        state: &G::State,
        player: &G::Player,
    ) -> Option<Option<G::Move>> {
        Some(state.moves(player).next())
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////

/// A completely random strategy for a game.  This will generally not be
/// a viable strategy for most games, but it can be useful for testing
/// purposes.  It is guaranteed to always provide some choice of `Move`,
/// assuming there are any.
pub struct RandomStrategy<G: Game> {
    seed: u64,
    // A phantom type to associate with the game type, as `RandomStrategy`
    // does not need to store game specific data.
    marker: PhantomData<G>,
}

impl<G: Game> Debug for RandomStrategy<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RandomStrategy")
            .field("seed", &self.seed)
            .finish()
    }
}

impl<G: Game> RandomStrategy<G> {
    /// Create a new random strategy from the given seed.
    pub fn new(seed: u64) -> Self {
        RandomStrategy {
            seed,
            marker: PhantomData,
        }
    }

    /// Obtain the seed use to initialize this strategy.
    pub fn seed(&self) -> u64 {
        self.seed
    }
}

impl<G: Game> Default for RandomStrategy<G> {
    fn default() -> Self {
        let mut rng = rand::rng();
        RandomStrategy::new(rng.random::<u64>())
    }
}

impl<G: Game> Strategy<G> for RandomStrategy<G> {
    type Config<'l> = ();

    fn choose<'l>(
        &mut self,
        _config: Self::Config<'l>,
        state: &G::State,
        player: &G::Player,
    ) -> Option<Option<G::Move>> {
        let mut hasher = DefaultHasher::new();
        state.hash(&mut hasher);
        // Use ChaCha random number generator for forward compatibility.
        let mut rng = ChaCha20Rng::seed_from_u64(self.seed + hasher.finish());

        // TODO Might be a more efficient option for sampling the moves,
        //   but ChaChaRng does not implement IteratorRandom.
        let moves: Vec<G::Move> = state.moves(player).collect();
        // No valid moves available.
        if moves.is_empty() {
            return Some(None);
        }
        // Randomly select one of the valid moves.
        let index = (rng.as_rngcore().next_u64() as usize) % moves.len();
        Some(moves.get(index).cloned())
    }
}
