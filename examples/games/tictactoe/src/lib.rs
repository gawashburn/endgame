use endgame_direction::Direction;
use endgame_grid::shape::{HashShapeContainer, HashShapeContainerIterator};
use endgame_grid::square;
use endgame_grid::{Coord, DirectionType, ShapeContainer};
use endgame_ludic::game::{Game, State};
use endgame_ludic::payoffs::Payoffs;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt::Display;


// TODO Generalize to additional players for testing?
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Debug, Serialize, Deserialize)]
pub enum TicTacToePlayer {
    X,
    O,
}

impl TicTacToePlayer {
    pub fn as_str(&self) -> &str {
        use TicTacToePlayer::*;
        match self {
            X => "X",
            O => "O",
        }
    }
}

impl Display for TicTacToePlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// Create trait?
impl TicTacToePlayer {
    pub fn next(self) -> Self {
        match self {
            TicTacToePlayer::X => TicTacToePlayer::O,
            TicTacToePlayer::O => TicTacToePlayer::X,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub struct TicTacToeMove<const N: usize>(pub square::Coord);

pub struct MoveIterator<'l, const N: usize> {
    state: &'l TicTacToeState<N>,
    iter: HashShapeContainerIterator<'l, square::Coord, Option<TicTacToePlayer>>,
}

impl<'l, const N: usize> Iterator for MoveIterator<'l, N> {
    type Item = TicTacToeMove<N>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.state.is_over() {
            return None;
        }
        // Iterate over the grid until we find an empty square.
        while let Some((coord, opt_player)) = self.iter.next() {
            match opt_player {
                // If the square is occupied, continue to the next square.
                Some(_) => continue,
                // If the square is empty, return the move.
                None => return Some(TicTacToeMove(*coord)),
            }
        }
        None
    }
}

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub struct TicTacToeState<const N: usize> {
    /// Keeping track of turns in the struct is not strictly necessary,
    /// as we can extract that from the board.  But it makes things
    /// simpler.
    turns: usize,
    /// The player making the next move.
    player: TicTacToePlayer,
    /// The state of the board.
    board: HashShapeContainer<square::Coord, Option<TicTacToePlayer>>,
}

impl<const N: usize> Display for TicTacToeState<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in 0..N {
            for col in 0..N {
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

impl<const N: usize> TicTacToeState<N> {
    /// Check to see if the given player has a win.
    fn winner(&self, player: TicTacToePlayer) -> bool {
        let check_line = |x: usize, y: usize, dir_type: DirectionType, dir: Direction| {
            square::Coord::new(x as i32, y as i32)
                .direction_iterator(dir_type, dir, ..N)
                .take_while(|c| self.board.get(c) == Some(&Some(player)))
                .count()
                >= N
        };

        // Check all columns
        (0..N).any(|col| {
            check_line(col, 0, DirectionType::Face, Direction::North)
        }) ||
            // Check all rows
            (0..N).any(|row| {
                check_line(0, row, DirectionType::Face, Direction::East)
            }) ||
            // Check the upper-left to lower-right diagonal
            check_line(0, 0, DirectionType::Vertex, Direction::NorthEast)
            ||
            // Check the lower-left to upper-right diagonal
            check_line(N, 0, DirectionType::Vertex, Direction::NorthWest)
    }

    pub fn board(&self) -> &HashShapeContainer<square::Coord, Option<TicTacToePlayer>> {
        &self.board
    }
}

impl<const N: usize> State<TicTacToe<N>> for TicTacToeState<N> {
    fn current_players(&self) -> HashSet<TicTacToePlayer> {
        HashSet::from([self.player])
    }

    fn is_over(&self) -> bool {
        use TicTacToePlayer::*;
        // The game is over if one of the players won.
        self.winner(X) ||
            self.winner(O) ||
            // If all positions are occupied the game is also over.
            self.board.iter()
                .filter(|(_, v)| v.is_none()).count() == 0
    }

    fn moves(&self, player: TicTacToePlayer) -> MoveIterator<'_, N> {
        MoveIterator {
            state: &self,
            // If the provided player is not the current player, return an empty iterator.
            iter: if self.player == player {
                self.board.iter()
            } else {
                HashShapeContainerIterator::empty()
            },
        }
    }

    fn next(&self, moves: &HashMap<TicTacToePlayer, TicTacToeMove<N>>) -> Option<Self> {
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

        let mut new_board = self.board.clone();
        let old_contents = new_board.insert(m.0, Some(self.player));
        assert!(old_contents.is_some(), "Square must be in the board");
        assert!(old_contents.unwrap().is_none(), "Square is already occupied");
        Some(TicTacToeState {
            turns: self.turns + 1,
            player: self.player.next(),
            board: new_board,
        })
    }

    fn payoffs(&self) -> Payoffs<TicTacToe<N>> {
        use TicTacToePlayer::*;
        // To encourage not just completely giving up, we adjust the score
        // based upon the number of turns.  Winning in fewer turns yields
        // a better score, while losing in more turns is better.
        let max_moves = N * N;
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

/*
impl<const N: usize> GridState<TicTacToe<N>> for TicTacToeState<N> {
    type CoordData = Option<TicTacToePlayer>;

    type Grid = SquareFiniteGrid<Self::CoordData>;

    fn grid(&self) -> &Self::Grid {
        &self.board
    }
}
 */

#[derive(Clone, Debug)]
pub struct TicTacToe<const N: usize> {}

impl<const N: usize> Game for TicTacToe<N> {
    type Player = TicTacToePlayer;

    type Move = TicTacToeMove<N>;

    type MoveIterator<'l> = MoveIterator<'l, N>;

    type State = TicTacToeState<N>;

    // TODO More TicTacToe options?
    type Config = ();

    fn name() -> String {
        "TicTacToe".to_string()
    }

    fn new(_config: &Self::Config) -> Self {
        Self {}
    }

    fn players(&self) -> HashSet<TicTacToePlayer> {
        HashSet::from([TicTacToePlayer::X, TicTacToePlayer::O])
    }

    /// Create a new initial starting state.
    fn start(&self) -> TicTacToeState<N> {
        // TODO Annoying that we cannot use `range` function for this
        let mut board = HashShapeContainer::new();
        for x in 0..N {
            for y in 0..N {
                board.insert(square::Coord::new(x as i32, y as i32), None);
            }
        }
        TicTacToeState {
            turns: 0,
            // X always starts first.
            player: TicTacToePlayer::X,
            board,
        }
    }
}

/*
impl<const N: usize> GridGame for TicTacToe<N> {
    type GridCoord = Coord;
}
*/

#[test]
fn test_tictactoe() {}
