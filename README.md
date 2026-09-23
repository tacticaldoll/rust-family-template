# rust-family-template

The template for the tacticaldoll family of Rust repositories: sans-I/O library bricks and the
applications that compose them. It holds the shared governance, two buildable skeletons, and the
tools that stamp out a new repository and check an existing one for drift.

Repositories are copied from this template, never linked to it: each carries its own complete
governance and builds, tests, and governs itself with no reference back here.

## Layout

- [`FAMILY.md`](FAMILY.md) — the family governance: what is shared, what each repository owns, and
  why.
- [`skeleton/brick`](skeleton/brick) — a sans-I/O library core: `seed-contract`, `seed`,
  `seed-governance`.
- [`skeleton/app`](skeleton/app) — a consumer application: `seed` with a pure `domain` and an
  effectful `shell`, plus `seed-governance`.
- [`scripts/instantiate.sh`](scripts/instantiate.sh) — stamp out a new repository from a profile.
- [`scripts/family-check.py`](scripts/family-check.py) — check a repository against a profile.

Both skeletons are real workspaces on [Tianheng](https://github.com/tacticaldoll/tianheng) 0.6.1
and pass their own Definition of Done. `seed` is a placeholder product name that the skeletons use
for nothing else.

## Start a repository

```sh
scripts/instantiate.sh brick <name> ../<name>
cd ../<name> && git init -b main
```

Then replace the placeholder product text — `<Name> In One Sentence`, the axioms, the review
questions, `PROJECT.md`, `docs/domain-language.md`, and the core itself — and grow the constitution
with the product.

## Check a repository

```sh
scripts/family-check.py brick <name> ../<name>
```

It reports every drift finding against the profile and exits non-zero on any; it never writes to
the repository.

## License

Licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT), at your option.
