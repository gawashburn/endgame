use crate::game::Game;
use ordered_float::OrderedFloat;
use std::collections::{HashMap, HashSet};


//////////////////////////////////////////////////////////////////////////////////////////////////

/// An individual player's "payoff" is represented as an ordered floating
/// point number.  It is an abstract metric of their performance in a game.
/// A value of zero indicates that the game was a draw. Negative values
/// indicate a loss.  Positive values indicate a win.  The magnitude of the
/// number may be used to indicate the "quality" of the win.  For example, we
/// could consider giving a player a higher score if they won by taking more
/// pieces or territory.  Or for certain games, solving it in fewer steps
/// would yield a higher score.
pub type Payoff = OrderedFloat<f64>;

/// Payoffs are a vector of individual payoffs. The length of will be dependent
/// on the number of players.  As players must be convertible to an index, that
/// index corresponds to the location in vector for that player's payoff.
#[derive(Debug, Clone)]
pub struct Payoffs<G: Game> {
    payoffs: HashMap<G::Player, Payoff>,
}

impl<G: Game> Payoffs<G> {
    /// TODO
    pub fn from_players(players: HashSet<G::Player>) -> Self {
        Self {
            payoffs: players
                .into_iter()
                .map(|player| (player, OrderedFloat(0.0)))
                .collect(),
        }
    }

    /// TODO
    pub fn from_map(payoffs: HashMap<G::Player, Payoff>) -> Self {
        Self { payoffs }
    }

    /// TODO
    pub fn from_slice(slice: &[(G::Player, Payoff)]) -> Self {
        Self {
            payoffs: HashMap::from_iter(slice.iter().cloned()),
        }
    }

    /// TODO

    pub fn payoff(&self, player: &G::Player) -> Option<&Payoff> {
        self.payoffs.get(player)
    }

    pub fn iter(&self) -> impl Iterator<Item=(&G::Player, &Payoff)> {
        let mut payoffs = self.payoffs.iter().collect::<Vec<_>>();
        payoffs.sort_by(|(p1, _), (p2, _)| p1.cmp(p2));
        payoffs.into_iter()
    }
}

// TODO
/*
impl<G: Game> IntoIterator for Payoffs<G> {
    type Item = (G::Player, Payoff);
    type IntoIter = std::vec::IntoIter<(&G::Player, &Payoff)>;

    fn into_iter(self) -> Self::IntoIter {
        let mut payoffs = self.payoffs.iter().collect::<Vec<_>>();
        payoffs.sort_by(|(p1, _), (p2, _)| p1.cmp(p2));
        payoffs.into_iter()
    }
}
*/

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
        for (player, payoff) in other.payoffs {
            // If the player already exists, add to their payoff.
            // Otherwise, insert a new entry.
            self.payoffs
                // TODO Require Player to implement Copy?
                .entry(player.clone())
                .and_modify(|e| *e += *payoff)
                .or_insert(payoff);
        }
    }
}

// TODO Why?
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
