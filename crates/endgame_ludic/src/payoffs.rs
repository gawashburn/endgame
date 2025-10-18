use crate::game::Game;
use ordered_float::OrderedFloat;
use std::collections::{HashMap, HashSet};


//////////////////////////////////////////////////////////////////////////////////////////////////

/// An individual player's "payoff" is represented as an ordered floating point number.  It is an
/// abstract metric of their performance in a game.  A value of zero indicates that the game was
/// a draw. Negative values indicate a loss.  Positive values indicate a win.  The magnitude of
/// the number may be used to indicate the "quality" of the win.  For example, we could consider
/// giving a player a higher score if they won by taking more pieces or territory.  Or for certain
/// games, solving it in fewer steps would yield a higher score.
pub type Payoff = OrderedFloat<f64>;

/// `Payoffs` are just a map of individual `Payoff`s. 
#[derive(Debug, Clone, Default)]
pub struct Payoffs<G: Game> {
    payoffs: HashMap<G::Player, Payoff>,
}

impl<G: Game> Payoffs<G> {
    /// Construct `Payoffs` corresponding to a draw from a `HashSet` of `Player`s.  In most
    /// cases this should be the complete set of `Player`s for a given instance of `G`.
    pub fn from_players(players: HashSet<G::Player>) -> Self {
        Self {
            payoffs: players
                .into_iter()
                .map(|player| (player, OrderedFloat(0.0)))
                .collect(),
        }
    }

    /// Construct `Payoffs` from a `HashMaps` of `Player`s and `Payoff`s.  In most, cases this
    /// `HashMap` should completely cover the set of `Player`s for a given instance of `G`.
    pub fn from_map(payoffs: HashMap<G::Player, Payoff>) -> Self {
        Self { payoffs }
    }

    /// Construct `Payoffs` from a slice of `Player`s and `Payoff`s.  In most, cases this should
    /// completely cover the set of `Player`s for a given instance of `G`.
    pub fn from_slice(slice: &[(G::Player, Payoff)]) -> Self {
        Self {
            payoffs: HashMap::from_iter(slice.iter().cloned()),
        }
    }

    /// Obtain the `Payoff` for a given `Player`, if there is one.
    pub fn payoff(&self, player: &G::Player) -> Option<&Payoff> {
        self.payoffs.get(player)
    }

    /// Returns an iterator over the `Player`s and `Payoff`s in this `Payoffs`.
    pub fn iter(&self) -> impl Iterator<Item=(&G::Player, &Payoff)> {
        let mut payoffs = self.payoffs.iter().collect::<Vec<_>>();
        payoffs.sort_by(|(p1, _), (p2, _)| p1.cmp(p2));
        payoffs.into_iter()
    }
}

impl<G: Game> std::ops::Add for Payoffs<G> {
    type Output = Payoffs<G>;

    fn add(self, other: Self) -> Self {
        let mut result = self.clone();
        result += other;
        result
    }
}

impl<G: Game> std::ops::AddAssign for Payoffs<G> {
    fn add_assign(&mut self, other: Self) {
        self.add_assign(&other);
    }
}

impl<G: Game> std::ops::AddAssign<&Payoffs<G>> for Payoffs<G> {
    fn add_assign(&mut self, other: &Self) {
        for (player, payoff) in other.payoffs.iter() {
            // If the player already exists, add to their payoff.
            // Otherwise, insert a new entry.
            self.payoffs
                // TODO Require Player to implement Copy?
                .entry(player.clone())
                .and_modify(|e| *e += *payoff)
                .or_insert(*payoff);
        }
    }
}
