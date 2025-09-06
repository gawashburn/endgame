# Endgame Grid

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
<a href="https://crates.io/crates/endgame_grid"><img src="https://img.shields.io/crates/v/endgame_grid?style=flat-square" alt="Crates.io version" /></a>

The `endgame_grid` crate provides functionality for working grid systems.
It currently has support for square, hexagonal, and triangular grids.

For example of the functionality this crate provides, see the online
[grid demo](https://gawashburn.github.io/endgame_grid_demo/).

## Future work

* Possibly add support for pointy top hexagonal grids, instead of just flat top.
  While this might be a useful convenience in some cases, in practice users of
  the library could just rotate their grids by 90 degrees when displaying them.
* Add support for Rust's `Step` trait once it is stabilized.
* The path algorithm for square grids is counterintuitively more complex than
  that required for hexagonal grids. Is there a better rounding strategy that
  would resolve the ambiguity when the linear interpolated values pass through
  grid vertices?
* The path algorithm for triangular grids currently is relatively expensive,
  as it relies on a conversion to screen space to determine the current error.
  Additionally, in some cases it will produce correct paths that are visually
  suboptimal. Like the square grid path algorithm, this tends to happen when
  interpolated values pass through grid vertices.
* The algorithms for tessellating rectangles with hexagonal and triangular grids
  are a bit complex and could potentially be optimized further. If nothing
  else, with some additional effort it should be possible to avoid the need to
  check intersection with the rectangle for obviously interior grid cells.
  are currently somewhat simplistic. There is likely room for improvement.
* Currenly, the support for `Shape` and `ShapeContainer` is relatively limited.
* At present the only implementations of the `Shape` and `ShapeContainer`
  traits is via `HashShape` and `HashShapeContainer`. It should be possible to
  provide implementations optimized for specific grid types.
* On triangular grid, a triangle will touch the vertex of nine other triangles.
  Currently, the vertex directions for a triangle will only allow for traversing
  to three of these. The `endgame_direction` crate only supports the usual
  cardinal and ordinal directions, which is insufficient to provide directions
  for all nine triangles. One solution would be to extend `endgame_direction`
  to also provide the secondary intercardinal directions
  (West-northwest, etc.).
* At present the focus in `endgame_grid` has been on grid cells. It ought
  to be possible extend it with better support for working with grid vertices
  and edges.
* The trait interfaces aim to be quite general, so it should be possible to add
  support for more exotic grids, for example, perhaps octagons and squares.