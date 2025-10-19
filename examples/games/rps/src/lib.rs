use endgame_ludic::game;
use endgame_ludic::payoffs::Payoffs;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::hash::Hash;

//////////////////////////////////////////////////////////////////////////////////////////////////
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Debug, Serialize, Deserialize)]
pub enum Player {
    A,
    B,
}
const ALL_PLAYERS: [Player; 2] = [Player::A, Player::B];

impl Player {
    pub fn as_str(&self) -> &str {
        use Player::*;
        match self {
            A => "A",
            B => "B",
        }
    }
}

impl Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum Move {
    Rock,
    Paper,
    Scissors,
}

impl Move {
    pub fn as_str(&self) -> &str {
        use Move::*;
        match self {
            Rock => "Rock",
            Paper => "Paper",
            Scissors => "Scissors",
        }
    }

    /// Return the move that strictly beats the given move.
    /// Rock is beaten by Paper, Paper is beaten by Scissors, and Scissors is
    /// beaten by Rock.
    fn beating_move(m: Move) -> Move {
        use Move::*;
        match m {
            Rock => Paper,
            Paper => Scissors,
            Scissors => Rock,
        }
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
const ALL_MOVES: [Move; 3] = [Move::Rock, Move::Paper, Move::Scissors];

//////////////////////////////////////////////////////////////////////////////////////////////////
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct State {
    rounds: usize,
    turn: usize,
    moves: HashMap<Player, Vec<Move>>,
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..self.turn {
            writeln!(
                f,
                "Turn {}: Player A played {}, Player B played {}",
                i + 1,
                self.player_moves(Player::A)[i].as_str(),
                self.player_moves(Player::B)[i].as_str(),
            )?;
        }
        Ok(())
    }
}

impl Hash for State {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.rounds.hash(state);
        self.turn.hash(state);
        self.player_moves(Player::A).hash(state);
        self.player_moves(Player::B).hash(state);
    }
}

impl State {
    /// Create a new `State` with the given number of rounds.  The game must be
    /// at least one round long.
    pub fn new(rounds: usize) -> Self {
        assert!(
            rounds > 0,
            "The game requires that at least one round be played."
        );

        Self {
            rounds,
            turn: 0,
            moves: HashMap::from([(Player::A, vec![]), (Player::B, vec![])]),
        }
    }

    /// Helper for accessing the moves of the given player.
    fn player_moves(&self, player: Player) -> &Vec<Move> {
        self.moves
            .get(&player)
            .expect("All players should have entries.")
    }

    /// Compute the number of wins for players A and B across the completed
    /// turns. Returns a tuple (a_wins, b_wins).
    pub fn wins(&self) -> (usize, usize) {
        let mut a_wins = 0;
        let mut b_wins = 0;
        for (a_move, b_move) in self
            .player_moves(Player::A)
            .iter()
            .zip(self.player_moves(Player::B).iter())
        {
            if Move::beating_move(*a_move) == *b_move {
                // B's move beats A's.
                b_wins += 1;
            } else if Move::beating_move(*b_move) == *a_move {
                // A's move beats B's.
                a_wins += 1;
            } // Otherwise a draw
        }
        (a_wins, b_wins)
    }
}

impl game::State<Game> for State {
    fn current_players(&self) -> HashSet<Player> {
        if !self.is_over() { HashSet::from(ALL_PLAYERS) } else { HashSet::new() }
    }

    fn is_over(&self) -> bool {
        // End the game early if either player has already won a strict majority of the
        // configured rounds (best-of rounds). Also end if the maximum number of rounds
        // has been played.
        let majority = (self.rounds / 2) + 1;
        let (a_wins, b_wins) = self.wins();
        (a_wins >= majority) || (b_wins >= majority) || (self.turn >= self.rounds)
    }

    fn moves(&self, _player: &Player) -> core::array::IntoIter<Move, 3> {
        ALL_MOVES.into_iter()
    }

    fn next(&self, moves: &HashMap<Player, Move>) -> Option<Self> {
        // The correct number of moves must have been supplied.
        if moves.len() != 2 {
            return None;
        }
        let mut new_state = self.clone();
        let new_moves = &mut new_state.moves;
        for (player, m) in moves.clone() {
            new_moves
                .get_mut(&player)
                .expect("All players should have an entry")
                .push(m);
        }
        new_state.turn += 1;
        Some(new_state)
    }

    fn payoffs(&self) -> Payoffs<Game> {
        let (a_wins, b_wins) = self.wins();
        use Player::*;
        Payoffs::from_slice(&[
            (A, OrderedFloat(a_wins as f64)),
            (B, OrderedFloat(b_wins as f64)),
        ])
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Config {
    /// The number of rounds to play.  Must be at least one.
    // TODO Arguably we could allow zero rounds?  That could be treated as equivalent to a draw.
    pub rounds: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self { rounds: 3 }
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default)]
pub struct Game {
    config: Config,
}

impl game::Game for Game {
    fn name() -> String {
        "Rock, Paper, Scissors".to_string()
    }

    type Player = Player;

    type Move = Move;

    type MoveIterator<'l> = core::array::IntoIter<Move, 3>;

    type State = State;

    type Config = Config;

    fn new(config: &Self::Config) -> Self {
        assert!(config.rounds > 0);
        Self {
            config: config.clone(),
        }
    }

    fn players(&self) -> HashSet<Player> {
        HashSet::from(ALL_PLAYERS)
    }

    fn start(&self) -> State {
        State::new(self.config.rounds)
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////
