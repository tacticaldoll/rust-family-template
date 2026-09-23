# Project

## Vision

Seed is an application that composes sans-I/O bricks behind a thin I/O shell.

## Product Positioning

Seed is a consumer: it owns its effects, its persistence, and the wiring between the bricks it
composes, and delegates every hard mechanic to those bricks.

## Core Contract

- `seed` is the application crate: `crate::domain` is the pure functional core, `crate::shell`
  owns every effect, and the binary drives the shell.
- `seed-governance` is an unpublished gate that holds the Tianheng constitution.
- Composed bricks: none yet. Name each brick here with the mechanism it owns.

## Non-Goals

- Reimplementing a mechanism a composed brick owns.
- Offering its library target as a reusable API: the library exists so the binary and its tests
  share one core.

## References

- `docs/domain-language.md` — the canonical vocabulary.
- `AGENTS.seed-law.md` — the generated projection of the accepted constitution.
