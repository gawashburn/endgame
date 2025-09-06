# Endgame Direction

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
<a href="https://crates.io/crates/endgame_direction"><img src="https://img.shields.io/crates/v/endgame_direction?style=flat-square" alt="Crates.io version" /></a>

The `endgame_direction` crate provides an implementation of
cardinal and ordinal directions. It can be used independently of the rest of the
Endgame library.

Part of the motivation for creating this crate was that the canonical
`direction` crate bakes in some a coordinates, while the Endgame library
provides it own support for grid coordinate systems.

## Future work

* It could make sense to extend the set of directions to include the
  secondary intercardinal directions (West-northwest, etc.). The
  primary use case for this currently would be to support all possible
  vertex directions on a triangular grid in `endgame_grid`.
* It might make sense to add localized text for the directions names.