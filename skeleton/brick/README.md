# Seed

A thin, sans-I/O core for Rust: explicit inputs in, an explicit decision out, and no semantic
judgment of its own.

## Scope

Seed owns one mechanism. The domain supplies meaning, a runtime supplies time and I/O, and Seed
computes the decision. See `PROJECT.md` for the full contract and non-goals.

## Architecture

- [`seed-contract`](crates/seed-contract) — the isolated core.
- [`seed`](crates/seed) — the curated entrypoint you depend on.
- [`seed-governance`](crates/seed-governance) — the unpublished Tianheng gate.

## Contributing

`AGENTS.md` is the contributor and agent guide, including the Definition of Done;
`docs/development-flow.md` is the short checklist.

## License

Licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT), at your option.
