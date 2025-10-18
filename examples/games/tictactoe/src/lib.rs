use endgame_direction::Direction;
use endgame_grid::shape::{HashShapeContainer, HashShapeContainerIterator};
use endgame_grid::square;
use endgame_grid::{Coord, DirectionType, ShapeContainer};
use endgame_ludic::game;
use endgame_ludic::payoffs::Payoffs;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt::Display;

//////////////////////////////////////////////////////////////////////////////////////////////////

// TODO Generalize to additional players for testing?
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Debug, Serialize, Deserialize)]
pub enum Player {
    X,
    O,
}

impl Player {
    pub fn as_str(&self) -> &str {
        use Player::*;
        match self {
            X => "X",
            O => "O",
        }
    }
}

impl Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// TODO Create trait for Player in endgame_ludic?
impl Player {
    pub fn next(self) -> Self {
        use Player::*;
        match self {
            X => O,
            O => X,
        }
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub struct Move(pub square::Coord);

//////////////////////////////////////////////////////////////////////////////////////////////////

pub struct MoveIterator<'l> {
    state: &'l State,
    iter: HashShapeContainerIterator<'l, square::Coord, Option<Player>>,
}

impl<'l> Iterator for MoveIterator<'l> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if <State as game::State<Game>>::is_over(self.state) {
            return None;
        }
        // Iterate over the grid until we find an empty square.
        while let Some((coord, opt_player)) = self.iter.next() {
            match opt_player {
                // If the square is occupied, continue to the next square.
                Some(_) => continue,
                // If the square is empty, return the move.
                None => return Some(Move(*coord)),
            }
        }
        None
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub struct State {
    /// The size of the game board.
    size: usize,
    /// Keeping track of turns in the struct is not strictly necessary, as we can extract that
    /// from the board.  But it simplifies some computations.
    turns: usize,
    /// The player making the next move.
    player: Player,
    /// The state of the board.
    board: HashShapeContainer<square::Coord, Option<Player>>,
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in 0..self.size {
            for col in 0..self.size {
                match self.board.get(&square::Coord::new(col as i32, row as i32)) {
                    Some(Some(p)) => f.write_str(p.as_str())?,
                    _ => f.write_str(".")?,
                };
            }
            f.write_str("\n")?;
        }
        Ok(())
    }
}

impl State {
    /// Construct a new `State` for the given size game board.  The size must be at least 1.
    fn new(size: usize) -> Self {
        assert!(size > 0, "The board must not be zero sized.");

        // TODO Annoying that we cannot use `range` function for this.  Look into adding
        //   a shape creation function for this case.
        let mut board = HashShapeContainer::new();
        for x in 0..size {
            for y in 0..size {
                board.insert(square::Coord::new(x as i32, y as i32), None);
            }
        }
        Self {
            size,
            turns: 0,
            // Player X always starts first.
            player: Player::X,
            board,
        }
    }

    /// Check to see if the given `Player` has won.
    fn winner(&self, player: Player) -> bool {
        let check_line = |x: usize, y: usize, dir_type: DirectionType, dir: Direction| {
            square::Coord::new(x as i32, y as i32)
                .direction_iterator(dir_type, dir, ..self.size)
                .take_while(|c| self.board.get(c) == Some(&Some(player)))
                .count()
                >= self.size
        };

        // Check all columns
        (0..self.size).any(|col| {
            check_line(col, 0, DirectionType::Face, Direction::North)
        }) ||
            // Check all rows
            (0..self.size).any(|row| {
                check_line(0, row, DirectionType::Face, Direction::East)
            }) ||
            // Check the upper-left to lower-right diagonal
            check_line(0, 0, DirectionType::Vertex, Direction::NorthEast)
            ||
            // Check the lower-left to upper-right diagonal
            check_line(self.size, 0, DirectionType::Vertex, Direction::NorthWest)
    }

    pub fn board(&self) -> &HashShapeContainer<square::Coord, Option<Player>> {
        &self.board
    }
}

impl game::State<Game> for State {
    fn current_players(&self) -> HashSet<Player> {
        HashSet::from([self.player])
    }

    fn is_over(&self) -> bool {
        use Player::*;
        // The game is over if one of the players won.
        self.winner(X) ||
            self.winner(O) ||
            // If all positions are occupied, the game is also over.
            self.board.iter()
                .filter(|(_, v)| v.is_none()).count() == 0
    }

    fn moves(&self, player: &Player) -> MoveIterator<'_> {
        MoveIterator {
            state: &self,
            // If the provided player is not the current player, return an empty iterator.
            iter: if self.player == *player {
                self.board.iter()
            } else {
                HashShapeContainerIterator::empty()
            },
        }
    }

    fn next(&self, moves: &HashMap<Player, Move>) -> Option<Self> {
        // If the game is over or an incorrect number of moves have been provided,
        // return None.
        if self.is_over() || moves.len() > 1 || moves.is_empty() {
            return None;
        }

        // Obtain the move for the current player.  If there is no move,
        // for the current player, return None.
        let Some(m) = moves.get(&self.player) else {
            return None;
        };
        // If the coordinate for this move is already occupied, return None.
        if matches!(self.board.get(&m.0), Some(Some(_))) {
            return None;
        }

        let mut new_board = self.board.clone();
        let old_contents = new_board.insert(m.0, Some(self.player));
        assert!(old_contents.is_some(), "Square must be in the board");
        assert!(old_contents.unwrap().is_none(), "Square is already occupied");
        Some(State {
            size: self.size,
            turns: self.turns + 1,
            player: self.player.next(),
            board: new_board,
        })
    }

    fn payoffs(&self) -> Payoffs<Game> {
        use Player::*;
        // To encourage not just completely giving up, we adjust the score
        // based upon the number of turns.  Winning in fewer turns yields
        // a better score, while losing in more turns is better.
        let max_moves = self.size * self.size;
        let win_score = (1 + max_moves - self.turns) as f64 / max_moves as f64;
        let lose_score = -win_score;
        let (x_payoff, o_payoff) = if self.winner(X) {
            (OrderedFloat(win_score), OrderedFloat(lose_score))
        } else if self.winner(O) {
            (OrderedFloat(lose_score), OrderedFloat(win_score))
        } else {
            (OrderedFloat(0.0), OrderedFloat(0.0))
        };

        Payoffs::from_slice(&[(X, x_payoff), (O, o_payoff)])
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Config {
    /// The size of the game board.  This must be at least one.
    pub size: usize,
}

impl Default for Config {
    fn default() -> Self {
        // Default to the traditional 3x3 game.
        Self { size: 3 }
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default)]
pub struct Game {
    config: Config,
}

impl game::Game for Game {
    type Player = Player;

    type Move = Move;

    type MoveIterator<'l> = MoveIterator<'l>;

    type State = State;

    type Config = Config;

    fn name() -> String {
        "Tic-Tac-Toe".to_string()
    }

    fn new(config: &Self::Config) -> Self {
        assert!(config.size > 0);
        Self { config: config.clone() }
    }

    fn players(&self) -> HashSet<Player> {
        use Player::*;
        HashSet::from([X, O])
    }

    fn start(&self) -> State {
        State::new(self.config.size)
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////
