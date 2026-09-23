# Project

## Vision

Seed is a thin, sans-I/O core that turns explicit inputs into an explicit decision.

## Product Positioning

Seed owns one mechanism and no meaning. The domain supplies every semantic judgment; a runtime
supplies time and performs I/O; Seed computes and records.

## Core Contract

- `seed-contract` is the isolated core: no I/O, no ambient clock, no exposed `async fn`.
- `seed` is the curated published entrypoint and re-exports the core; it holds no logic.
- `seed-governance` is an unpublished gate that holds the Tianheng constitution.

## Non-Goals

- A runtime, scheduler, or durable store.
- Judging what a domain value means.

## References

- `docs/domain-language.md` — the canonical vocabulary.
- `AGENTS.seed-law.md` — the generated projection of the accepted constitution.
