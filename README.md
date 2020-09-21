# TNTW

Totally Not Total War - a toy attempt at implementing Total War: Rome 2 in the bevy engine.

This is a project, intended to teach myself game engine deveopment, with the eventual goal
of it being used as a test bed for prototyping reinforcement learning (RL) algorithms for
an AI opponent.

## Implemented Features

- Left click to select units
- Right click to set unit waypoint while selected
- Units should move towards waypoint, and then transition to idle once they arrive
- "S" to stop unit
- "R" to toggle run/walk
- "ESC" to quit

## Coming soon(tm)

- different icon for selected/unselected units
- UI for unit state
- More than one unit
- Ability to select more than one unit

## Why?

I started playing video games during quarantine/lockdown, but the built-in AI for most strategy games sucks.
Rome 2 in particular is fun because its fairly immersive due to its realism, but when the AI do stupid things it
breaks that realism a bit.

I thought: "Well, they did it with Starcraft 2 & AlphaStar, it can't be that hard, right?". I look a bunch of reinforcement
learning courses, and had some toy algorithms running, but applying them to a game without any internal API seemed
like a bit of a daunting task, so I decided to build my own simulator instead (because thats totally easier, right?).

I picked bevy because I use Rust at work, and also the hackernews hype train.
