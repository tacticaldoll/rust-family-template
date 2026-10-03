# rust-family-template

The template for the tacticaldoll family of Rust repositories: sans-I/O library bricks and the
applications that compose them. It holds the shared governance, the buildable brick skeleton, the
style an application shares with the kit it is born from, and the tools that start a repository
and check any repository for drift.

Repositories are copied from this template, never linked to it: each carries its own complete
governance and builds, tests, and governs itself with no reference back here.

## Layout

- [`FAMILY.md`](FAMILY.md) — the family governance: what is shared, what each repository owns, and
  why.
- [`skeleton/brick`](skeleton/brick) — a sans-I/O library core: `seed-contract`, `seed`,
  `seed-governance`.
- [`style/app`](style/app) — the style a consumer application shares: the shared `AGENTS.md`
  sections, the whole-file copies, and the reference preambles.
- [`kit/app`](kit/app) — the application birth kit overlaid on the style: an empty `seed` library
  and `seed-governance` holding only the gate-independence law. An application's architecture and
  the rest of its law are its own.
- [`scripts/instantiate.sh`](scripts/instantiate.sh) — start a new brick or application.
- [`scripts/family-check.py`](scripts/family-check.py) — check a repository against a profile.
- [`scripts/profile-check.py`](scripts/profile-check.py) — check that the brick and application
  references share the family style.

The brick skeleton is a real workspace on [Tianheng](https://github.com/tacticaldoll/tianheng)
0.8.0 and passes its own Definition of Done. `seed` is a placeholder product name that the
references use for nothing else.

## Start a repository

```sh
scripts/instantiate.sh <brick|app> <name> ../<name>
cd ../<name> && git init -b main
```

Then replace the placeholder product text — `<Name> In One Sentence`, the axioms, the review
questions, `PROJECT.md`, `docs/domain-language.md`, and the core itself — and grow the constitution
with the product.

For an application, write the intent in `PROJECT.md` and fill the `repository-owned` slots in
`AGENTS.md`, `README.md`, and `docs/domain-language.md`; its architecture and law arrive through
its first OpenSpec changes. Either way the new repository passes its own Definition of Done at
birth and neither references nor depends on this template afterwards.

## Check a repository

```sh
scripts/family-check.py <brick|app> <name> ../<name>
```

It reports every drift finding against the profile and exits non-zero on any; it never writes to
the repository.

## License

Licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT), at your option.
