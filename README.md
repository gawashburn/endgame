# Endgame

<p>  
<a href="https://crates.io/crates/endgame"><img src="https://img.shields.io/crates/v/endgame?style=flat-square" alt="Crates.io version" /></a>
<img src="https://github.com/gawashburn/endgame/actions/workflows/tests.yml/badge.svg" alt="Testing action" />
<a href='https://coveralls.io/github/gawashburn/endgame?branch=master'><img src='https://coveralls.io/repos/github/gawashburn/endgame/badge.svg?branch=main' alt='Coverage Status' /></a>
<img src="https://img.shields.io/github/license/gawashburn/endgame" alt="MIT License" />
</p>

The Endgame library is a turn-based game engine, but not a Game Engine. There
plenty of quality Game Engines out there for handling your graphics, audio,
input, networking, and so forth. The Endgame library is instead an engine
and tools for helping you build out and test your game mechanics. It could
even be paired with a Game Engine to flesh out the rest of your game.

This is the initial commit, which is essentially a minimal crate while I
migrate my code from the existing internal repository and crate to this public
one.

## Table of contents

- [What's in a name?](#whats-in-a-name)

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