# Endgame Rock, Paper, Scissors

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

The `rps` crate provides an instantiation of the abstractions provided by the
`endgame_ludic` crate for the classic game
[Rock, Paper, Scissors](https://en.wikipedia.org/wiki/Rock_paper_scissors). Rock, Paper, Scissors
is probably the simplest "interesting" turn-based game where the players make moves simultaneously.
As such, it is a useful test case for the endgame library's functionality. It is currently
parameterizable by the number of rounds, where the winner is whichever plays wins the majority of
the rounds.

## Future work

* The [Iterated Prisoner's Dilemma](https://en.wikipedia.org/wiki/Prisoner's_dilemma) is perhaps
  just a little simpler game with simultaneous moves and I should add an implementation as well.
* It would be useful to also allow parameterizing the game by the number of players.
* It might also be interesting to allow configuring unbalanced games where some choices can defeat
  more or fewer opposing choices.