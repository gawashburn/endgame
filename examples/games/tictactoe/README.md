# Endgame Tic-Tac-Toe

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

The `tictactoe` crate provides an instantiation of the abstractions provided by the
`endgame_ludic` crate for the classic game [Tic-tac-toe](https://en.wikipedia.org/wiki/Tic-tac-toe).
Tic-tac-toe is probably the simplest "interesting" turn-based game where the players alternate
their moves. As such, it is a useful test case for the endgame library's functionality. It is
currently parameterizable by the size of the game board.

## Future work

* In addition to parameterizing the game board size, it would be useful to allow parameterizing by
  the number players.
* Support changing the underlying grid cell kind.