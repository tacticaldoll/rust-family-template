# rust-family-template

The template for the tacticaldoll family of Rust repositories: sans-I/O library bricks and the
applications that compose them. It holds the shared governance, the buildable brick skeleton, the
style an application shares, and the tools that stamp out a brick and check any repository for
drift.

Repositories are copied from this template, never linked to it: each carries its own complete
governance and builds, tests, and governs itself with no reference back here.

## Layout

- [`FAMILY.md`](FAMILY.md) — the family governance: what is shared, what each repository owns, and
  why.
- [`skeleton/brick`](skeleton/brick) — a sans-I/O library core: `seed-contract`, `seed`,
  `seed-governance`.
- [`style/app`](style/app) — the style a consumer application shares: the shared `AGENTS.md`
  sections, the whole-file copies, and the reference preambles. It is not a workspace; an
  application's architecture and law are its own.
- [`scripts/instantiate.sh`](scripts/instantiate.sh) — stamp out a new brick.
- [`scripts/family-check.py`](scripts/family-check.py) — check a repository against a profile.
- [`scripts/profile-check.py`](scripts/profile-check.py) — check that the brick skeleton and the
  application style share the family style.

The brick skeleton is a real workspace on [Tianheng](https://github.com/tacticaldoll/tianheng)
0.7.0 and passes its own Definition of Done. `seed` is a placeholder product name that the
references use for nothing else.

## Start a repository

```sh
scripts/instantiate.sh brick <name> ../<name>
cd ../<name> && git init -b main
```

Then replace the placeholder product text — `<Name> In One Sentence`, the axioms, the review
questions, `PROJECT.md`, `docs/domain-language.md`, and the core itself — and grow the constitution
with the product.

An application is not stamped out. It copies the files in `style/app/` (renaming `seed`), fills
the `repository-owned` slots with its own identity, axioms, and review questions, and brings
everything else `FAMILY.md` lists for every repository — its own workspace and
`[workspace.package]` settings, `<name>-governance` crate and constitution, CI jobs, `PROJECT.md`,
`README.md`, `BACKLOG.md`, `docs/domain-language.md`, and `openspec/specs/`. `family-check.py app`
then holds it to the shared style.

## Check a repository

```sh
scripts/family-check.py <brick|app> <name> ../<name>
```

It reports every drift finding against the profile and exits non-zero on any; it never writes to
the repository.

## License

Licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT), at your option.
