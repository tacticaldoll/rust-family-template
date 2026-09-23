# Seed

An application that composes sans-I/O bricks behind a thin I/O shell.

## Scope

Seed owns its effects, persistence, and wiring; the bricks it composes own the hard mechanics. See
`PROJECT.md` for the full contract and non-goals.

## Architecture

- [`seed`](crates/seed) — the application: a pure `domain` core driven by an effectful `shell`.
- [`seed-governance`](crates/seed-governance) — the unpublished Tianheng gate.

## Contributing

`AGENTS.md` is the contributor and agent guide, including the Definition of Done;
`docs/development-flow.md` is the short checklist.

## License

Licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT), at your option.
