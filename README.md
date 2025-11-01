# Endgame

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
<img src="https://github.com/gawashburn/endgame/actions/workflows/tests.yml/badge.svg" alt="Testing action" />
<a href='https://coveralls.io/github/gawashburn/endgame?branch=master'><img src='https://coveralls.io/repos/github/gawashburn/endgame/badge.svg?branch=master' alt='Coverage Status' /></a>

The Endgame library is a turn-based game engine, but not a Game Engine. There
plenty of quality Game Engines out there for handling your graphics, audio,
input, networking, and so forth. The Endgame library is instead an engine
and a collection of tools for helping you build out and test your game
mechanics. It could even be paired with a Game Engine to flesh out the
rest of your game.

The Endgame library was originally developed entirely in a private repository.
However, to maintain a high bar for quality, as I open up the code, I am
spending a fair amount of time reviewing, filling in missing functionality, and
testing. As such, only a few of the crates comprising the Endgame library are
currently available. Given my available time, it may take months before I've
worked my way through everything.

At present the most complete crates are `endgame_direction` and `endgame_grid`. The latter has
a [web based demo](https://gawashburn.github.io/endgame_grid_demo/) available for experimentation.

## Table of contents

- [Crates](#crates)
- [Development](#development)
- [What's in a name?](#whats-in-a-name)

## Crates

The endgame library consists of several crates, some of which are
optional or can be used independently.

- [`endgame`](README.md): The main crate, which provides the
  complete functionality of the Endgame library.
- [`endgame_direction`](crates/endgame_direction/README.md): A crate for
  working with cardinal and ordinal directions.
- [`endgame_grid`](crates/endgame_grid/README.md): A crate for working with
  grid systems. Currently, it has support for square, hexagonal, and triangular
  grids. This would appear to be the most comprehensive Rust library for working with all three
  kinds of grids using a common interface, perhaps in any language.
- [`endgame_egui`](crates/endgame_egui/README.md): A crate that provides helpers and integration
  with the `egui` GUI crate.
- [`endgame_ludic`](crates/endgame_ludic/README.md): A crate that provides a high-level abstraction
  for turn-based games.

There are also a number of example crates that demonstrate how to use
the endgame library's functionality:

- [`grid_demo`](examples/grid_demo/README.md): An `eframe` application demonstrating and
  exercising the `endgame_grid` crate.
- [`tictactoe`](examples/games/tictactoe/README.md): An instantiation of the `endgame_ludic` for the
  classic game Tic-Tac-Toe.
- [`rps`](examples/games/rps/README.md): An instantiation of the `endgame_ludic` for the
  classic game Rock, Paper, Scissors.

## Development

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
<a href="https://blog.rust-lang.org/2023/01/10/Rust-1.83.0.html"><img src="https://img.shields.io/badge/rustc-1.83.0+-lightgray.svg" alt="Rust 1.83.0+" /></a>

The Endgame library is written in [Rust](https://www.rust-lang.org/).
I have not yet attempted cross-compilation, but so far the library has been
developed in such that it should work on nearly all platforms.
Some crates make use of [`egui`](https://github.com/emilk/egui) for their user
interface, but it has support for macOS, Windows, Linux, Android and browsers
via WebAssembly.

Pull requests are definitely welcome. I am still a relative Rust novice, so it
also entirely possible there are better or more idiomatic ways to write some of
this code.

I have endeavoured to ensure that Endgame has is thoroughly tested, with
relatively high coverage. Additionally, I have made use
of [cargo-mutants](https://mutants.rs/) to
ensure that the tests are not accidentally vacuous. So please try to add
appropriate tests for submitted changes.

Power metal has also been a key component of development, but there are
currently no tests or actions validating that.

## What's in a name?

I had originally chosen a name for this project that was more directly
influenced by the game I was building this engine to support. But there was
already a crate with that name, as well as libraries in other languages,
and at least one company. I tried some investigation into other thematically
related names, but most wound up being too long or esoteric.

Next I decided to try a sillier name, but it turns out that was already used
by a series of games.

Musing, it struck me, what kind of name was a
<a href="https://godotengine.org/">Godot</a> for a Game Engine? Obviously,
it was a reference to the
play <i><a href="https://en.wikipedia.org/wiki/Waiting_for_Godot">Waiting for
Godot</a></i>
by Samuel Beckett. So I thought, why not another Beckett play? And after a
quick web search, I
found <i><a href="https://en.wikipedia.org/wiki/Endgame_(play)">Endgame</a></i>,
which was so on the nose that I couldn't resist.