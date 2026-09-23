# seed-governance

Executable architectural governance for the Seed workspace — the Tianheng constitution.

This crate is an internal gate, not a published library (`publish = false`). It depends only on
the [Tianheng](https://github.com/tacticaldoll/tianheng) composed adopter surface and holds the
workspace's dependency boundaries, the core's sans-I/O purity, the facade's re-exports-only shape,
workspace coverage, and the accepted constitution's generated projection, `AGENTS.seed-law.md`.

Run it from the workspace root:

```sh
cargo run -p seed-governance -- check --manifest-path Cargo.toml
```

Regenerate the projection after a deliberate, reviewed law change:

```sh
BLESS=1 cargo test -p seed-governance law_projection_is_fresh
```

Part of [Seed](https://github.com/tacticaldoll/seed).

## License

Licensed under either of [Apache-2.0](https://github.com/tacticaldoll/seed/blob/main/LICENSE-APACHE) or [MIT](https://github.com/tacticaldoll/seed/blob/main/LICENSE-MIT), at your option.
