# Seed Tianheng Law Projection

This file is generated from `constitution()` in `crates/seed-governance/src/main.rs`.
The Rust declaration is authoritative; do not edit the projection by hand.
Regenerate it with `BLESS=1 cargo test -p seed-governance law_projection_is_fresh`.

# Constitution: seed

## Static boundaries

### `seed` (crate)

> seed is the application; it must never take a normal dependency on its own governance gate, which judges the workspace from outside it.

- **rule**: forbid dependency on (crates: seed-governance)
- **kind**: crate · **severity**: enforce

### `seed-governance` (crate)

> the governance gate must stay independent of the workspace graph it judges: its normal dependencies are Tianheng's composed adopter surface alone, never an individual governance instrument or a workspace crate under judgment.

- **rule**: restrict dependencies to (only: tianheng)
- **kind**: crate · **severity**: enforce
