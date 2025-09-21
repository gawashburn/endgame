use crate::game::{Game, State};
use crate::payoffs::Payoffs;
use rand::Rng;
use rand_chacha::ChaCha20Rng;
use rand_core::{CryptoRngCore, SeedableRng};
use std::collections::{HashMap, HashSet};
use std::hash::{DefaultHasher, Hash, Hasher};
//use rand_core::{CryptoRngCore, RngCore, SeedableRng};
use std::marker::PhantomData;
//////////////////////////////////////////////////////////////////////////////////////////////////

/// An abstract representation of a strategy for playing a given game.
pub trait Strategy<G: Game> {
    /// Given a state of the `Game`, choose a valid move for the given `Player`.
    /// TODO use contracts crate or similar to validate the post-condition?
    fn choose(&mut self,
              state: &G::State, player: G::Player) -> Option<G::Move>;
}

//////////////////////////////////////////////////////////////////////////////////////////////////

/// A completely random strategy for a game.  This will generally not be
/// a viable strategy for most games, but it can be useful for testing
/// purposes.
#[derive(Debug, Clone)]
pub struct RandomStrategy<G: Game> {
    seed: u64,
    // Phantom type to associate with the game type, as `RandomStrategy`
    // does not need to store game specific data.
    marker: PhantomData<G>,
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
    fn choose(&mut self, state: &G::State, player: G::Player) -> Option<G::Move> {
        let mut hasher = DefaultHasher::new();
        state.hash(&mut hasher);
        // Use ChaCha random number generator for forward compatibility.
        let mut rng = ChaCha20Rng::seed_from_u64(self.seed + hasher.finish());

        // TODO Might be a more efficient option for sampling the moves,
        //   but ChaChaRng does not implement IteratorRandom.
        let moves: Vec<G::Move> = state.moves(player).collect();
        // No valid moves available.
        if moves.is_empty() {
            return None;
        }
        // Randomly select one of the valid moves.
        let index = (rng.as_rngcore().next_u64() as usize) % moves.len();
        moves.get(index).cloned()
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////

// A solution is a payoff along with an optional Move that will transition a
/// Game State on step closer to a State that yields the associated payoff.
/// If there is no move then this Solution corresponds to a terminal state.
#[derive(Clone, Debug)]
struct Solution<G: Game> {
    payoffs: Payoffs<G>,
    opt_move: HashMap<G::Player, G::Move>,
}

pub struct DFSStrategy<G: Game> {
    /// Map from a Game State to the current best Solution.
    solutions: HashMap<G::State, Solution<G>>,
    visited: HashSet<G::State>,
    stack: Vec<G::State>,
}

impl<G: Game> DFSStrategy<G> {
    /// Construct a new DFSPlayer
    fn new() -> Self {
        DFSStrategy {
            solutions: HashMap::new(),
            visited: HashSet::new(),
            stack: Vec::new(),
        }
    }
}

/*
impl<G: Game> Strategy<G> for DFSStrategy<G> {

    /// Choose a best move for the given player in the given State,
    /// if one exists.  For example, there will be no move if the
    /// game is over for the current player.
    fn choose(&mut self, state: &G::State, player: G::Player) -> Option<G::Move> {
        // Verify that the stack is empty when starting.
        assert!(self.stack.is_empty());

        self.visited.insert(state.clone());
        self.stack.push(state.clone());

        // A little annoying that we have to clone the state here, but
        // it isn't entirely in vain as we move it into the solutions map if
        // all its children have been solved.
        while let Some(s) = self.stack.last().map(ToOwned::to_owned) {
            //s.print();

            // If this state already has a solution we can pop the stack
            // and continue.
            if self.solutions.contains_key(&s) {
                self.stack.pop();
                continue;
            }

            // If the game is over, store the payoff, pop, and continue.
            if s.is_over() {
                self.solutions.insert(
                    s.clone(),
                    Solution {
                        payoffs: s.payoffs(),
                        opt_move: None,
                    },
                );
                self.stack.pop();
                continue;
            }

            // Otherwise, search the child states.
            let mut child_solutions = Vec::new();
            let mut all_visited = true;
            for m in s.moves() {
                // All moves should be valid, but play it safe.
                if let Some(child) = s.next(&m) {
                    /* If the child is solved, add the solution to the vector,
                     * otherwise push it on the stack if it has not already
                     * been visited.
                     */
                    if let Some(solution) = self.solutions.get(&child) {
                        child_solutions.push(Solution {
                            opt_move: Some(m),
                            ..solution.clone()
                        });
                    } else if (!self.visited.contains(&child)) {
                        all_visited = false;
                        self.visited.insert(child.clone());
                        self.stack.push(child.clone());
                    }
                }
            }

            // If all the children have already been visited we can stop
            // the search at this node.
            if all_visited {
                // If this isn't a terminal state, but all children are
                // all children have been visited, there is no solution for it
                // so just pop and continue as any other choice would lead
                // to cycles.
                if (child_solutions.is_empty()) {
                    self.stack.pop();
                    continue;
                }

                let current_player = s.current_player();
                // Sort the solutions to find the one with maximum score for
                // the current player.
                child_solutions.sort_by(|s1, s2| {
                    let p1 = s1.payoffs.payoff(current_player);
                    let p2 = s2.payoffs.payoff(current_player);
                    p2.cmp(p1)
                });

                // After sorting, the first solution will be the best for the
                // current player.
                self.solutions.insert(s, child_solutions.swap_remove(0));
                // Continue and we'll pop next time around.
                continue;
            }
        }

        // Return the solution move for the state, if there is one.
        self.solutions.get(state).and_then(|s| s.opt_move.clone())
    }
}

 */