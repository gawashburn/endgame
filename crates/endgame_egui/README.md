# Endgame egui

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
<a href="https://crates.io/crates/endgame_egui"><img src="https://img.shields.io/crates/v/endgame_egui?style=flat-square" alt="Crates.io version" /></a>

The `endgame_egui` crate provides helper code for integrating the `endgame`
library with the immediate-mode GUI library `egui`.

It provides functionality such as:

* Drawing labels.
* Drawing straight and arc arrows.
* Drawing hollow arrows, and hollow self arrows.
* Drawing grid cells.
* Drawing grid tesselations within rectangles.
* Drawing shapes.

## Future work

* Some of the helpers in this crate are not particularly specific to
  the `endgame` library. In time, it might make sense to look at splitting
  them out to an independent crate or upstreaming them.
* Overall, the there are a number of improvements in styling that could be
  supported.
* The code is still quite rough in some places and could be better organized,
  documented, and optimized.
* Tests using `egui`'s `egui_kittest` crate could be added.