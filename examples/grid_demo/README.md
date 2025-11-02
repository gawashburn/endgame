# Endgame Grid Demo

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

The `grid_demo` crate provides a simple application that exists for both
pedagogical and testing purposes. During development, there have been
a few occasions where the implementation of `endgame_grid` has been
broken, yet in such a way that despite elaborate coverage and unit
tests the bug was not immediate apparent. To help avoid these mistakes,
it was useful to have a visual illustration of the different functionality
it provides.

For example, some initial implementations of the `path_iterator`
method were technically correct, traversing the expected distance, but
would produce paths that were not following a "line" as directly as they
could be.

A fair amount of credit and inspiration goes to the excellent interactive
visualizations found in [Red Blob Games](https://www.redblobgames.com/)'s
guide to [hexagonal grids](https://www.redblobgames.com/grids/hexagons/).

A [web based version of this demo](https://gawashburn.github.io/endgame/grid_demo/)
is available for experimentation.

## Usage

By virtue of being implemented with `eframe`, there a few different options
for running the demo. It can be compiled and run directly on most platforms.
But it is also possible to run it in a browser. The `trunk` framework seems to
work quite well for this purpose:

```
cargo install --locked trunk
```

## Future work

* While probably more effort has been spent in making this demo visually
  appealing than is truly warranted, I think there is definitely still
  room for improvement.
* The default `egui` label font seems to be missing '∇'.
* Tests using `egui`'s `egui_kittest` crate could be added.