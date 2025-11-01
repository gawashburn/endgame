# Endgame Game

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
<a href="https://crates.io/crates/endgame_direction"><img src="https://img.shields.io/crates/v/endgame?style=flat-square" alt="Crates.io version" /></a>

This somewhat redundantly named crate provides an abstraction for representing
turn based games.

## Future work

* Extend `game::State` with an optional method for producing a set of game
  states along with isomorphisms between them and the current state. This would
  be useful in search space reduction in the case that there are various
  symmetries in the game that could be exploited.
* Add additional primitive `Strategy`s.
* Consider adding game combinators. Such as parallel or sequential composition
  of games. While perhaps an interesting exercise, whether these would be of
  general utility beyond computational game theory is questionable.
  