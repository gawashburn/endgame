use crate::game::{Game, State};
use itertools::Itertools;
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

    /// Given a state of the `Game`, attempt to choose a valid mo ve for the
    /// given `Player`. If the strategy cannot recommend a `Move`, `None`
    /// wil be returned.  If there is no possible move for the player,
    /// `Some(None)` will be returned.
    // TODO Would there be value in moving to using a Result instead of Option here?
    // TODO use contracts crate or similar to validate the post-condition?
    //   Prehaps consider contracts or secrust?
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
}

impl<G: Game> Strategy<G> for ConstantStrategy<G> {
    type Config<'l> = ();
    fn choose<'l>(
        &mut self,
        _config: Self::Config<'l>,
        _state: &G::State,
        player: &G::Player,
    ) -> Option<Option<G::Move>> {
        // TODO Validate that move is acceptable for the current state?
        self.map.get(&player).cloned()
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

/// A completely random strategy for a game.  This will generally not be
/// a viable strategy for most games, but it can be useful for testing
/// purposes.  It is guaranteed to always provide some choice of `Move`,
/// assuming there are any.
pub struct RandomStrategy<G: Game> {
    seed: u64,
    // Phantom type to associate with the game type, as `RandomStrategy`
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

//////////////////////////////////////////////////////////////////////////////////////////////////

/*

/// `DFSStrategy` uses a depth-first search to explore the game graph and find the optimal move
/// for each player for a give game state.  Except for games with extremely small state spaces,
/// this strategy will generally not be practical but can be useful for testing purposes.
///
/// This implementation does not require that the graph of game states be acyclic.  A cycle
/// would manifest as a "back pointer" on the search stack.  As such, its best solution
/// should just be the best solution for all the other possible moves from that state.
///
/// This strategy considers "optimal" to mean the move that will lead to the highest payoff
/// for a given player.  However, that payoff can only be guaranteed if there are no other
/// players that can move from that state.
pub struct DFSStrategy<G: Game> {
    /// A map of games stats
    edges: HashMap<G::State, HashMap<G::Player, G::Move>, G::State)>,
    back_edges: HashMap<G::State, (HashMap<G::Player, G::Move>, G::State)>,

    /// Map from a game states to the current best `Solution` for that state.
    solutions: HashMap<G::State, Solution<G>>,
    stack: Vec<G::State>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum SolutionState {
    /// The state has not been visited and the solution is not yet complete.
    #[default]
    Unvisited,
    /// The solution for this state is complete.
    Complete,
    /// This state is degenerate no terminal solution state exists.
    Unwinnable,
}

/// A `Solution` is a payoff along with an optional map of moves that will transition
/// the current state one step closer to a state that yields the associated
/// payoff. If there is no move then this `Solution` corresponds to a terminal
/// state.
#[derive(Clone, Debug)]
struct Solution<G: Game> {
    /// The payoffs associated with this solution.
    payoffs: Payoffs<G>,
    /// None if this is a terminal state, otherwise the map of optimal moves
    /// for each player
    opt_moves: Option<HashMap<G::Player, G::Move>>,
    /// THe urrent state of this solution.
    state: SolutionState,
}

impl<G: Game> Solution<G> {
    /// Construct a new, incomplete solution with the given payoffs.
    fn terminal(payoffs: Payoffs<G>) -> Self {
        Self {
            payoffs,
            opt_moves: None,
            completed: true,
        }
    }
}

impl<G: Game> Default for Solution<G> {
    fn default() -> Self {
        Self {
            payoffs: Payoffs::default(),
            opt_moves: None,
            completed: false,
        }
    }
}


impl<G: Game> DFSStrategy<G> {
    /// Construct a new DFSPlayer
    fn new() -> Self {
        DFSStrategy {
            solutions: HashMap::new(),
            stack: Vec::new(),
        }
    }
}

impl<G: Game> Strategy<G> for DFSStrategy<G> {
    /// Choose a "best" move for the given player in the given State,
    /// if one exists.  For example, there will be no move if the
    /// game is over for the current player.
    fn choose(&mut self, state: &G::State, player: G::Player) -> Option<G::Move> {
        // Verify that the stack is empty when starting.
        assert!(self.stack.is_empty());

        self.stack.push(state.clone());

        // A little annoying that we have to clone the state here, but it isn't entirely
        // in vain as we move it into the solutions map if all its children have
        // been solved.
        while let Some(s) = self.stack.last().map(ToOwned::to_owned) {
            // If this state already has completed solution we can pop the stack and continue.
            if matches!(self.solutions.get(&s), Some(solution) if solution.completed) {
                self.stack.pop();
                continue;
            }

            // If the game is over, store the payoff, pop, and continue.
            if s.is_over() {
                self.solutions.insert(
                    s.clone(),
                    Solution::terminal(s.payoffs()),
                );
                self.stack.pop();
                continue;
            }

            // Otherwise, search the child states.

            // Multi-way cross-product of the current players' moves.
            let vec_moves: Vec<Vec<(G::Player, G::Move)>> = s
                .current_players()
                .into_iter()
                .map(|p| std::iter::repeat(p.clone()).zip(s.moves(p)).collect())
                .collect();
            let moves: Vec<HashMap<G::Player, G::Move>> = vec_moves
                .into_iter()
                .map(|v| v.into_iter())
                .multi_cartesian_product()
                .map(|v| HashMap::from_iter(v.into_iter()))
                .collect();

            let mut all_visited = true;
            let mut child_states = HashMap::new();
            for mm in moves {
                // All moves should be valid, but play it safe.
                if let Some(child) = s.next(&mm) {
                    // If the child hasn't been solved, add it to the stack.
                    // is solved add the solution to the vector,
                    // otherwise push it on the stack if it has not already
                    // been visited.
                    if !self.solutions.contains_key(&child) {
                        all_visited = false;
                        self.solutions.insert(child.clone(), Solution::default());
                        self.stack.push(child);
                    } else {
                        child_states.insert(mm, child);
                    }
                } else {
                    unreachable!("Invalid move encountered: {mm:?}")
                }
            }

            // If all the children have already been visited we can stop
            // the search at this node.
            if all_visited {
                let mut child_solutions = HashMap::new();
                for (mm, child) in child_states {
                    if let Some(solution) = self.solutions.get(&child) {
                        child_solutions.insert(mm, solution.clone());
                    } else {
                        unreachable!("Child state should have a solution");
                    }
                }


                // If this isn't a terminal state, but all children are
                // all children have been visited, there is no solution for it
                // so just pop and continue as any other choice would lead
                // to cycles.
                if (child_solutions.is_empty()) {
                    self.stack.pop();
                    continue;
                }

                // Sort the solutions to find the one with maximum score for
                // the selected player.
                child_solutions.sort_by(|s1, s2| {
                    let p1 = s1.payoffs.payoff(&player);
                    let p2 = s2.payoffs.payoff(&player);
                    p2.cmp(&p1)
                });

                // After sorting, the first solution will be the best for the selected player.
                self.solutions.insert(s, child_solutions.swap_remove(0));
                // Continue and we'll pop next time around.
                continue;
            }
        }

        // Return the solution move for the state, if there is one.
        //   TODO Can the number of clones be reduced here?
        self.solutions
            .get(state)
            .and_then(|s| s.opt_moves.clone().and_then(|mm| mm.get(&player).cloned()))
    }
}

 */
